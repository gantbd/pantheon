//! Strict, read-only composition of the exact result presented for J1 review.
//!
//! This module projects `design/runes/j1-strict-review-evidence.machine.rune`.
//! It issues evidence identity only. It grants no human disposition, Chronos
//! mutation, publication, acceptance, `Done`, rollout, or go/no-go authority.

use crate::{j1_agent_result::OwnerBoundAgentResult, j1_review_link::J1ReviewLinkEvidenceV1};
use agora_protocol::v2::{IdentifiedMessagePageV1, MessageIdV1};
use chronos_protocol::workflow_v9;
use daimon_protocol::{
    AgentSessionIdV1, AgentTurnIdV1, ContentDigestV1, ExecutionIdV1, TaskAttemptRefV1,
};
use moira_protocol::{
    agent_participation::{self as participation, ParticipationViewV1},
    task_agent_journey::JourneyIdV1,
};
use sha2::{Digest, Sha256};
use std::fmt;

const RECEIPT_DOMAIN: &[u8] = b"pantheon.j1.strict-review-evidence.v1\0";
const MAX_CANONICAL_FIELD_BYTES: usize = 4_096;

#[derive(Clone, Eq, PartialEq)]
pub struct StrictJ1ReviewEvidenceV1 {
    pub receipt_digest: [u8; 32],
    pub journey: JourneyIdV1,
    pub task_attempt: TaskAttemptRefV1,
    pub session: AgentSessionIdV1,
    pub turn: AgentTurnIdV1,
    pub execution: ExecutionIdV1,
    pub output_digest: ContentDigestV1,
    pub chronos_revision: u64,
    pub agora_revision: u64,
    pub message_id: MessageIdV1,
    pub message_sequence: u64,
}

impl fmt::Debug for StrictJ1ReviewEvidenceV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StrictJ1ReviewEvidenceV1")
            .field("journey", &self.journey)
            .field("task_attempt", &"<opaque>")
            .field("session", &self.session)
            .field("turn", &self.turn)
            .field("execution", &self.execution)
            .field("output_digest", &self.output_digest)
            .field("receipt_digest", &"<opaque>")
            .field("chronos_revision", &self.chronos_revision)
            .field("agora_revision", &self.agora_revision)
            .field("message_id", &self.message_id)
            .field("message_sequence", &self.message_sequence)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrictJ1ReviewRefusalV1 {
    ReviewLinkNotCurrent,
    ParticipationMismatch,
    ParticipationNotTerminal,
    SessionMismatch,
    TurnMismatch,
    TerminalCursorMismatch,
    TerminalDigestMismatch,
    TaskKeyMismatch,
    IncarnationMismatch,
    AttemptMismatch,
    RoomMismatch,
    AgoraRevisionStale,
    RetentionGap,
    TruncatedObservation,
    MessageMissing,
    MessageIdentityMismatch,
    MessageAmbiguous,
    MessageSequenceMismatch,
    MessageAuthorMismatch,
    MessageBodyMismatch,
    OutputDigestMismatch,
    CanonicalFieldTooLarge,
}

