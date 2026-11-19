//! Headless controls for Moira-owned task-agent journey admission.

use moira_client::{
    TaskAgentJourneyMutationClient, TaskAgentJourneyReadOnlyClient,
    unix::{
        TaskAgentJourneyClientErrorV1, TaskAgentJourneyClientLimitsV1, TaskAgentJourneyUnixPort,
    },
};
use moira_protocol::{TaskRefV1, agent_participation::ProfileIdV1, task_agent_journey as journey};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

pub const CATALOGUE_CAPABILITY: &str = "moira.task-agent.catalogue.v1";
pub const PREVIEW_CAPABILITY: &str = "moira.task-agent.preview.v1";
pub const INSTANTIATE_CAPABILITY: &str = "moira.task-agent.instantiate.v1";
pub const OBSERVE_CAPABILITY: &str = "moira.task-agent.observe.v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewDocumentV1 {
    version: u64,
    expected_storage_revision: Option<u64>,
    operation: String,
    template: TemplateRefDocumentV1,
    task: TaskDocumentV1,
    profile: String,
    room_policy: String,
    initiating_principal: String,
    accountable_human: String,
    objective_digest: String,
    acceptance_digest: String,
    current_evidence_digest: String,
    requested_ttl: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateRefDocumentV1 {
    id: String,
    version: u64,
    digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskDocumentV1 {
    external_task_key: String,
    incarnation: u64,
    chronos_revision: u64,
    evidence_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InstantiateDocumentV1 {
    version: u64,
    expected_storage_revision: u64,
    operation: String,
    preview_owner: String,
    preview_sequence: u64,
    expected_owner_revision: u64,
    accountable_human: String,
    consequence_digest: String,
}

pub fn catalogue(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = one_socket(&mut args)?;
    let port = match connect(&socket) {
        Ok(port) => port,
        Err(error) => return transport_failure(CATALOGUE_CAPABILITY, error),
    };
    let mut client = TaskAgentJourneyReadOnlyClient::new(port);
    match client.catalogue() {
        Ok(journey::ReplyV1::Accepted(outcome)) => match *outcome {
            journey::OutcomeV1::Catalogue {
                owner,
                owner_revision,
                storage_revision,
                templates,
            } => emit(json!({
                "version": 1,
                "peer": "moira",
                "capability": CATALOGUE_CAPABILITY,
                "complete": true,
                "status": "current",
                "owner": hex(&owner.0),
                "owner_revision": owner_revision,
                "storage_revision": storage_revision,
                "templates": templates.into_iter().map(template_json).collect::<Vec<_>>(),
                "boundary": "catalogue evidence grants no workflow admission, provider access, room membership, or effect authority",
            })),
            _ => failure(
                CATALOGUE_CAPABILITY,
                "invalid_reply",
                "unexpected Moira outcome",
                None,
            ),
        },
        Ok(journey::ReplyV1::Refused(value)) => refusal(CATALOGUE_CAPABILITY, value),
        Err(error) => transport_failure(CATALOGUE_CAPABILITY, error),
    }
}

pub fn preview(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let socket = required(&mut args, "preview requires SOCKET and REQUEST_JSON")?;
    let document = required(&mut args, "preview requires REQUEST_JSON")?;
    no_more(&mut args)?;
    let document: PreviewDocumentV1 = read_document(&PathBuf::from(document))?;
    let expected = document.expected_storage_revision;
    let request = preview_request(document)?;
    let port = match connect(&PathBuf::from(socket)) {
        Ok(port) => port,
        Err(error) => return transport_failure(PREVIEW_CAPABILITY, error),
    };
    let mut client = TaskAgentJourneyMutationClient::new(port);
    match client.preview(expected, request) {
        Ok(journey::ReplyV1::Accepted(outcome)) => match *outcome {
            journey::OutcomeV1::Previewed {
                storage_revision,
                receipt,
            } => {
                let preview = receipt.preview;
                let confirmation = confirmation(&preview);
                emit(json!({
                    "version": 1,
                    "peer": "moira",
                    "capability": PREVIEW_CAPABILITY,
                    "complete": true,
                    "status": "previewed",
                    "storage_revision": storage_revision,
                    "replayed": receipt.replayed,
                    "preview": preview_json(&preview),
                    "required_confirmation": confirmation,
                    "next_action": "review the owner-issued preview, persist its identifiers and exact confirmation, then invoke moira-task-agent-instantiate",
                    "boundary": "preview creates no workflow, room, session, lease, grant, participation, provider call, or peer effect",
                }))
            }
            _ => failure(
                PREVIEW_CAPABILITY,
                "invalid_reply",
                "unexpected Moira outcome",
                None,
            ),
        },
        Ok(journey::ReplyV1::Refused(value)) => refusal(PREVIEW_CAPABILITY, value),
        Err(error) => transport_failure(PREVIEW_CAPABILITY, error),
    }
}

pub fn instantiate(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = required(
        &mut args,
        "instantiate requires SOCKET REQUEST_JSON CONFIRMATION",
    )?;
    let document = required(&mut args, "instantiate requires REQUEST_JSON CONFIRMATION")?;
    let supplied = required(&mut args, "instantiate requires CONFIRMATION")?
        .into_string()
        .map_err(|_| "confirmation must be UTF-8")?;
    no_more(&mut args)?;
    let document: InstantiateDocumentV1 = read_document(&PathBuf::from(document))?;
    let expected_storage_revision = document.expected_storage_revision;
    let request = instantiate_request(document)?;
    let required = instantiate_confirmation(&request);
    if supplied != required {
        return failure(
            INSTANTIATE_CAPABILITY,
            "confirmation_refused",
            "exact owner-bound human confirmation required",
            Some(json!({"required_confirmation": required})),
        );
    }
    let port = match connect(&PathBuf::from(socket)) {
        Ok(port) => port,
        Err(error) => return transport_failure(INSTANTIATE_CAPABILITY, error),
    };
    let mut client = TaskAgentJourneyMutationClient::new(port);
    match client.instantiate(Some(expected_storage_revision), request) {
        Ok(journey::ReplyV1::Accepted(outcome)) => match *outcome {
            journey::OutcomeV1::Admitted {
                storage_revision,
                receipt,
            } => emit(json!({
                "version": 1,
                "peer": "moira",
                "capability": INSTANTIATE_CAPABILITY,
                "complete": true,
                "status": "admitted",
                "storage_revision": storage_revision,
                "replayed": receipt.replayed,
                "journey": journey_json(&receipt.journey),
                "next_action": "observe the durable journey and wait for an owner-published progression command; admission alone starts no peer effect",
                "boundary": "human confirmation is not Aegis authority and durable admission does not imply a room, session, participation, provider call, or completed task",
            })),
            _ => failure(
                INSTANTIATE_CAPABILITY,
                "invalid_reply",
                "unexpected Moira outcome",
                None,
            ),
        },
        Ok(journey::ReplyV1::Refused(value)) => refusal(INSTANTIATE_CAPABILITY, value),
        Err(error) => transport_failure(INSTANTIATE_CAPABILITY, error),
    }
}

pub fn observe(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let socket = PathBuf::from(required(
        &mut args,
        "observe requires SOCKET OWNER_HEX SEQUENCE",
    )?);
    let owner = required_text(&mut args, "observe requires OWNER_HEX")?;
    let sequence = required_text(&mut args, "observe requires SEQUENCE")?.parse::<u64>()?;
    no_more(&mut args)?;
    let id = journey::JourneyIdV1 {
        owner: journey::OwnerIdV1(parse_hex(&owner)?),
        sequence,
    };
    let port = match connect(&socket) {
        Ok(port) => port,
        Err(error) => return transport_failure(OBSERVE_CAPABILITY, error),
    };
    let mut client = TaskAgentJourneyReadOnlyClient::new(port);
    let status = match client.status() {
        Ok(journey::ReplyV1::Accepted(outcome)) => match *outcome {
            journey::OutcomeV1::Status(status) => status,
            _ => {
                return failure(
                    OBSERVE_CAPABILITY,
                    "invalid_reply",
                    "unexpected status outcome",
                    None,
                );
            }
        },
        Ok(journey::ReplyV1::Refused(value)) => return refusal(OBSERVE_CAPABILITY, value),
        Err(error) => return transport_failure(OBSERVE_CAPABILITY, error),
    };
    match client.journey(id) {
        Ok(journey::ReplyV1::Accepted(outcome)) => match *outcome {
            journey::OutcomeV1::Journey(Some(view)) => emit(json!({
                "version": 1,
                "peer": "moira",
                "capability": OBSERVE_CAPABILITY,
                "complete": !status.needs_reopen,
                "status": if status.needs_reopen { "fenced" } else { "current" },
                "owner_revision": status.owner_revision,
                "storage_revision": status.storage_revision,
                "journey": journey_json(&view),
                "next_action": if status.needs_reopen { "restart the Moira owner before any mutation" } else { "use only owner-published progression commands for the next effect" },
                "boundary": "this is Moira journey evidence; peer readiness and task disposition remain owner-native facts",
            })),
            journey::OutcomeV1::Journey(None) => failure(
                OBSERVE_CAPABILITY,
                "not_found",
                "journey is not present under this owner identity",
                Some(
                    json!({"owner_revision": status.owner_revision, "storage_revision": status.storage_revision}),
                ),
            ),
            _ => failure(
                OBSERVE_CAPABILITY,
                "invalid_reply",
                "unexpected journey outcome",
                None,
            ),
        },
        Ok(journey::ReplyV1::Refused(value)) => refusal(OBSERVE_CAPABILITY, value),
        Err(error) => transport_failure(OBSERVE_CAPABILITY, error),
    }
}

fn connect(path: &PathBuf) -> Result<TaskAgentJourneyUnixPort, TaskAgentJourneyClientErrorV1> {
    TaskAgentJourneyUnixPort::connect(path, TaskAgentJourneyClientLimitsV1::default())
}

fn preview_request(
    value: PreviewDocumentV1,
) -> Result<journey::PreviewRequestV1, Box<dyn std::error::Error>> {
    if value.version != 1 {
        return Err("preview document version must be 1".into());
    }
    let room_policy = match value.room_policy.as_str() {
        "direct" => journey::RoomPolicyV1::Direct,
        "shared-context" => journey::RoomPolicyV1::SharedContext,
        _ => return Err("room_policy must be direct or shared-context".into()),
    };
    Ok(journey::PreviewRequestV1 {
        operation: journey::PreviewOperationIdV1(parse_hex(&value.operation)?),
        template: journey::TemplateRefV1 {
            id: journey::TemplateIdV1(parse_hex(&value.template.id)?),
            version: value.template.version,
            digest: parse_hex(&value.template.digest)?,
        },
        task: journey::TaskAdmissionV1 {
            task: TaskRefV1 {
                external_task_key: parse_hex(&value.task.external_task_key)?,
                incarnation: value.task.incarnation,
            },
            chronos_revision: value.task.chronos_revision,
            evidence_digest: parse_hex(&value.task.evidence_digest)?,
        },
        profile: ProfileIdV1(parse_hex(&value.profile)?),
        room_policy,
        initiating_principal: journey::PrincipalIdV1(parse_hex(&value.initiating_principal)?),
        accountable_human: journey::PrincipalIdV1(parse_hex(&value.accountable_human)?),
        objective_digest: parse_hex(&value.objective_digest)?,
        acceptance_digest: parse_hex(&value.acceptance_digest)?,
        current_evidence_digest: parse_hex(&value.current_evidence_digest)?,
        requested_ttl: value.requested_ttl,
    })
}

fn instantiate_request(
    value: InstantiateDocumentV1,
) -> Result<journey::InstantiateRequestV1, Box<dyn std::error::Error>> {
    if value.version != 1 {
        return Err("instantiate document version must be 1".into());
    }
    let preview = journey::PreviewIdV1 {
        owner: journey::OwnerIdV1(parse_hex(&value.preview_owner)?),
        sequence: value.preview_sequence,
    };
    Ok(journey::InstantiateRequestV1 {
        operation: journey::InstantiateOperationIdV1(parse_hex(&value.operation)?),
        preview,
        expected_owner_revision: value.expected_owner_revision,
        confirmation: journey::HumanConfirmationV1 {
            preview,
            accountable_human: journey::PrincipalIdV1(parse_hex(&value.accountable_human)?),
            consequence_digest: parse_hex(&value.consequence_digest)?,
        },
    })
}

fn confirmation(preview: &journey::PreviewViewV1) -> String {
    format!(
        "INSTANTIATE MOIRA TASK AGENT PREVIEW {}:{} AT OWNER REVISION {} FOR ACCOUNTABLE HUMAN {} WITH CONSEQUENCE {}",
        hex(&preview.id.owner.0),
        preview.id.sequence,
        preview.owner_revision,
        hex(&preview.request.accountable_human.0),
        hex(&preview.consequence_digest),
    )
}

fn instantiate_confirmation(request: &journey::InstantiateRequestV1) -> String {
    format!(
        "INSTANTIATE MOIRA TASK AGENT PREVIEW {}:{} AT OWNER REVISION {} FOR ACCOUNTABLE HUMAN {} WITH CONSEQUENCE {}",
        hex(&request.preview.owner.0),
        request.preview.sequence,
        request.expected_owner_revision,
        hex(&request.confirmation.accountable_human.0),
        hex(&request.confirmation.consequence_digest),
    )
}

fn template_json(value: journey::TemplateV1) -> Value {
    json!({
        "id": hex(&value.id.0), "version": value.version, "digest": hex(&value.digest),
        "work_mode": format!("{:?}", value.work_mode), "profile": hex(&value.profile.0),
        "room_policy": format!("{:?}", value.room_policy),
        "budget": {"max_context_bytes": value.budget.max_context_bytes, "max_output_tokens": value.budget.max_output_tokens, "max_elapsed_millis": value.budget.max_elapsed_millis},
        "grounding_policy_digest": hex(&value.grounding_policy_digest),
        "authority_policy_digest": hex(&value.authority_policy_digest),
        "consequence_digest": hex(&value.consequence_digest), "retired": value.retired,
    })
}

fn preview_json(value: &journey::PreviewViewV1) -> Value {
    json!({
        "owner": hex(&value.id.owner.0), "sequence": value.id.sequence,
        "owner_revision": value.owner_revision, "observed_at": value.observed_at,
        "expires_at": value.expires_at,
        "template": {
            "id": hex(&value.request.template.id.0),
            "version": value.request.template.version,
            "digest": hex(&value.request.template.digest),
        },
        "task": {
            "external_task_key": hex(&value.request.task.task.external_task_key),
            "incarnation": value.request.task.task.incarnation,
            "chronos_revision": value.request.task.chronos_revision,
            "evidence_digest": hex(&value.request.task.evidence_digest),
        },
        "profile": hex(&value.request.profile.0),
        "room_policy": format!("{:?}", value.request.room_policy),
        "initiating_principal": hex(&value.request.initiating_principal.0),
        "accountable_human": hex(&value.request.accountable_human.0),
        "objective_digest": hex(&value.request.objective_digest),
        "acceptance_digest": hex(&value.request.acceptance_digest),
        "current_evidence_digest": hex(&value.request.current_evidence_digest),
        "preview_operation": hex(&value.request.operation.0),
        "work_mode": format!("{:?}", value.work_mode), "budget": {
            "max_context_bytes": value.budget.max_context_bytes,
            "max_output_tokens": value.budget.max_output_tokens,
            "max_elapsed_millis": value.budget.max_elapsed_millis,
        },
        "grounding_policy_digest": hex(&value.grounding_policy_digest),
        "authority_policy_digest": hex(&value.authority_policy_digest),
        "consequence_digest": hex(&value.consequence_digest),
    })
}

fn journey_json(value: &journey::JourneyViewV1) -> Value {
    let chronos_attempt = value.task_attempt.map(task_attempt_json);
    let (activation_status, activation_boundary) = if chronos_attempt.is_some() {
        (
            "task_attempt_ready",
            "Moira durably retained one exact Chronos claim; no Aegis delegation, participation, Daimon session, provider call, or completed-work receipt is implied",
        )
    } else {
        (
            "not_published",
            "admission retains intent and human confirmation but starts no peer effect",
        )
    };
    json!({
        "owner": hex(&value.id.owner.0), "sequence": value.id.sequence,
        "workflow_owner": hex(&value.workflow.owner.0), "workflow_sequence": value.workflow.sequence,
        "revision": value.revision, "phase": format!("{:?}", value.phase),
        "confirmed_at": value.confirmed_at,
        "admission": {
            "operation": hex(&value.admission.operation.0),
            "expected_owner_revision": value.admission.expected_owner_revision,
            "confirmed_preview_owner": hex(&value.admission.confirmation.preview.owner.0),
            "confirmed_preview_sequence": value.admission.confirmation.preview.sequence,
            "accountable_human": hex(&value.admission.confirmation.accountable_human.0),
            "consequence_digest": hex(&value.admission.confirmation.consequence_digest),
        },
        "preview": preview_json(&value.preview),
        "task_attempt": chronos_attempt.clone(),
        "activation": {
            "status": activation_status,
            "aegis_delegation": null,
            "chronos_attempt": chronos_attempt,
            "participation": null,
            "daimon_session": null,
            "expires_at": null,
            "completed_work_receipt": null,
            "boundary": activation_boundary,
        },
    })
}

fn task_attempt_json(value: journey::TaskAttemptBindingV1) -> Value {
    json!({
        "journey_owner": hex(&value.journey.owner.0),
        "journey_sequence": value.journey.sequence,
        "expected_journey_revision": value.expected_journey_revision,
        "external_task_key": hex(&value.task.external_task_key),
        "incarnation": value.task.incarnation,
        "attempt_number": value.attempt_number,
        "chronos_revision": value.chronos_revision,
        "boundary": "exact retained claim evidence only; this is not a lease bearer, Aegis delegation, session, provider invocation, or completion receipt",
    })
}

fn refusal(capability: &str, value: journey::RefusalV1) -> Result<(), Box<dyn std::error::Error>> {
    failure(
        capability,
        "refused",
        "Moira refused the owner command",
        Some(json!({"refusal": format!("{value:?}"), "next_action": refusal_next_action(value)})),
    )
}

fn refusal_next_action(value: journey::RefusalV1) -> &'static str {
    use journey::RefusalV1::*;
    match value {
        StorageRevisionConflict | OwnerRevisionConflict => {
            "re-observe owner status and create a fresh preview or request"
        }
        PreviewExpired | PreviewNotFound | InvalidPreview => {
            "create and review a fresh owner preview"
        }
        InvalidConfirmation => {
            "use the exact human, consequence, preview, and owner revision from the current preview"
        }
        NotAuthorized => {
            "obtain the configured owner access; human confirmation does not grant authority"
        }
        InvalidCheckpoint | Storage => "stop mutation and recover or restart the Moira owner",
        _ => {
            "inspect the current catalogue and owner status, correct the bounded request, and retry with a new operation identity if its meaning changed"
        }
    }
}

fn transport_failure(
    capability: &str,
    value: TaskAgentJourneyClientErrorV1,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = match value {
        TaskAgentJourneyClientErrorV1::NotSent(_) => "not_sent",
        TaskAgentJourneyClientErrorV1::OutcomeUnknown(_) => "outcome_unknown",
        TaskAgentJourneyClientErrorV1::InvalidReady
        | TaskAgentJourneyClientErrorV1::InvalidReply => "invalid_reply",
        TaskAgentJourneyClientErrorV1::InvalidLimits
        | TaskAgentJourneyClientErrorV1::RequestLimit => "client_refused",
    };
    let next = if status == "outcome_unknown" {
        "do not retry blindly; reconnect and observe/replay the exact operation identity"
    } else {
        "verify the selected Moira owner endpoint and bounded client contract"
    };
    failure(
        capability,
        status,
        "Moira task-agent transport did not return a definitive accepted result",
        Some(json!({"detail": format!("{value:?}"), "next_action": next})),
    )
}

fn failure(
    capability: &str,
    status: &str,
    message: &str,
    detail: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    emit(
        json!({"version": 1, "peer": "moira", "capability": capability, "complete": false, "status": status, "detail": detail}),
    )?;
    Err(std::io::Error::other(message).into())
}

fn emit(value: Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn read_document<T: for<'de> Deserialize<'de>>(
    path: &PathBuf,
) -> Result<T, Box<dyn std::error::Error>> {
    if std::fs::metadata(path)?.len() > 64 * 1024 {
        return Err("task-agent request document exceeds 64 KiB".into());
    }
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn parse_hex<const N: usize>(value: &str) -> Result<[u8; N], Box<dyn std::error::Error>> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("identifier must be {} hexadecimal digits", N * 2).into());
    }
    let mut result = [0; N];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn one_socket(
    args: &mut impl Iterator<Item = OsString>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = PathBuf::from(required(args, "catalogue requires SOCKET")?);
    no_more(args)?;
    Ok(path)
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| message.into())
}

