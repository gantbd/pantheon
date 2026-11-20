//! Typed, read-only composition of the owner evidence required to present one
//! J1 result for human review.

use agora_chronos_adapter::task_conversation_local::{
    ConversationReferenceWireV1, LinkWireV1, LocalReplyV1, LocalRequestV1,
    TASK_CONVERSATION_CAPABILITY_V1, TaskConversationLocalClientV1,
};
use chronos_client::WorkflowWireClientV9;
use chronos_protocol::{Limits, Reference, Revision, workflow_v9};
use moira_client::{
    TaskAgentJourneyReadOnlyClient,
    unix::{TaskAgentJourneyClientLimitsV1, TaskAgentJourneyUnixPort},
};
use moira_protocol::task_agent_journey::{
    self as journey, ActorParticipationRelationshipViewV1, TaskAttemptNamespaceViewV1,
};
use serde_json::json;
use std::{ffi::OsString, fmt, path::PathBuf};

pub const CAPABILITY: &str = "j1.review-link.observe.v1";

const CHRONOS_LIMITS: Limits = Limits {
    max_frame_bytes: 64 * 1024,
    max_id_bytes: 256,
    max_graph_tasks: 1,
    max_dependencies_per_task: 1,
    max_capabilities_per_task: 1,
    max_capability_bytes: 1,
    max_page_entries: 1,
};

#[derive(Clone)]
pub struct J1ReviewLinkEvidenceV1 {
    pub namespace: TaskAttemptNamespaceViewV1,
    pub actor_participation: ActorParticipationRelationshipViewV1,
    pub task_conversation_owner_revision: u64,
    pub task_conversation: LinkWireV1,
    pub chronos_revision: u64,
    pub chronos_reference: Reference,
    pub chronos_snapshot: workflow_v9::Snapshot,
}

impl fmt::Debug for J1ReviewLinkEvidenceV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("J1ReviewLinkEvidenceV1")
            .field("journey", &self.namespace.journey)
            .field("task_id_bytes", &self.namespace.chronos_task_id.len())
            .field("scope_bytes", &self.task_conversation.task.scope.len())
            .field(
                "room_id_bytes",
                &self.task_conversation.conversation.room.len(),
            )
            .field("attempt_number", &self.namespace.attempt_number)
            .field(
                "task_conversation_owner_revision",
                &self.task_conversation_owner_revision,
            )
            .field("chronos_revision", &self.chronos_revision)
            .field("chronos_status", &self.chronos_snapshot.status)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum J1ReviewLinkRefusalV1 {
    InvalidTaskConversationObservation,
    AmbiguousTaskConversationObservation,
    JourneyMismatch,
    TaskAttemptMismatch,
    TaskIdentityMismatch,
    NonCanonicalIncarnation,
    IncarnationMismatch,
    RoomMismatch,
    ThreadIdentityUnavailable,
    PublicationPredatesRelationship,
    InvalidChronosReply,
    NonCanonicalRevision,
    StaleChronosObservation,
    NotReadyForReview,
}