/// Qualifies the exact owner-bound Daimon result as the exact Moira-attributed
/// Agora publication currently linked to a Chronos task awaiting human review.
///
/// A refusal is a typed Maybe-machine result. Callers must not substitute a
/// similar task, room, actor, message, or body, and must not promote a refusal
/// to a Boolean decision.
pub fn qualify_strict_j1_review_evidence(
    review: &J1ReviewLinkEvidenceV1,
    participation: &ParticipationViewV1,
    result: &OwnerBoundAgentResult,
    page: &IdentifiedMessagePageV1,
) -> Result<StrictJ1ReviewEvidenceV1, StrictJ1ReviewRefusalV1> {
    if review.chronos_snapshot.status != workflow_v9::Status::ReadyForReview
        || review.chronos_snapshot.execution_phase != workflow_v9::ExecutionPhase::Succeeded
        || review.chronos_snapshot.blocker.is_some()
        || review.chronos_snapshot.attempt_number != Some(review.namespace.attempt_number)
        || review.chronos_reference.task != review.namespace.chronos_task_id
        || review.actor_participation.journey != review.namespace.journey
        || review.actor_participation.authority_principal
            != review.actor_participation.publication.subject
    {
        return Err(StrictJ1ReviewRefusalV1::ReviewLinkNotCurrent);
    }
    let admitted_attempt = review.actor_participation.participation.attempt;
    if admitted_attempt.task_key != review.namespace.task.external_task_key {
        return Err(StrictJ1ReviewRefusalV1::TaskKeyMismatch);
    }
    if admitted_attempt.incarnation != review.namespace.task.incarnation {
        return Err(StrictJ1ReviewRefusalV1::IncarnationMismatch);
    }
    if admitted_attempt.attempt != review.namespace.attempt_number {
        return Err(StrictJ1ReviewRefusalV1::AttemptMismatch);
    }
    if review.task_conversation.task.task != review.namespace.chronos_task_id
        || review.chronos_reference.incarnation != review.namespace.task.incarnation.to_string()
    {
        return Err(StrictJ1ReviewRefusalV1::ReviewLinkNotCurrent);
    }
    if review.task_conversation.task.incarnation != review.namespace.task.incarnation.to_string() {
        return Err(StrictJ1ReviewRefusalV1::IncarnationMismatch);
    }
    if review.task_conversation.task.attempt != Some(review.namespace.attempt_number) {
        return Err(StrictJ1ReviewRefusalV1::AttemptMismatch);
    }

    if participation.binding != review.actor_participation.participation {
        return Err(StrictJ1ReviewRefusalV1::ParticipationMismatch);
    }
    let terminal = participation
        .terminal
        .ok_or(StrictJ1ReviewRefusalV1::ParticipationNotTerminal)?;
    if participation.phase != participation::PhaseV1::Terminal
        || participation.active_turn.is_some()
        || participation.pending.is_some()
        || terminal.disposition != participation::DispositionV1::Completed
        || terminal.stop_reason != participation::StopReasonV1::EndTurn
    {
        return Err(StrictJ1ReviewRefusalV1::ParticipationNotTerminal);
    }
    if result.session.into_bytes() != participation.binding.session.0 {
        return Err(StrictJ1ReviewRefusalV1::SessionMismatch);
    }
    if result.turn.into_bytes() != terminal.turn.0 {
        return Err(StrictJ1ReviewRefusalV1::TurnMismatch);
    }
    if participation.last_event_cursor != Some(result.finished_cursor) {
        return Err(StrictJ1ReviewRefusalV1::TerminalCursorMismatch);
    }
    if terminal.output_digest != result.output_digest.into_bytes() {
        return Err(StrictJ1ReviewRefusalV1::TerminalDigestMismatch);
    }

    if result.task_attempt.task_key != review.namespace.task.external_task_key {
        return Err(StrictJ1ReviewRefusalV1::TaskKeyMismatch);
    }
    if result.task_attempt.task_incarnation != review.namespace.task.incarnation {
        return Err(StrictJ1ReviewRefusalV1::IncarnationMismatch);
    }
    if result.task_attempt.attempt_number != review.namespace.attempt_number {
        return Err(StrictJ1ReviewRefusalV1::AttemptMismatch);
    }

    let publication = &review.actor_participation.publication;
    if page.room_id != publication.room_id
        || review.task_conversation.conversation.room != publication.room_id
    {
        return Err(StrictJ1ReviewRefusalV1::RoomMismatch);
    }
    if page.revision < publication.agora_revision
        || page.revision < review.task_conversation.agora_revision
    {
        return Err(StrictJ1ReviewRefusalV1::AgoraRevisionStale);
    }
    if page.oldest_retained_sequence > publication.message_sequence {
        return Err(StrictJ1ReviewRefusalV1::RetentionGap);
    }
    if page.next.is_some() {
        return Err(StrictJ1ReviewRefusalV1::TruncatedObservation);
    }

    let expected_id = MessageIdV1::new(publication.message_id);
    let mut exact_id = page
        .messages
        .iter()
        .filter(|message| message.message_id == expected_id);
    let message = exact_id.next().ok_or_else(|| {
        if page
            .messages
            .iter()
            .any(|message| message.sequence == publication.message_sequence)
        {
            StrictJ1ReviewRefusalV1::MessageIdentityMismatch
        } else {
            StrictJ1ReviewRefusalV1::MessageMissing
        }
    })?;
    if exact_id.next().is_some() {
        return Err(StrictJ1ReviewRefusalV1::MessageAmbiguous);
    }
    if message.sequence != publication.message_sequence {
        return Err(StrictJ1ReviewRefusalV1::MessageSequenceMismatch);
    }
    if message.author != publication.actor {
        return Err(StrictJ1ReviewRefusalV1::MessageAuthorMismatch);
    }
    if message.body.as_bytes() != result.bytes {
        return Err(StrictJ1ReviewRefusalV1::MessageBodyMismatch);
    }
    let observed_digest = ContentDigestV1::new(Sha256::digest(message.body.as_bytes()).into());
    if observed_digest != result.output_digest {
        return Err(StrictJ1ReviewRefusalV1::OutputDigestMismatch);
    }

    let receipt_digest = canonical_receipt_digest(review, participation, result, page, message)?;
    Ok(StrictJ1ReviewEvidenceV1 {
        receipt_digest,
        journey: review.namespace.journey,
        task_attempt: result.task_attempt,
        session: result.session,
        turn: result.turn,
        execution: result.execution,
        output_digest: result.output_digest,
        chronos_revision: review.chronos_revision,
        agora_revision: page.revision,
        message_id: message.message_id,
        message_sequence: message.sequence,
    })
}

