//! Exact, bounded Agora observations for independently admitted agent tools.

use std::{collections::BTreeSet, ffi::OsString, path::PathBuf};

use agora_client::v2::{ClientErrorV2, ClientV2};
use agora_local_client::{LocalClientLimits, LocalServicePort};
use agora_protocol::v2::{
    AdmissionPolicyV2, CommandV2, OutcomeV2, PageCursorV2, ReplyV2, RoleV2, RoomSummaryV2,
};
use serde_json::{Value, json};

pub const ROOM_CAPABILITY: &str = "rooms.read.v2";
pub const MESSAGE_CAPABILITY: &str = "messages.read.v2";
const ROOM_PAGE_SIZE: u32 = 64;
const MAX_ROOM_PAGES: usize = 8;
const MAX_MESSAGE_LIMIT: u32 = 64;

pub fn room(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage =
        "Agora room observation requires SOCKET PARTICIPANT <conversation|composition> ROOM_ID";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let participant = required_text(&mut args, usage)?;
    let transport = required_text(&mut args, usage)?;
    let room_id = required_text(&mut args, usage)?;
    no_more(&mut args, usage)?;
    let mut client = connect(socket, &participant, &transport, ROOM_CAPABILITY)?;
    let mut cursor = None;
    let mut seen = BTreeSet::new();
    let mut revision = None;
    for _ in 0..MAX_ROOM_PAGES {
        let reply = match client.execute(CommandV2::Rooms {
            cursor,
            limit: ROOM_PAGE_SIZE,
        }) {
            Ok(reply) => reply,
            Err(error) => return client_failure(ROOM_CAPABILITY, error),
        };
        let page = match reply {
            ReplyV2::Accepted(OutcomeV2::Rooms(page)) => page,
            ReplyV2::Refused(refusal) => {
                return failure(
                    ROOM_CAPABILITY,
                    "refused",
                    &format!("Agora refused room observation: {refusal:?}"),
                    "verify participant membership and the exact owner-selected room",
                );
            }
            ReplyV2::Accepted(_) => return invalid_reply(ROOM_CAPABILITY),
        };
        if revision.is_some_and(|expected| expected != page.revision) {
            return failure(
                ROOM_CAPABILITY,
                "owner_changed",
                "Agora room catalogue revision changed during exact selection",
                "retry the observation from the first page",
            );
        }
        revision = Some(page.revision);
        if let Some(summary) = page.rooms.iter().find(|room| room.room_id == room_id) {
            return emit_success(
                ROOM_CAPABILITY,
                json!({
                    "revision": page.revision,
                    "room": room_projection(summary),
                }),
                "one exact room visible to the deployment-fixed Agora participant; no other room, message content, membership list, mutation, moderation, task, or execution authority is disclosed",
            );
        }
        let Some(next) = page.next else {
            return failure(
                ROOM_CAPABILITY,
                "not_found",
                "Agora did not disclose the exact room to this participant",
                "use an exact current room identity visible to the deployment-fixed participant; do not substitute another room",
            );
        };
        if !seen.insert(next.0) {
            return invalid_reply(ROOM_CAPABILITY);
        }
        cursor = Some(next);
    }
    failure(
        ROOM_CAPABILITY,
        "response_limit",
        "Agora exact-room selection exceeded the bounded eight-page scan",
        "have the operator narrow or partition the participant's room catalogue; no partial catalogue is emitted",
    )
}

pub fn messages(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Agora message observation requires SOCKET PARTICIPANT <conversation|composition> ROOM_ID CURSOR_OR_- LIMIT";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let participant = required_text(&mut args, usage)?;
    let transport = required_text(&mut args, usage)?;
    let room_id = required_text(&mut args, usage)?;
    let cursor_text = required_text(&mut args, usage)?;
    let cursor = if cursor_text == "-" {
        None
    } else {
        match cursor_text.parse::<u64>() {
            Ok(cursor) => Some(PageCursorV2(cursor)),
            Err(_) => {
                return failure(
                    MESSAGE_CAPABILITY,
                    "configuration_refused",
                    "Agora message cursor must be '-' or one unsigned opaque cursor value",
                    "supply the exact next_cursor from a prior owner response, or '-' for the first page",
                );
            }
        }
    };
    let limit_text = required_text(&mut args, usage)?;
    let limit = match limit_text.parse::<u32>() {
        Ok(limit) if (1..=MAX_MESSAGE_LIMIT).contains(&limit) => limit,
        _ => {
            return failure(
                MESSAGE_CAPABILITY,
                "configuration_refused",
                "Agora message limit must be 1..=64",
                "select a bounded page size from 1 through 64",
            );
        }
    };
    no_more(&mut args, usage)?;
    let mut client = connect(socket, &participant, &transport, MESSAGE_CAPABILITY)?;
    let reply = match client.execute(CommandV2::Messages {
        room_id: room_id.clone(),
        cursor,
        limit,
    }) {
        Ok(reply) => reply,
        Err(error) => return client_failure(MESSAGE_CAPABILITY, error),
    };
    match reply {
        ReplyV2::Accepted(OutcomeV2::Messages(page)) if page.room_id == room_id => emit_success(
            MESSAGE_CAPABILITY,
            json!({
                "room": room_id,
                "revision": page.revision,
                "oldest_retained_sequence": page.oldest_retained_sequence,
                "next_message_sequence": page.next_message_sequence,
                "next_cursor": page.next.map(|cursor| cursor.0),
                "count": page.messages.len(),
                "messages": page.messages.iter().map(|message| json!({
                    "sequence": message.sequence,
                    "author": message.author.as_str(),
                    "logical_time": message.logical_time,
                    "body": message.body.as_str(),
                    "addressed_to": &message.addressed_to,
                })).collect::<Vec<_>>(),
            }),
            "one bounded message page from one exact room visible to the deployment-fixed Agora participant; body content is intentionally disclosed, while membership, other rooms, mutation, moderation, task, and execution authority remain unavailable",
        ),
        ReplyV2::Accepted(OutcomeV2::Messages(_)) => failure(
            MESSAGE_CAPABILITY,
            "invalid_reply",
            "Agora returned a different room identity",
            "stop agent admission and inspect owner protocol compatibility",
        ),
        ReplyV2::Refused(refusal) => failure(
            MESSAGE_CAPABILITY,
            "refused",
            &format!("Agora refused message observation: {refusal:?}"),
            "verify participant membership, the exact room identity, cursor, and retention frontier",
        ),
        ReplyV2::Accepted(_) => invalid_reply(MESSAGE_CAPABILITY),
    }
}