/// Executes the read-only operational composition without accepting semantic
/// identity aliases from command-line arguments.
pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "j1-review-link-observe requires MOIRA_SOCKET TASK_CONVERSATION_SOCKET CHRONOS_DIRECTORY JOURNEY_OWNER_HEX JOURNEY_SEQUENCE";
    let moira_socket = PathBuf::from(args.next().ok_or(usage)?);
    let task_conversation_socket = PathBuf::from(args.next().ok_or(usage)?);
    let chronos_directory = PathBuf::from(args.next().ok_or(usage)?);
    let journey_owner = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "journey owner identity must be UTF-8")?;
    let journey_sequence = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "journey sequence must be UTF-8")?
        .parse::<u64>()?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let journey = journey::JourneyIdV1 {
        owner: journey::OwnerIdV1(parse_hex_16(&journey_owner)?),
        sequence: journey_sequence,
    };

    let port = match TaskAgentJourneyUnixPort::connect(
        &moira_socket,
        TaskAgentJourneyClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(_) => {
            return failure(
                "moira_unavailable",
                "Moira task-agent journey owner is unavailable",
                "start or reattach the selected Moira owner, then re-observe this exact journey",
            );
        }
    };
    let mut moira = TaskAgentJourneyReadOnlyClient::new(port);
    let status_reply = match moira.status() {
        Ok(reply) => reply,
        Err(_) => {
            return failure(
                "moira_unavailable",
                "Moira task-agent journey owner did not return status",
                "inspect or restart the selected Moira owner, then re-observe this exact journey",
            );
        }
    };
    let status = match accepted_status(status_reply) {
        Ok(status) => status,
        Err(error) => {
            return failure(
                "moira_evidence_refused",
                &error.to_string(),
                "inspect the selected Moira owner and exact journey; do not substitute another journey",
            );
        }
    };
    if status.needs_reopen {
        return failure(
            "moira_fenced",
            "Moira reports recovery required before this evidence can be current",
            "restart the selected Moira owner, then re-observe this exact journey",
        );
    }
    let namespace_reply = match moira.task_attempt_namespace(journey) {
        Ok(reply) => reply,
        Err(_) => {
            return failure(
                "moira_unavailable",
                "Moira task-agent journey owner did not return the task-attempt namespace",
                "inspect or restart the selected Moira owner, then re-observe this exact journey",
            );
        }
    };
    let namespace = match accepted_namespace(namespace_reply) {
        Ok(namespace) => namespace,
        Err(error) => {
            return failure(
                "moira_evidence_refused",
                &error.to_string(),
                "inspect the selected Moira journey's exact task-attempt binding; do not invent a task alias",
            );
        }
    };
    let actor_reply = match moira.actor_participation_relationship(journey) {
        Ok(reply) => reply,
        Err(_) => {
            return failure(
                "moira_unavailable",
                "Moira task-agent journey owner did not return actor-participation evidence",
                "inspect or restart the selected Moira owner, then re-observe this exact journey",
            );
        }
    };
    let actor_participation = match accepted_actor_participation(actor_reply) {
        Ok(value) => value,
        Err(error) => {
            return failure(
                "moira_evidence_refused",
                &error.to_string(),
                "inspect the selected Moira journey's admitted participation and publication evidence",
            );
        }
    };

    let task_conversation_reply = match TaskConversationLocalClientV1::new(task_conversation_socket)
        .request(&LocalRequestV1::ByConversation {
            conversation: ConversationReferenceWireV1 {
                room: actor_participation.publication.room_id.clone(),
                thread_root: None,
            },
        }) {
        Ok(reply) => reply,
        Err(_) => {
            return failure(
                "task_conversation_unavailable",
                "task-conversation relationship owner is unavailable",
                "start or reattach the selected task-conversation owner, then re-observe the publication room",
            );
        }
    };

    let chronos_request = workflow_v9::Request {
        version: 9,
        command: workflow_v9::Command::Snapshot {
            task: namespace.chronos_task_id.clone(),
        },
    };
    let chronos_response = match chronos_local::LocalClient::new(chronos_directory)
        .request_workflow_v9(chronos_request)
    {
        Ok(response) => response,
        Err(_) => {
            return failure(
                "chronos_unavailable",
                "Chronos workflow owner is unavailable",
                "start or reattach the selected Chronos owner, then re-observe the exact task",
            );
        }
    };

    let evidence = match qualify_j1_review_link(
        namespace,
        actor_participation,
        task_conversation_reply,
        chronos_response,
        CHRONOS_LIMITS,
    ) {
        Ok(evidence) => evidence,
        Err(refusal) => {
            return failure(
                "evidence_refused",
                &format!("J1 review-link evidence refused: {refusal:?}"),
                refusal_next_action(refusal),
            );
        }
    };

    let participation = evidence.actor_participation.participation;
    let publication = &evidence.actor_participation.publication;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "pantheon",
            "capability": CAPABILITY,
            "complete": true,
            "status": "review_ready_with_result",
            "moira": {
                "owner_revision": status.owner_revision,
                "storage_revision": status.storage_revision,
                "journey_owner": hex(&evidence.namespace.journey.owner.0),
                "journey_sequence": evidence.namespace.journey.sequence,
                "journey_revision": evidence.actor_participation.journey_revision,
                "participation_owner_revision": evidence.actor_participation.participation_owner_revision,
            },
            "task": {
                "external_task_key": hex(&evidence.namespace.task.external_task_key),
                "scope": evidence.task_conversation.task.scope,
                "task_id": evidence.namespace.chronos_task_id,
                "incarnation": evidence.namespace.task.incarnation,
                "attempt": evidence.namespace.attempt_number,
                "chronos_revision": evidence.chronos_revision,
                "workflow_status": "ready_for_review",
            },
            "participation": {
                "id": hex(&participation.participation.0),
                "session": hex(&participation.session.0),
                "profile": hex(&participation.profile.0),
                "member": hex(&participation.member.0),
            },
            "publication": {
                "actor": publication.actor,
                "effect": hex(&publication.effect),
                "source": hex(&publication.source),
                "room_sequence": publication.room_sequence,
                "room_id": publication.room_id,
                "message_sequence": publication.message_sequence,
                "message_id": hex(&publication.message_id),
                "agora_revision": publication.agora_revision,
            },
            "relationship": {
                "owner": TASK_CONVERSATION_CAPABILITY_V1,
                "owner_revision": evidence.task_conversation_owner_revision,
                "link_id": evidence.task_conversation.link_id,
                "operation_id": evidence.task_conversation.operation_id,
            },
            "next_action": "present this exact content-free relationship and the separately retained result to the named human acceptance owner; acceptance and Done remain separate",
            "boundary": "read-only J1 review-link evidence only; no result bytes, publication, delivery claim, task mutation, acceptance, Done, go-no-go, or rollout authority",
        }))?
    );
    Ok(())
}