fn canonical_receipt_digest(
    review: &J1ReviewLinkEvidenceV1,
    participation: &ParticipationViewV1,
    result: &OwnerBoundAgentResult,
    page: &IdentifiedMessagePageV1,
    message: &agora_protocol::v2::IdentifiedMessageV1,
) -> Result<[u8; 32], StrictJ1ReviewRefusalV1> {
    let mut value = Vec::with_capacity(1_024);
    value.extend_from_slice(RECEIPT_DOMAIN);

    let namespace = &review.namespace;
    put_fixed(&mut value, &namespace.journey.owner.0);
    put_u64(&mut value, namespace.journey.sequence);
    put_u64(&mut value, namespace.expected_journey_revision);
    put_fixed(&mut value, &namespace.task.external_task_key);
    put_u64(&mut value, namespace.task.incarnation);
    put_u32(&mut value, namespace.attempt_number);
    put_u64(&mut value, namespace.chronos_revision);
    put_bytes(&mut value, namespace.chronos_task_id.as_bytes())?;

    let relationship = &review.actor_participation;
    put_u64(&mut value, relationship.journey_revision);
    put_fixed(&mut value, &relationship.participation.participation.0);
    put_fixed(&mut value, &relationship.participation.workflow.0);
    put_fixed(&mut value, &relationship.participation.deliberation.0);
    put_fixed(&mut value, &relationship.participation.member.0);
    put_fixed(&mut value, &relationship.participation.session.0);
    put_fixed(&mut value, &relationship.participation.profile.0);
    put_fixed(&mut value, &relationship.participation.attempt.task_key);
    put_u64(&mut value, relationship.participation.attempt.incarnation);
    put_u32(&mut value, relationship.participation.attempt.attempt);
    put_fixed(
        &mut value,
        &relationship.participation.method.machine_digest,
    );
    put_fixed(&mut value, &relationship.participation.method.state_digest);
    put_fixed(
        &mut value,
        &relationship.participation.grounding.authority_digest,
    );
    put_fixed(
        &mut value,
        &relationship.participation.grounding.surface_digest,
    );
    put_u8(
        &mut value,
        surface_depth_tag(relationship.participation.grounding.depth),
    );
    put_u8(&mut value, persona_tag(relationship.participation.persona));
    put_u64(&mut value, relationship.participation_owner_revision);
    put_fixed(&mut value, &relationship.authority_decision);
    put_fixed(&mut value, &relationship.authority_principal);
    put_u64(&mut value, relationship.authority_epoch);
    put_u64(&mut value, relationship.authority_expires_at);

    let publication = &relationship.publication;
    put_fixed(&mut value, &publication.effect);
    put_fixed(&mut value, &publication.source);
    put_u64(&mut value, publication.room_sequence);
    put_bytes(&mut value, publication.room_id.as_bytes())?;
    put_u64(&mut value, publication.message_sequence);
    put_fixed(&mut value, &publication.message_id);
    put_u64(&mut value, publication.agora_revision);
    put_bytes(&mut value, publication.actor.as_bytes())?;
    put_fixed(&mut value, &publication.subject);

    put_u64(&mut value, review.task_conversation_owner_revision);
    put_bytes(&mut value, review.task_conversation.link_id.as_bytes())?;
    put_bytes(&mut value, review.task_conversation.task.scope.as_bytes())?;
    put_bytes(&mut value, review.task_conversation.task.task.as_bytes())?;
    put_bytes(
        &mut value,
        review.task_conversation.task.incarnation.as_bytes(),
    )?;
    put_u32(
        &mut value,
        review.task_conversation.task.attempt.unwrap_or_default(),
    );
    put_bytes(
        &mut value,
        review.task_conversation.conversation.room.as_bytes(),
    )?;
    put_bytes(&mut value, review.task_conversation.operation_id.as_bytes())?;
    put_bytes(&mut value, review.task_conversation.principal_id.as_bytes())?;
    put_u64(&mut value, review.task_conversation.created_at_unix_seconds);
    put_optional_bytes(
        &mut value,
        review
            .task_conversation
            .visibility_principal_id
            .as_deref()
            .map(str::as_bytes),
    )?;
    put_bytes(
        &mut value,
        review.task_conversation.chronos_revision.as_bytes(),
    )?;
    put_u64(&mut value, review.task_conversation.agora_revision);
    put_u64(
        &mut value,
        review.task_conversation.observed_at_unix_seconds,
    );
    put_u64(&mut value, review.chronos_revision);
    put_bytes(&mut value, review.chronos_reference.task.as_bytes())?;
    put_bytes(&mut value, review.chronos_reference.incarnation.as_bytes())?;
    put_optional_u32(&mut value, review.chronos_snapshot.attempt_number);
    put_usize(&mut value, review.chronos_snapshot.dependency_count)?;
    put_usize(
        &mut value,
        review.chronos_snapshot.required_capability_count,
    )?;
    put_i32(&mut value, review.chronos_snapshot.priority);
    put_u32(&mut value, review.chronos_snapshot.max_attempts);

    put_u64(&mut value, participation.revision);
    put_optional_u64(&mut value, participation.session_revision);
    put_optional_u64(&mut value, participation.last_event_cursor);
    put_optional_fixed_32(&mut value, participation.profile_document_digest);
    let terminal = participation
        .terminal
        .expect("terminal participation was checked before receipt encoding");
    put_fixed(&mut value, &terminal.turn.0);
    put_fixed(&mut value, &terminal.output_digest);
    put_fixed(&mut value, &terminal.provider_evidence_digest);

    put_fixed(&mut value, &result.session.into_bytes());
    put_fixed(&mut value, &result.turn.into_bytes());
    put_fixed(&mut value, &result.execution.into_bytes());
    put_fixed(&mut value, &result.task_attempt.task_key);
    put_u64(&mut value, result.task_attempt.task_incarnation);
    put_u32(&mut value, result.task_attempt.attempt_number);
    put_u64(&mut value, result.terminal_revision);
    put_u64(&mut value, result.assistant_cursor);
    put_u64(&mut value, result.finished_cursor);
    put_fixed(&mut value, &result.output_digest.into_bytes());

    put_u64(&mut value, page.revision);
    put_bytes(&mut value, page.room_id.as_bytes())?;
    put_u64(&mut value, page.oldest_retained_sequence);
    put_u64(&mut value, page.next_message_sequence);
    put_fixed(&mut value, &message.message_id.into_bytes());
    put_u64(&mut value, message.sequence);
    put_bytes(&mut value, message.author.as_bytes())?;

    Ok(Sha256::digest(value).into())
}

