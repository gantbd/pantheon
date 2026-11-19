//! Exact Moira stakeholder-ensemble observation shared by headless and agents.

use moira_client::DeliberationReadOnlyClient;
use moira_protocol::deliberation::{DeliberationIdV1, HumanGateMaterialRefV1};
use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

pub const CAPABILITY: &str = "moira.deliberation.observe.v1";

pub enum Observation {
    Current {
        registry_revision: u64,
        deliberation: Value,
        review: Option<Value>,
        pending_packet: Option<HumanGateMaterialRefV1>,
    },
    NotFound {
        registry_revision: u64,
    },
    Failure {
        status: &'static str,
        detail: String,
    },
}

pub fn observe(socket: PathBuf, id: DeliberationIdV1) -> Observation {
    let port = match moira_tui_local_client::LocalDeliberationPort::connect(
        socket,
        moira_tui_local_client::ClientLimits::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return Observation::Failure {
                status: "connect_refused",
                detail: format!("{error:?}"),
            };
        }
    };
    let mut client = DeliberationReadOnlyClient::new(port);
    match moira_tui::observe_deliberation(&mut client, id) {
        moira_tui::DeliberationObservation::Current {
            registry_revision,
            view,
            review,
        } => {
            let pending_packet = view
                .human_gate_request
                .filter(|_| view.human_gate_response.is_none())
                .map(|request| request.decision_packet);
            Observation::Current {
                registry_revision,
                deliberation: moira_tui::deliberation_view_json(*view),
                review: review.map(|value| moira_tui::review_view_json(*value)),
                pending_packet,
            }
        }
        moira_tui::DeliberationObservation::NotFound { registry_revision } => {
            Observation::NotFound { registry_revision }
        }
        moira_tui::DeliberationObservation::Stale => Observation::Failure {
            status: "stale",
            detail: "registry revision changed during observation".into(),
        },
        moira_tui::DeliberationObservation::NeedsReopen => Observation::Failure {
            status: "needs_reopen",
            detail: "owner store requires reopen".into(),
        },
        moira_tui::DeliberationObservation::Refused(reason) => Observation::Failure {
            status: "refused",
            detail: reason,
        },
        moira_tui::DeliberationObservation::InvalidReply => Observation::Failure {
            status: "invalid_reply",
            detail: "unexpected owner reply".into(),
        },
        moira_tui::DeliberationObservation::Transport => Observation::Failure {
            status: "transport_unavailable",
            detail: "owner transport unavailable".into(),
        },
    }
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "pantheon moira-deliberation-observe SOCKET DELIBERATION_HEX";
    let socket = PathBuf::from(args.next().ok_or(usage)?);
    let id_text = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "Moira deliberation identity must be UTF-8")?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let id = DeliberationIdV1(parse_hex_16(&id_text)?);
    let (status, complete, owner) = match observe(socket, id) {
        Observation::Current {
            registry_revision,
            deliberation,
            review,
            ..
        } => (
            "current",
            true,
            json!({
                "registry_revision": registry_revision,
                "deliberation": deliberation,
                "review": review,
            }),
        ),
        Observation::NotFound { registry_revision } => (
            "not_found",
            false,
            json!({"registry_revision": registry_revision}),
        ),
        Observation::Failure { status, detail } => (status, false, json!({"detail": detail})),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "moira",
            "capability": CAPABILITY,
            "complete": complete,
            "status": status,
            "deliberation": id_text,
            "owner": owner,
            "boundary": "one exact owner deliberation and stable review disclose stakeholder standing, coverage/equity, grounding, evidence, dissent, correlation and pending or settled human-gate facts; this observation grants no ensemble fitness inference, member independence, response authority, role activation, task mutation, agent execution, approval, sign-off, or rollout authority",
            "next_action": if complete {
                "use the owner facts as evidence; route any material unresolved decision to the named human gate"
            } else if status == "not_found" {
                "verify the exact owner-issued deliberation identity; do not substitute another ensemble"
            } else {
                "reopen or restore the deployment-selected Moira owner and repeat the exact read"
            },
        }))?
    );
    if complete {
        Ok(())
    } else {
        Err(std::io::Error::other("Moira deliberation observation unavailable").into())
    }
}

fn parse_hex_16(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "Moira deliberation identity must be exactly 32 lowercase hex characters".into(),
        );
    }
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_lowercase_deliberation_identity_is_required() {
        assert_eq!(
            parse_hex_16("00112233445566778899aabbccddeeff").unwrap()[15],
            255
        );
        assert!(parse_hex_16("00112233445566778899AABBCCDDEEFF").is_err());
        assert!(parse_hex_16("00").is_err());
        assert!(parse_hex_16("gg112233445566778899aabbccddeeff").is_err());
    }
}