/// Composes already public owner observations without mutating any owner.
///
/// The task-conversation owner supplies the exact scope relationship. Chronos
/// V9 supplies current task truth but does not publish that scope, so this
/// function deliberately does not reattribute the relationship owner's scope
/// to Chronos.
pub fn qualify_j1_review_link(
    namespace: TaskAttemptNamespaceViewV1,
    actor_participation: ActorParticipationRelationshipViewV1,
    mut task_conversation_reply: LocalReplyV1,
    chronos_response: workflow_v9::Response,
    chronos_limits: Limits,
) -> Result<J1ReviewLinkEvidenceV1, J1ReviewLinkRefusalV1> {
    if task_conversation_reply.version != 1
        || task_conversation_reply.capability != TASK_CONVERSATION_CAPABILITY_V1
        || !task_conversation_reply.complete
        || task_conversation_reply.status != "current"
        || task_conversation_reply.refusal.is_some()
        || task_conversation_reply.links.is_empty()
        || task_conversation_reply.link_count < task_conversation_reply.links.len()
    {
        return Err(J1ReviewLinkRefusalV1::InvalidTaskConversationObservation);
    }
    let task_conversation_owner_revision = task_conversation_reply
        .revision
        .filter(|revision| *revision > 0)
        .ok_or(J1ReviewLinkRefusalV1::InvalidTaskConversationObservation)?;
    if namespace.journey != actor_participation.journey {
        return Err(J1ReviewLinkRefusalV1::JourneyMismatch);
    }
    let admitted_attempt = actor_participation.participation.attempt;
    if admitted_attempt.task_key != namespace.task.external_task_key
        || admitted_attempt.incarnation != namespace.task.incarnation
        || admitted_attempt.attempt != namespace.attempt_number
    {
        return Err(J1ReviewLinkRefusalV1::TaskAttemptMismatch);
    }
    if actor_participation.authority_principal != actor_participation.publication.subject {
        return Err(J1ReviewLinkRefusalV1::JourneyMismatch);
    }
    let mut task_named = task_conversation_reply
        .links
        .drain(..)
        .filter(|link| link.task.task == namespace.chronos_task_id)
        .collect::<Vec<_>>();
    if task_named.is_empty() {
        return Err(J1ReviewLinkRefusalV1::TaskIdentityMismatch);
    }
    if task_named
        .iter()
        .any(|link| canonical_u64(&link.task.incarnation).is_none())
    {
        return Err(J1ReviewLinkRefusalV1::NonCanonicalIncarnation);
    }
    task_named
        .retain(|link| canonical_u64(&link.task.incarnation) == Some(namespace.task.incarnation));
    if task_named.is_empty() {
        return Err(J1ReviewLinkRefusalV1::IncarnationMismatch);
    }
    task_named.retain(|link| link.task.attempt == Some(namespace.attempt_number));
    if task_named.is_empty() {
        return Err(J1ReviewLinkRefusalV1::TaskAttemptMismatch);
    }
    task_named.retain(|link| link.conversation.room == actor_participation.publication.room_id);
    if task_named.is_empty() {
        return Err(J1ReviewLinkRefusalV1::RoomMismatch);
    }
    if task_named
        .iter()
        .all(|link| link.conversation.thread_root.is_some())
    {
        return Err(J1ReviewLinkRefusalV1::ThreadIdentityUnavailable);
    }
    task_named.retain(|link| link.conversation.thread_root.is_none());
    let [task_conversation] = task_named.as_slice() else {
        return Err(J1ReviewLinkRefusalV1::AmbiguousTaskConversationObservation);
    };
    let task_conversation = task_conversation.clone();
    if actor_participation.publication.agora_revision < task_conversation.agora_revision {
        return Err(J1ReviewLinkRefusalV1::PublicationPredatesRelationship);
    }

    let request = workflow_v9::Request {
        version: 9,
        command: workflow_v9::Command::Snapshot {
            task: namespace.chronos_task_id.clone(),
        },
    };
    chronos_response
        .validate_for(&request, chronos_limits)
        .map_err(|_| J1ReviewLinkRefusalV1::InvalidChronosReply)?;
    let workflow_v9::Outcome::Snapshot {
        value: Some(entry),
        revision: Some(revision),
    } = chronos_response.outcome
    else {
        return Err(J1ReviewLinkRefusalV1::InvalidChronosReply);
    };
    if entry.reference.task != namespace.chronos_task_id {
        return Err(J1ReviewLinkRefusalV1::TaskIdentityMismatch);
    }
    let chronos_incarnation = canonical_u64(&entry.reference.incarnation)
        .ok_or(J1ReviewLinkRefusalV1::NonCanonicalIncarnation)?;
    if chronos_incarnation != namespace.task.incarnation {
        return Err(J1ReviewLinkRefusalV1::IncarnationMismatch);
    }
    if entry.snapshot.attempt_number != Some(namespace.attempt_number) {
        return Err(J1ReviewLinkRefusalV1::TaskAttemptMismatch);
    }
    if entry.snapshot.status != workflow_v9::Status::ReadyForReview
        || entry.snapshot.execution_phase != workflow_v9::ExecutionPhase::Succeeded
        || entry.snapshot.blocker.is_some()
    {
        return Err(J1ReviewLinkRefusalV1::NotReadyForReview);
    }
    let chronos_revision = finite_revision(&revision)?;
    let relationship_chronos_revision = canonical_u64(&task_conversation.chronos_revision)
        .ok_or(J1ReviewLinkRefusalV1::NonCanonicalRevision)?;
    if chronos_revision < namespace.chronos_revision
        || chronos_revision < relationship_chronos_revision
    {
        return Err(J1ReviewLinkRefusalV1::StaleChronosObservation);
    }

    Ok(J1ReviewLinkEvidenceV1 {
        namespace,
        actor_participation,
        task_conversation_owner_revision,
        task_conversation,
        chronos_revision,
        chronos_reference: entry.reference,
        chronos_snapshot: entry.snapshot,
    })
}