fn put_fixed(value: &mut Vec<u8>, field: &[u8]) {
    value.extend_from_slice(field);
}

fn put_u64(value: &mut Vec<u8>, field: u64) {
    value.extend_from_slice(&field.to_be_bytes());
}

fn put_u32(value: &mut Vec<u8>, field: u32) {
    value.extend_from_slice(&field.to_be_bytes());
}

fn put_i32(value: &mut Vec<u8>, field: i32) {
    value.extend_from_slice(&field.to_be_bytes());
}

fn put_u8(value: &mut Vec<u8>, field: u8) {
    value.push(field);
}

fn put_usize(value: &mut Vec<u8>, field: usize) -> Result<(), StrictJ1ReviewRefusalV1> {
    let field =
        u64::try_from(field).map_err(|_| StrictJ1ReviewRefusalV1::CanonicalFieldTooLarge)?;
    put_u64(value, field);
    Ok(())
}

fn put_optional_u32(value: &mut Vec<u8>, field: Option<u32>) {
    match field {
        Some(field) => {
            put_u8(value, 1);
            put_u32(value, field);
        }
        None => put_u8(value, 0),
    }
}

fn put_optional_u64(value: &mut Vec<u8>, field: Option<u64>) {
    match field {
        Some(field) => {
            put_u8(value, 1);
            put_u64(value, field);
        }
        None => put_u8(value, 0),
    }
}

