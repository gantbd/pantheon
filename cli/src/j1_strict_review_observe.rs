//! Operational, read-only acquisition of strict J1 review evidence.

use crate::{
    agora_observe::{AgoraConnectionErrorV1, connect_client},
    j1_agent_result::observe_owner_bound_agent_result,
    j1_review_link::{observe_j1_review_link, parse_hex_16},
    j1_strict_review::qualify_strict_j1_review_evidence,
};
use agora_client::v2::ClientV2;
use agora_local_client::LocalServicePort;
use agora_protocol::v2::{CommandV2, IdentifiedMessagePageV1, OutcomeV2, PageCursorV2, ReplyV2};
use moira_client::{
    AgentParticipationReadOnlyClient,
    unix::{AgentParticipationUnixPort, ParticipationClientLimitsV1},
};
use moira_protocol::{
    agent_participation as participation,
    task_agent_journey::{JourneyIdV1, OwnerIdV1},
};
use serde_json::json;
use std::{collections::BTreeSet, ffi::OsString, path::PathBuf};

pub const CAPABILITY: &str = "j1.strict-review-evidence.observe.v1";
const IDENTIFIED_MESSAGE_PAGE_SIZE: u32 = 64;
const MAX_IDENTIFIED_MESSAGE_PAGES: usize = 8;
const MAX_IDENTIFIED_MESSAGES: usize =
    IDENTIFIED_MESSAGE_PAGE_SIZE as usize * MAX_IDENTIFIED_MESSAGE_PAGES;

pub fn run(args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    match execute(args) {
        Ok(()) => Ok(()),
        Err(error) => failure(&error.to_string()),
    }
}