fn connect(
    socket: PathBuf,
    participant: &str,
    transport: &str,
    capability: &'static str,
) -> Result<ClientV2<LocalServicePort>, Box<dyn std::error::Error>> {
    let result = match transport {
        "conversation" => {
            LocalServicePort::connect(&socket, participant, LocalClientLimits::default())
        }
        "composition" => LocalServicePort::connect_composition(
            &socket,
            participant,
            LocalClientLimits::default(),
        ),
        _ => {
            failure(
                capability,
                "configuration_refused",
                "Agora transport must be conversation or composition",
                "use the deployment-selected conversation or composition transport profile",
            )?;
            unreachable!("failure always returns an error")
        }
    };
    match result {
        Ok((client, port)) => Ok(ClientV2::new(client, port)),
        Err(error) => {
            failure(
                capability,
                "owner_unavailable",
                &format!("Agora connection failed: {error:?}"),
                "verify the deployment-fixed Agora owner socket, participant, and transport profile",
            )?;
            unreachable!("failure always returns an error")
        }
    }
}

fn room_projection(room: &RoomSummaryV2) -> Value {
    json!({
        "id": room.room_id.as_str(),
        "name": room.name.as_str(),
        "admission": match room.admission {
            AdmissionPolicyV2::Open => "open",
            AdmissionPolicyV2::InviteOnly => "invite_only",
        },
        "observer_role": match room.observer_role {
            RoleV2::Owner => "owner",
            RoleV2::Moderator => "moderator",
            RoleV2::Member => "member",
        },
        "member_count": room.member_count,
        "retained_message_count": room.retained_message_count,
        "next_message_sequence": room.next_message_sequence,
    })
}

fn client_failure(
    capability: &'static str,
    error: ClientErrorV2<agora_local_client::LocalAccessError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = match error {
        ClientErrorV2::NotSent(_) => "not_sent",
        ClientErrorV2::OutcomeUnknown(_) | ClientErrorV2::UnresolvedOutcome => "outcome_unknown",
        ClientErrorV2::RequestIdExhausted => "request_id_exhausted",
    };
    failure(
        capability,
        status,
        &format!("Agora observation failed: {error:?}"),
        "retry only a read-only observation against the same deployment-fixed owner and exact selector",
    )
}

fn invalid_reply(capability: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    failure(
        capability,
        "invalid_reply",
        "Agora returned an outcome outside the selected observation capability",
        "stop agent admission and inspect owner protocol compatibility",
    )
}

fn emit_success(
    capability: &'static str,
    evidence: Value,
    boundary: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "agora",
            "capability": capability,
            "complete": true,
            "status": "current",
            "evidence": evidence,
            "boundary": boundary,
        }))?
    );
    Ok(())
}

fn failure(
    capability: &'static str,
    status: &'static str,
    detail: &str,
    next_action: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "agora",
            "capability": capability,
            "complete": false,
            "status": status,
            "detail": detail,
            "next_action": next_action,
            "boundary": "no aggregate room catalogue, cross-room substitution, mutation, moderation, task, shell, filesystem, provider, or execution fallback is permitted",
        }))?
    );
    Err(std::io::Error::other("Agora observation unavailable").into())
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| usage.into())
}

fn required_text(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<String, Box<dyn std::error::Error>> {
    required(args, usage)?
        .into_string()
        .map_err(|_| usage.into())
}

fn no_more(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        Err(usage.into())
    } else {
        Ok(())
    }
}