fn finite_revision(value: &Revision) -> Result<u64, J1ReviewLinkRefusalV1> {
    match value {
        Revision::Finite(value) => {
            canonical_u64(value).ok_or(J1ReviewLinkRefusalV1::NonCanonicalRevision)
        }
        Revision::Exhausted => Err(J1ReviewLinkRefusalV1::NonCanonicalRevision),
    }
}

fn canonical_u64(value: &str) -> Option<u64> {
    if value.is_empty()
        || value.len() > 20
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return None;
    }
    value.parse().ok()
}

fn refusal_next_action(refusal: J1ReviewLinkRefusalV1) -> &'static str {
    match refusal {
        J1ReviewLinkRefusalV1::InvalidTaskConversationObservation
        | J1ReviewLinkRefusalV1::AmbiguousTaskConversationObservation
        | J1ReviewLinkRefusalV1::TaskIdentityMismatch
        | J1ReviewLinkRefusalV1::IncarnationMismatch
        | J1ReviewLinkRefusalV1::TaskAttemptMismatch
        | J1ReviewLinkRefusalV1::RoomMismatch
        | J1ReviewLinkRefusalV1::ThreadIdentityUnavailable
        | J1ReviewLinkRefusalV1::PublicationPredatesRelationship => {
            "inspect the exact task-conversation relationship and Moira journey observations; repair the owner relationship rather than supplying an alias"
        }
        J1ReviewLinkRefusalV1::JourneyMismatch => {
            "re-observe the exact Moira journey; do not combine evidence from another journey"
        }
        J1ReviewLinkRefusalV1::NonCanonicalIncarnation
        | J1ReviewLinkRefusalV1::NonCanonicalRevision
        | J1ReviewLinkRefusalV1::InvalidChronosReply => {
            "stop composition and inspect owner protocol compatibility; do not normalize the invalid owner value"
        }
        J1ReviewLinkRefusalV1::StaleChronosObservation
        | J1ReviewLinkRefusalV1::NotReadyForReview => {
            "re-observe the exact Chronos task; wait for current ReadyForReview evidence or return the task to its responsible workflow"
        }
    }
}