fn required_text(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<String, Box<dyn std::error::Error>> {
    required(args, message)?
        .into_string()
        .map_err(|_| message.into())
}

fn no_more(args: &mut impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        Err("unexpected task-agent command argument".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_binds_owner_revision_human_and_consequence() {
        let preview = journey::PreviewViewV1 {
            id: journey::PreviewIdV1 {
                owner: journey::OwnerIdV1([1; 16]),
                sequence: 2,
            },
            owner_revision: 3,
            request: journey::PreviewRequestV1 {
                operation: journey::PreviewOperationIdV1([4; 16]),
                template: journey::TemplateRefV1 {
                    id: journey::TemplateIdV1([5; 16]),
                    version: 1,
                    digest: [6; 32],
                },
                task: journey::TaskAdmissionV1 {
                    task: TaskRefV1 {
                        external_task_key: [7; 16],
                        incarnation: 1,
                    },
                    chronos_revision: 8,
                    evidence_digest: [9; 32],
                },
                profile: ProfileIdV1([10; 16]),
                room_policy: journey::RoomPolicyV1::SharedContext,
                initiating_principal: journey::PrincipalIdV1([11; 16]),
                accountable_human: journey::PrincipalIdV1([12; 16]),
                objective_digest: [13; 32],
                acceptance_digest: [14; 32],
                current_evidence_digest: [15; 32],
                requested_ttl: 30,
            },
            observed_at: 100,
            expires_at: 130,
            work_mode: journey::WorkModeV1::Collaborator,
            budget: journey::BudgetV1 {
                max_context_bytes: 1,
                max_output_tokens: 2,
                max_elapsed_millis: 3,
            },
            grounding_policy_digest: [16; 32],
            authority_policy_digest: [17; 32],
            consequence_digest: [18; 32],
        };
        let text = confirmation(&preview);
        assert!(text.contains("01010101010101010101010101010101:2"));
        assert!(text.contains("OWNER REVISION 3"));
        assert!(text.contains("0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c"));
        assert!(text.ends_with(&"12".repeat(32)));

        let id = journey::JourneyIdV1 {
            owner: preview.id.owner,
            sequence: 0,
        };
        let projected = journey_json(&journey::JourneyViewV1 {
            id,
            workflow: journey::WorkflowRefV1 {
                owner: id.owner,
                sequence: id.sequence,
            },
            revision: 1,
            phase: journey::PhaseV1::TaskAttemptReady,
            admission: journey::InstantiateRequestV1 {
                operation: journey::InstantiateOperationIdV1([19; 16]),
                preview: preview.id,
                expected_owner_revision: preview.owner_revision,
                confirmation: journey::HumanConfirmationV1 {
                    preview: preview.id,
                    accountable_human: preview.request.accountable_human,
                    consequence_digest: preview.consequence_digest,
                },
            },
            confirmed_at: 101,
            preview,
            task_attempt: Some(journey::TaskAttemptBindingV1 {
                journey: id,
                expected_journey_revision: 0,
                task: preview.request.task.task,
                attempt_number: 2,
                chronos_revision: 9,
            }),
        });
        assert_eq!(projected["phase"], "TaskAttemptReady");
        assert_eq!(projected["activation"]["status"], "task_attempt_ready");
        assert_eq!(projected["task_attempt"]["attempt_number"], 2);
        assert_eq!(
            projected["activation"]["chronos_attempt"],
            projected["task_attempt"]
        );
        assert!(projected["activation"]["aegis_delegation"].is_null());
        assert!(projected["activation"]["daimon_session"].is_null());
    }

    #[test]
    fn fixed_width_hex_refuses_ambiguous_identity() {
        assert!(parse_hex::<16>("00").is_err());
        assert_eq!(parse_hex::<16>(&"ab".repeat(16)).unwrap(), [0xab; 16]);
    }
}