fn put_optional_fixed_32(value: &mut Vec<u8>, field: Option<[u8; 32]>) {
    match field {
        Some(field) => {
            put_u8(value, 1);
            put_fixed(value, &field);
        }
        None => put_u8(value, 0),
    }
}

fn put_bytes(value: &mut Vec<u8>, field: &[u8]) -> Result<(), StrictJ1ReviewRefusalV1> {
    let length = u32::try_from(field.len())
        .ok()
        .filter(|length| (*length as usize) <= MAX_CANONICAL_FIELD_BYTES)
        .ok_or(StrictJ1ReviewRefusalV1::CanonicalFieldTooLarge)?;
    value.extend_from_slice(&length.to_be_bytes());
    value.extend_from_slice(field);
    Ok(())
}

fn put_optional_bytes(
    value: &mut Vec<u8>,
    field: Option<&[u8]>,
) -> Result<(), StrictJ1ReviewRefusalV1> {
    match field {
        Some(field) => {
            put_u8(value, 1);
            put_bytes(value, field)
        }
        None => {
            put_u8(value, 0);
            Ok(())
        }
    }
}

fn surface_depth_tag(value: moira_protocol::deliberation::SurfaceDepthV1) -> u8 {
    use moira_protocol::deliberation::SurfaceDepthV1;
    match value {
        SurfaceDepthV1::Ambient => 0,
        SurfaceDepthV1::Activated => 1,
        SurfaceDepthV1::Reference => 2,
    }
}