fn accepted_status(
    reply: journey::ReplyV1,
) -> Result<journey::StatusV1, Box<dyn std::error::Error>> {
    match reply {
        journey::ReplyV1::Accepted(outcome) => match *outcome {
            journey::OutcomeV1::Status(status) => Ok(status),
            _ => Err("Moira returned a different outcome for owner status".into()),
        },
        journey::ReplyV1::Refused(refusal) => {
            Err(format!("Moira refused owner status: {refusal:?}").into())
        }
    }
}

fn accepted_namespace(
    reply: journey::ReplyV1,
) -> Result<TaskAttemptNamespaceViewV1, Box<dyn std::error::Error>> {
    match reply {
        journey::ReplyV1::Accepted(outcome) => match *outcome {
            journey::OutcomeV1::TaskAttemptNamespace(Some(value)) => Ok(value),
            journey::OutcomeV1::TaskAttemptNamespace(None) => {
                Err("Moira has no retained task namespace for this journey".into())
            }
            _ => Err("Moira returned a different outcome for task namespace".into()),
        },
        journey::ReplyV1::Refused(refusal) => {
            Err(format!("Moira refused task namespace observation: {refusal:?}").into())
        }
    }
}

fn accepted_actor_participation(
    reply: journey::ReplyV1,
) -> Result<ActorParticipationRelationshipViewV1, Box<dyn std::error::Error>> {
    match reply {
        journey::ReplyV1::Accepted(outcome) => match *outcome {
            journey::OutcomeV1::ActorParticipationRelationship(Some(value)) => Ok(value),
            journey::OutcomeV1::ActorParticipationRelationship(None) => Err(
                "Moira has no retained actor-participation relationship for this journey".into(),
            ),
            _ => Err("Moira returned a different actor-participation outcome".into()),
        },
        journey::ReplyV1::Refused(refusal) => {
            Err(format!("Moira refused actor-participation observation: {refusal:?}").into())
        }
    }
}

fn parse_hex_16(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32 {
        return Err("journey owner must be exactly 32 hexadecimal digits".into());
    }
    let mut output = [0_u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair)?;
        output[index] = u8::from_str_radix(text, 16)
            .map_err(|_| "journey owner must contain only hexadecimal digits")?;
    }
    Ok(output)
}