fn execute(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "j1-strict-review-observe requires MOIRA_JOURNEY_SOCKET MOIRA_PARTICIPATION_SOCKET TASK_CONVERSATION_SOCKET CHRONOS_DIRECTORY DAIMON_SOCKET AGORA_SOCKET AGORA_PARTICIPANT <conversation|composition> JOURNEY_OWNER_HEX JOURNEY_SEQUENCE";
    let moira_journey_socket = PathBuf::from(args.next().ok_or(usage)?);
    let moira_participation_socket = PathBuf::from(args.next().ok_or(usage)?);
    let task_conversation_socket = PathBuf::from(args.next().ok_or(usage)?);
    let chronos_directory = PathBuf::from(args.next().ok_or(usage)?);
    let daimon_socket = PathBuf::from(args.next().ok_or(usage)?);
    let agora_socket = PathBuf::from(args.next().ok_or(usage)?);
    let agora_participant = text_arg(&mut args, usage)?;
    let agora_transport = text_arg(&mut args, usage)?;
    let journey_owner = text_arg(&mut args, usage)?;
    let journey_sequence = text_arg(&mut args, usage)?.parse::<u64>()?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let journey = JourneyIdV1 {
        owner: OwnerIdV1(parse_hex_16(&journey_owner)?),
        sequence: journey_sequence,
    };

    let initial = observe_j1_review_link(
        &moira_journey_socket,
        task_conversation_socket.clone(),
        chronos_directory.clone(),
        journey,
    )
    .map_err(|error| format!("initial J1 review-link observation refused: {error}"))?;
    let participation_id = initial
        .evidence
        .actor_participation
        .participation
        .participation;
    let initial_participation =
        observe_terminal_participation(&moira_participation_socket, participation_id)?;
    let terminal = initial_participation
        .terminal
        .ok_or("Moira participation lost terminal evidence after qualification")?;
    let session = daimon_protocol::AgentSessionIdV1::new(initial_participation.binding.session.0);
    let turn = daimon_protocol::AgentTurnIdV1::new(terminal.turn.0);
    let result = observe_owner_bound_agent_result(&daimon_socket, session, turn)?;

    let publication = &initial.evidence.actor_participation.publication;
    let page = observe_identified_messages(
        agora_socket,
        &agora_participant,
        &agora_transport,
        &publication.room_id,
    )?;

    let final_participation =
        observe_terminal_participation(&moira_participation_socket, participation_id)?;
    let final_review = observe_j1_review_link(
        &moira_journey_socket,
        task_conversation_socket,
        chronos_directory,
        journey,
    )
    .map_err(|error| format!("final J1 review-link observation refused: {error}"))?;
    let evidence = qualify_strict_j1_review_evidence(
        &final_review.evidence,
        &final_participation,
        &result,
        &page,
    )
    .map_err(|refusal| format!("strict J1 review evidence refused: {refusal:?}"))?;
    let content = String::from_utf8(result.bytes.clone())
        .map_err(|_| "qualified J1 review content was not UTF-8")?;
    let publication = &final_review.evidence.actor_participation.publication;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "pantheon",
            "capability": CAPABILITY,
            "complete": true,
            "status": "strict_review_evidence_issued",
            "receipt_digest": format!("sha256:{}", hex(&evidence.receipt_digest)),
            "journey": {
                "owner": hex(&evidence.journey.owner.0),
                "sequence": evidence.journey.sequence,
            },
            "task": {
                "id": final_review.evidence.namespace.chronos_task_id,
                "incarnation": evidence.task_attempt.task_incarnation,
                "attempt": evidence.task_attempt.attempt_number,
                "chronos_revision": evidence.chronos_revision,
                "status": "ready_for_review",
            },
            "agent": {
                "participation": hex(&participation_id.0),
                "session": hex(&evidence.session.into_bytes()),
                "turn": hex(&evidence.turn.into_bytes()),
                "execution": hex(&evidence.execution.into_bytes()),
                "terminal_revision": result.terminal_revision,
                "output_digest": format!("sha256:{}", hex(&evidence.output_digest.into_bytes())),
            },
            "publication": {
                "room": publication.room_id,
                "message_id": hex(&evidence.message_id.into_bytes()),
                "message_sequence": evidence.message_sequence,
                "actor": publication.actor,
                "agora_revision": evidence.agora_revision,
            },
            "content_bytes": result.bytes.len(),
            "content": content,
            "next_action": "bind this exact receipt and challenged content into a separately governed UADD request for the named human acceptance owner",
            "boundary": "read-only exact J1 review evidence; no human disposition, acceptance, RevisionRequired, Deferred settlement, rejection effect, Chronos mutation, Done, rollout, or go-no-go authority",
        }))?
    );
    Ok(())
}

fn observe_terminal_participation(
    socket: &PathBuf,
    id: participation::ParticipationIdV1,
) -> Result<participation::ParticipationViewV1, Box<dyn std::error::Error>> {
    let port = AgentParticipationUnixPort::connect(socket, ParticipationClientLimitsV1::default())
        .map_err(|error| format!("Moira participation connection refused: {error:?}"))?;
    let mut client = AgentParticipationReadOnlyClient::new(port);
    let status = match client
        .status()
        .map_err(|error| format!("Moira participation status failed: {error:?}"))?
    {
        participation::ReplyV1::Accepted(outcome) => match *outcome {
            participation::OutcomeV1::Status(status) => status,
            _ => return Err("Moira returned a different participation status outcome".into()),
        },
        participation::ReplyV1::Refused(refusal) => {
            return Err(format!("Moira refused participation status: {refusal:?}").into());
        }
    };
    if status.needs_reopen {
        return Err("Moira participation owner requires recovery before observation".into());
    }
    let view = match client
        .participation(id)
        .map_err(|error| format!("Moira participation observation failed: {error:?}"))?
    {
        participation::ReplyV1::Accepted(outcome) => match *outcome {
            participation::OutcomeV1::Participation(Some(view)) => view,
            participation::OutcomeV1::Participation(None) => {
                return Err("Moira has no exact admitted participation".into());
            }
            _ => return Err("Moira returned a different participation outcome".into()),
        },
        participation::ReplyV1::Refused(refusal) => {
            return Err(format!("Moira refused participation observation: {refusal:?}").into());
        }
    };
    if view.phase != participation::PhaseV1::Terminal
        || view.active_turn.is_some()
        || view.pending.is_some()
        || view.terminal.is_none()
    {
        return Err("Moira participation is not a stable terminal observation".into());
    }
    Ok(view)
}