fn persona_tag(value: moira_protocol::deliberation::PersonaLensV1) -> u8 {
    use moira_protocol::deliberation::PersonaLensV1;
    match value {
        PersonaLensV1::Historian => 0,
        PersonaLensV1::IdeaStormer => 1,
        PersonaLensV1::Theorist => 2,
        PersonaLensV1::Researcher => 3,
        PersonaLensV1::RigorReviewer => 4,
        PersonaLensV1::ConvergenceFacilitator => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::j1_review_link::J1ReviewLinkEvidenceV1;
    use agora_chronos_adapter::task_conversation_local::{
        ConversationReferenceWireV1, LinkWireV1, TaskReferenceWireV1,
    };
    use agora_protocol::v2::{IdentifiedMessageV1, PageCursorV2};
    use chronos_protocol::{Reference, workflow_v9};
    use moira_protocol::{
        TaskRefV1, WorkflowIdV1, agent_participation as agent, deliberation,
        task_agent_journey::{
            ActorParticipationRelationshipViewV1, ActorPublicationEvidenceViewV1, JourneyIdV1,
            OwnerIdV1, TaskAttemptNamespaceViewV1,
        },
    };

    const BODY: &str = "The exact bounded result is ready for human review.";

    fn digest(bytes: &[u8]) -> ContentDigestV1 {
        ContentDigestV1::new(Sha256::digest(bytes).into())
    }

    fn review() -> J1ReviewLinkEvidenceV1 {
        let journey = JourneyIdV1 {
            owner: OwnerIdV1([1; 16]),
            sequence: 2,
        };
        J1ReviewLinkEvidenceV1 {
            namespace: TaskAttemptNamespaceViewV1 {
                journey,
                expected_journey_revision: 7,
                task: TaskRefV1 {
                    external_task_key: [3; 16],
                    incarnation: 4,
                },
                attempt_number: 2,
                chronos_revision: 12,
                chronos_task_id: "exact-task".into(),
            },
            actor_participation: ActorParticipationRelationshipViewV1 {
                journey,
                journey_revision: 8,
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
            },
            task_conversation_owner_revision: 3,
            task_conversation: LinkWireV1 {
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
            },
            chronos_revision: 14,
            chronos_reference: Reference {
                task: "exact-task".into(),
                incarnation: "4".into(),
            },
            chronos_snapshot: workflow_v9::Snapshot {
                status: workflow_v9::Status::ReadyForReview,
                blocker: None,
                execution_phase: workflow_v9::ExecutionPhase::Succeeded,
                attempt_number: Some(2),
                dependency_count: 0,
                required_capability_count: 0,
                priority: 0,
                max_attempts: 3,
            },
        }
    }

    fn result() -> OwnerBoundAgentResult {
        OwnerBoundAgentResult {
            session: AgentSessionIdV1::new([8; 16]),
            turn: AgentTurnIdV1::new([22; 16]),
            execution: ExecutionIdV1::new([23; 16]),
            task_attempt: TaskAttemptRefV1 {
                task_key: [3; 16],
                task_incarnation: 4,
                attempt_number: 2,
            },
            terminal_revision: 9,
            assistant_cursor: 1,
            finished_cursor: 2,
            output_digest: digest(BODY.as_bytes()),
            bytes: BODY.as_bytes().to_vec(),
        }
    }

    fn participation() -> agent::ParticipationViewV1 {
        agent::ParticipationViewV1 {
            binding: review().actor_participation.participation,
            revision: 10,
            phase: agent::PhaseV1::Terminal,
            session_revision: Some(11),
            last_event_cursor: Some(2),
            profile_document_digest: Some([24; 32]),
            capabilities: agent::CapabilitiesV1::default(),
            active_turn: None,
            permission: None,
            terminal: Some(agent::TerminalEvidenceV1 {
                turn: agent::AgentTurnIdV1([22; 16]),
                disposition: agent::DispositionV1::Completed,
                stop_reason: agent::StopReasonV1::EndTurn,
                output_digest: digest(BODY.as_bytes()).into_bytes(),
                provider_evidence_digest: [25; 32],
            }),
            pending: None,
        }
    }

    fn page() -> IdentifiedMessagePageV1 {
        IdentifiedMessagePageV1 {
            revision: 21,
            room_id: "exact-room".into(),
            messages: vec![IdentifiedMessageV1 {
                message_id: MessageIdV1::new([18; 16]),
                sequence: 4,
                author: "exact-agent".into(),
                logical_time: 100,
                body: BODY.into(),
                addressed_to: vec![],
            }],
            oldest_retained_sequence: 1,
            next_message_sequence: 5,
            next: None,
        }
    }

    #[test]
    fn issues_stable_content_free_receipt_for_exact_owner_evidence() {
        let first =
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &page())
                .unwrap();
        let second =
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &page())
                .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.task_attempt.attempt_number, 2);
        assert_eq!(first.message_sequence, 4);
        let debug = format!("{first:?}");
        assert!(!debug.contains(BODY));
        assert!(!debug.contains("exact-room"));
        assert!(!debug.contains("exact-agent"));
    }

    #[test]
    fn refuses_each_task_attempt_substitution() {
        let mut wrong_key = result();
        wrong_key.task_attempt.task_key = [99; 16];
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &wrong_key, &page()),
            Err(StrictJ1ReviewRefusalV1::TaskKeyMismatch)
        );

        let mut wrong_incarnation = result();
        wrong_incarnation.task_attempt.task_incarnation = 5;
        assert_eq!(
            qualify_strict_j1_review_evidence(
                &review(),
                &participation(),
                &wrong_incarnation,
                &page()
            ),
            Err(StrictJ1ReviewRefusalV1::IncarnationMismatch)
        );

        let mut wrong_attempt = result();
        wrong_attempt.task_attempt.attempt_number = 3;
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &wrong_attempt, &page()),
            Err(StrictJ1ReviewRefusalV1::AttemptMismatch)
        );
    }

    #[test]
    fn refuses_participation_session_turn_cursor_and_digest_substitution() {
        let mut wrong_binding = participation();
        wrong_binding.binding.profile = agent::ProfileIdV1([99; 16]);
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &wrong_binding, &result(), &page()),
            Err(StrictJ1ReviewRefusalV1::ParticipationMismatch)
        );

        let mut wrong_session = result();
        wrong_session.session = AgentSessionIdV1::new([99; 16]);
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &wrong_session, &page()),
            Err(StrictJ1ReviewRefusalV1::SessionMismatch)
        );

        let mut wrong_turn = result();
        wrong_turn.turn = AgentTurnIdV1::new([99; 16]);
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &wrong_turn, &page()),
            Err(StrictJ1ReviewRefusalV1::TurnMismatch)
        );

        let mut wrong_cursor = participation();
        wrong_cursor.last_event_cursor = Some(3);
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &wrong_cursor, &result(), &page()),
            Err(StrictJ1ReviewRefusalV1::TerminalCursorMismatch)
        );

        let mut wrong_digest = participation();
        wrong_digest.terminal.as_mut().unwrap().output_digest = [99; 32];
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &wrong_digest, &result(), &page()),
            Err(StrictJ1ReviewRefusalV1::TerminalDigestMismatch)
        );
    }

    #[test]
    fn refuses_wrong_message_identity_author_and_body() {
        let mut wrong_id = page();
        wrong_id.messages[0].message_id = MessageIdV1::new([42; 16]);
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &wrong_id),
            Err(StrictJ1ReviewRefusalV1::MessageIdentityMismatch)
        );

        let mut wrong_author = page();
        wrong_author.messages[0].author = "substitute-agent".into();
        assert_eq!(
            qualify_strict_j1_review_evidence(
                &review(),
                &participation(),
                &result(),
                &wrong_author
            ),
            Err(StrictJ1ReviewRefusalV1::MessageAuthorMismatch)
        );

        let mut wrong_body = page();
        wrong_body.messages[0].body.push_str(" changed");
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &wrong_body),
            Err(StrictJ1ReviewRefusalV1::MessageBodyMismatch)
        );
    }

    #[test]
    fn refuses_stale_evicted_truncated_and_ambiguous_observations() {
        let mut stale = page();
        stale.revision = 19;
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &stale),
            Err(StrictJ1ReviewRefusalV1::AgoraRevisionStale)
        );

        let mut evicted = page();
        evicted.oldest_retained_sequence = 5;
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &evicted),
            Err(StrictJ1ReviewRefusalV1::RetentionGap)
        );

        let mut truncated = page();
        truncated.next = Some(PageCursorV2(7));
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &truncated),
            Err(StrictJ1ReviewRefusalV1::TruncatedObservation)
        );

        let mut ambiguous = page();
        ambiguous.messages.push(ambiguous.messages[0].clone());
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &ambiguous),
            Err(StrictJ1ReviewRefusalV1::MessageAmbiguous)
        );
    }

    #[test]
    fn independently_defends_the_owner_result_digest() {
        let mut invalid = result();
        invalid.output_digest = ContentDigestV1::new([0; 32]);
        let mut invalid_participation = participation();
        invalid_participation
            .terminal
            .as_mut()
            .unwrap()
            .output_digest = [0; 32];
        assert_eq!(
            qualify_strict_j1_review_evidence(&review(), &invalid_participation, &invalid, &page()),
            Err(StrictJ1ReviewRefusalV1::OutputDigestMismatch)
        );
    }

    #[test]
    fn owner_revision_changes_produce_a_distinct_receipt() {
        let first =
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &page())
                .unwrap();
        let mut later_page = page();
        later_page.revision += 1;
        let later =
            qualify_strict_j1_review_evidence(&review(), &participation(), &result(), &later_page)
                .unwrap();

        assert_ne!(first.receipt_digest, later.receipt_digest);
    }
}