fn hex<const N: usize>(value: &[u8; N]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn failure(
    status: &str,
    detail: &str,
    next_action: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "pantheon",
            "capability": CAPABILITY,
            "complete": false,
            "status": status,
            "detail": detail,
            "next_action": next_action,
            "boundary": "no partial owner evidence is promoted to a J1 review link",
        }))?
    );
    Err(std::io::Error::other(detail).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agora_chronos_adapter::task_conversation_local::{
        ConversationReferenceWireV1, TaskReferenceWireV1,
    };
    use moira_protocol::{
        TaskRefV1, WorkflowIdV1, agent_participation as agent, deliberation,
        task_agent_journey::{self as journey, ActorPublicationEvidenceViewV1},
    };

    const LIMITS: Limits = Limits {
        max_frame_bytes: 64 * 1024,
        max_id_bytes: 256,
        max_graph_tasks: 1,
        max_dependencies_per_task: 1,
        max_capabilities_per_task: 1,
        max_capability_bytes: 1,
        max_page_entries: 1,
    };

    fn journey_id() -> journey::JourneyIdV1 {
        journey::JourneyIdV1 {
            owner: journey::OwnerIdV1([1; 16]),
            sequence: 2,
        }
    }

    fn namespace() -> TaskAttemptNamespaceViewV1 {
        TaskAttemptNamespaceViewV1 {
            journey: journey_id(),
            expected_journey_revision: 0,
            task: TaskRefV1 {
                external_task_key: [3; 16],
                incarnation: 4,
            },
            attempt_number: 2,
            chronos_revision: 12,
            chronos_task_id: "exact-task".into(),
        }
    }

    fn actor_participation() -> ActorParticipationRelationshipViewV1 {
        ActorParticipationRelationshipViewV1 {
            journey: journey_id(),
            journey_revision: 7,
            participation: agent::BindingV1 {
                participation: agent::ParticipationIdV1([4; 16]),
                workflow: WorkflowIdV1([5; 16]),
                deliberation: deliberation::DeliberationIdV1([6; 16]),
                member: deliberation::EnsembleMemberIdV1([7; 16]),
                session: agent::AgentSessionIdV1([8; 16]),
                profile: agent::ProfileIdV1([9; 16]),
                attempt: agent::AttemptV1 {
                    task_key: [3; 16],
                    incarnation: 4,
                    attempt: 2,
                },
                method: deliberation::MethodRefV1 {
                    machine_digest: [10; 32],
                    state_digest: [11; 32],
                },
                grounding: deliberation::GroundingRefV1 {
                    authority_digest: [12; 32],
                    surface_digest: [13; 32],
                    depth: deliberation::SurfaceDepthV1::Activated,
                },
                persona: deliberation::PersonaLensV1::Researcher,
            },
            participation_owner_revision: 6,
            authority_decision: [14; 16],
            authority_principal: [15; 16],
            authority_epoch: 2,
            authority_expires_at: 2_000,
            publication: ActorPublicationEvidenceViewV1 {
                effect: [16; 16],
                source: [17; 16],
                room_sequence: 3,
                room_id: "exact-room".into(),
                message_sequence: 4,
                message_id: [18; 16],
                agora_revision: 20,
                actor: "exact-agent".into(),
                subject: [15; 16],
            },
        }
    }

    fn task_conversation() -> LocalReplyV1 {
        LocalReplyV1 {
            version: 1,
            capability: TASK_CONVERSATION_CAPABILITY_V1.into(),
            complete: true,
            status: "current".into(),
            revision: Some(3),
            link_count: 4,
            links: vec![LinkWireV1 {
                link_id: "11111111111111111111111111111111".into(),
                task: TaskReferenceWireV1 {
                    scope: "product-delivery".into(),
                    task: "exact-task".into(),
                    incarnation: "4".into(),
                    attempt: Some(2),
                },
                conversation: ConversationReferenceWireV1 {
                    room: "exact-room".into(),
                    thread_root: None,
                },
                operation_id: "22222222222222222222222222222222".into(),
                principal_id: "33333333333333333333333333333333".into(),
                created_at_unix_seconds: 100,
                visibility_principal_id: None,
                chronos_revision: "13".into(),
                agora_revision: 19,
                observed_at_unix_seconds: 101,
            }],
            refusal: None,
        }
    }

    fn chronos(status: workflow_v9::Status, revision: &str) -> workflow_v9::Response {
        let execution_phase = match status {
            workflow_v9::Status::ReadyForReview | workflow_v9::Status::Done => {
                workflow_v9::ExecutionPhase::Succeeded
            }
            workflow_v9::Status::InProgress => workflow_v9::ExecutionPhase::Running,
            _ => workflow_v9::ExecutionPhase::Ready,
        };
        workflow_v9::Response {
            version: 9,
            outcome: workflow_v9::Outcome::Snapshot {
                value: Some(workflow_v9::Entry {
                    reference: Reference {
                        task: "exact-task".into(),
                        incarnation: "4".into(),
                    },
                    snapshot: workflow_v9::Snapshot {
                        status,
                        blocker: None,
                        execution_phase,
                        attempt_number: Some(2),
                        dependency_count: 0,
                        required_capability_count: 0,
                        priority: 0,
                        max_attempts: 3,
                    },
                }),
                revision: Some(Revision::Finite(revision.into())),
            },
        }
    }

    #[test]
    fn qualifies_exact_owner_evidence_without_leaking_identity_in_debug() {
        let evidence = qualify_j1_review_link(
            namespace(),
            actor_participation(),
            task_conversation(),
            chronos(workflow_v9::Status::ReadyForReview, "14"),
            LIMITS,
        )
        .unwrap();

        assert_eq!(evidence.task_conversation.task.scope, "product-delivery");
        assert_eq!(evidence.task_conversation.conversation.room, "exact-room");
        assert_eq!(evidence.chronos_revision, 14);
        let debug = format!("{evidence:?}");
        assert!(!debug.contains("exact-task"));
        assert!(!debug.contains("exact-room"));
        assert!(!debug.contains("exact-agent"));
    }

    #[test]
    fn refuses_noncanonical_incarnation_and_thread_aliases() {
        let mut alias = task_conversation();
        alias.links[0].task.incarnation = "04".into();
        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                alias,
                chronos(workflow_v9::Status::ReadyForReview, "14"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::NonCanonicalIncarnation
        );

        let mut threaded = task_conversation();
        threaded.links[0].conversation.thread_root = Some(4);
        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                threaded,
                chronos(workflow_v9::Status::ReadyForReview, "14"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::ThreadIdentityUnavailable
        );
    }

    #[test]
    fn accepts_unrelated_room_links_but_refuses_ambiguous_exact_matches() {
        let mut shared_room = task_conversation();
        let mut unrelated = shared_room.links[0].clone();
        unrelated.link_id = "44444444444444444444444444444444".into();
        unrelated.task.task = "other-task".into();
        shared_room.links.push(unrelated);
        let evidence = qualify_j1_review_link(
            namespace(),
            actor_participation(),
            shared_room,
            chronos(workflow_v9::Status::ReadyForReview, "14"),
            LIMITS,
        )
        .unwrap();
        assert_eq!(evidence.task_conversation_owner_revision, 3);

        let mut ambiguous = task_conversation();
        ambiguous.links.push(ambiguous.links[0].clone());
        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                ambiguous,
                chronos(workflow_v9::Status::ReadyForReview, "14"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::AmbiguousTaskConversationObservation
        );
    }

    #[test]
    fn refuses_room_substitution_stale_status_and_non_review_status() {
        let mut room = task_conversation();
        room.links[0].conversation.room = "other-room".into();
        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                room,
                chronos(workflow_v9::Status::ReadyForReview, "14"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::RoomMismatch
        );

        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                task_conversation(),
                chronos(workflow_v9::Status::ReadyForReview, "12"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::StaleChronosObservation
        );

        assert_eq!(
            qualify_j1_review_link(
                namespace(),
                actor_participation(),
                task_conversation(),
                chronos(workflow_v9::Status::InProgress, "14"),
                LIMITS,
            )
            .unwrap_err(),
            J1ReviewLinkRefusalV1::NotReadyForReview
        );
    }
}