fn observe_identified_messages(
    socket: PathBuf,
    participant: &str,
    transport: &str,
    room_id: &str,
) -> Result<IdentifiedMessagePageV1, Box<dyn std::error::Error>> {
    let mut client =
        connect_client(socket, participant, transport).map_err(|error| match error {
            AgoraConnectionErrorV1::InvalidTransport => {
                "Agora transport must be conversation or composition".to_string()
            }
            AgoraConnectionErrorV1::Unavailable(detail) => detail,
        })?;
    let mut cursor = None;
    let mut seen = BTreeSet::new();
    let mut aggregate: Option<IdentifiedMessagePageV1> = None;
    for _ in 0..MAX_IDENTIFIED_MESSAGE_PAGES {
        let page = execute_identified_page(&mut client, room_id, cursor)?;
        let next = merge_identified_page(&mut aggregate, page)?;
        let Some(next) = next else {
            return aggregate.ok_or_else(|| "Agora returned no identified-message page".into());
        };
        if !seen.insert(next.0) {
            return Err("Agora repeated an identified-message cursor".into());
        }
        cursor = Some(next);
    }
    Err("Agora identified-message observation exceeded eight bounded pages".into())
}

fn merge_identified_page(
    aggregate: &mut Option<IdentifiedMessagePageV1>,
    page: IdentifiedMessagePageV1,
) -> Result<Option<PageCursorV2>, Box<dyn std::error::Error>> {
    let mut page_ids = BTreeSet::new();
    if page.oldest_retained_sequence > page.next_message_sequence
        || page
            .messages
            .windows(2)
            .any(|pair| pair[0].sequence >= pair[1].sequence)
        || page
            .messages
            .iter()
            .any(|message| !page_ids.insert(message.message_id))
        || page.messages.iter().any(|message| {
            message.sequence < page.oldest_retained_sequence
                || message.sequence >= page.next_message_sequence
        })
    {
        return Err("Agora returned an invalid identified-message interval".into());
    }
    if let Some(value) = aggregate.as_mut() {
        if value.revision != page.revision
            || value.room_id != page.room_id
            || value.oldest_retained_sequence != page.oldest_retained_sequence
            || value.next_message_sequence != page.next_message_sequence
        {
            return Err("Agora identified-message snapshot changed across pages".into());
        }
        if value.messages.len().saturating_add(page.messages.len()) > MAX_IDENTIFIED_MESSAGES {
            return Err("Agora identified-message observation exceeded its message bound".into());
        }
        if let (Some(previous), Some(next)) = (value.messages.last(), page.messages.first())
            && previous.sequence >= next.sequence
        {
            return Err("Agora identified-message pages overlap or regress".into());
        }
        if page.messages.iter().any(|message| {
            value
                .messages
                .iter()
                .any(|retained| retained.message_id == message.message_id)
        }) {
            return Err("Agora repeated an identified-message identity across pages".into());
        }
        value.messages.extend(page.messages);
        value.next = page.next;
    } else {
        if page.messages.len() > MAX_IDENTIFIED_MESSAGES {
            return Err("Agora identified-message observation exceeded its message bound".into());
        }
        *aggregate = Some(page);
    }
    Ok(aggregate.as_ref().and_then(|value| value.next))
}

fn execute_identified_page(
    client: &mut ClientV2<LocalServicePort>,
    room_id: &str,
    cursor: Option<PageCursorV2>,
) -> Result<IdentifiedMessagePageV1, Box<dyn std::error::Error>> {
    match client
        .execute(CommandV2::IdentifiedMessages {
            room_id: room_id.to_string(),
            cursor,
            limit: IDENTIFIED_MESSAGE_PAGE_SIZE,
        })
        .map_err(|error| format!("Agora identified-message observation failed: {error:?}"))?
    {
        ReplyV2::Accepted(OutcomeV2::IdentifiedMessages(page)) if page.room_id == room_id => {
            Ok(page)
        }
        ReplyV2::Accepted(OutcomeV2::IdentifiedMessages(_)) => {
            Err("Agora returned a different room identity".into())
        }
        ReplyV2::Refused(refusal) => {
            Err(format!("Agora refused identified-message observation: {refusal:?}").into())
        }
        ReplyV2::Accepted(_) => Err("Agora returned a different observation outcome".into()),
    }
}

fn text_arg(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<String, Box<dyn std::error::Error>> {
    args.next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| usage.into())
}

fn failure(detail: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "pantheon",
            "capability": CAPABILITY,
            "complete": false,
            "status": "evidence_refused",
            "detail": detail,
            "next_action": "inspect the named owner boundary and re-observe the same journey; do not substitute task, participation, session, turn, room, actor, message, or result identities",
            "boundary": "the unresolved join remains in the Maybe Machine; no partial evidence, human disposition, Chronos mutation, retry, acceptance, Done, or rollout is implied",
        }))?
    );
    Err(std::io::Error::other(detail).into())
}

fn hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use agora_protocol::v2::{IdentifiedMessageV1, MessageIdV1};

    fn page(revision: u64, sequence: u64, next: Option<PageCursorV2>) -> IdentifiedMessagePageV1 {
        IdentifiedMessagePageV1 {
            revision,
            room_id: "exact-room".into(),
            messages: vec![IdentifiedMessageV1 {
                message_id: MessageIdV1::new([sequence as u8; 16]),
                sequence,
                author: "exact-agent".into(),
                logical_time: sequence,
                body: format!("result-{sequence}"),
                addressed_to: vec![],
            }],
            oldest_retained_sequence: 1,
            next_message_sequence: 3,
            next,
        }
    }

    #[test]
    fn folds_only_one_revision_consistent_bounded_snapshot() {
        let mut aggregate = None;
        assert_eq!(
            merge_identified_page(&mut aggregate, page(7, 1, Some(PageCursorV2(9)))).unwrap(),
            Some(PageCursorV2(9))
        );
        assert_eq!(
            merge_identified_page(&mut aggregate, page(7, 2, None)).unwrap(),
            None
        );
        let aggregate = aggregate.unwrap();
        assert_eq!(aggregate.messages.len(), 2);
        assert_eq!(aggregate.revision, 7);
        assert!(aggregate.next.is_none());
    }

    #[test]
    fn refuses_cross_revision_or_cross_room_page_folding() {
        let mut aggregate = Some(page(7, 1, Some(PageCursorV2(9))));
        assert!(merge_identified_page(&mut aggregate, page(8, 2, None)).is_err());

        let mut aggregate = Some(page(7, 1, Some(PageCursorV2(9))));
        let mut other_room = page(7, 2, None);
        other_room.room_id = "substitute-room".into();
        assert!(merge_identified_page(&mut aggregate, other_room).is_err());
    }

    #[test]
    fn refuses_overlapping_sequences_and_repeated_identities() {
        let mut aggregate = Some(page(7, 1, Some(PageCursorV2(9))));
        assert!(merge_identified_page(&mut aggregate, page(7, 1, None)).is_err());

        let mut aggregate = Some(page(7, 1, Some(PageCursorV2(9))));
        let mut repeated_id = page(7, 2, None);
        repeated_id.messages[0].message_id = MessageIdV1::new([1; 16]);
        assert!(merge_identified_page(&mut aggregate, repeated_id).is_err());
    }
}
