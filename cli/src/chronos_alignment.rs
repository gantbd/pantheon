//! Exact Chronos product-alignment observation shared by headless and agents.

use chronos_client::AlignmentWireClientV1;
use chronos_protocol::alignment_v1::{
    Command, Limits, Observation as AlignmentObservation, Outcome, RefusalCode, Request,
};
use serde_json::json;
use std::{ffi::OsString, path::PathBuf};

pub const CAPABILITY: &str = "product-alignment.observe.v1";

const LIMITS: Limits = Limits {
    max_field_bytes: 256,
    max_evidence: 128,
    max_next_tasks: 64,
};

pub enum Observation {
    Current(Box<AlignmentObservation>),
    Failure {
        status: &'static str,
        detail: &'static str,
    },
}

pub fn observe(directory: PathBuf, scope: String) -> Observation {
    let mut client = chronos_local::alignment::AlignmentLocalClient::new(directory, LIMITS);
    match client.request_alignment_v1(Request {
        version: 1,
        command: Command::Observe {
            chronos_scope: scope,
        },
    }) {
        Ok(response) => match response.outcome {
            Outcome::Observed { observation } => Observation::Current(observation),
            Outcome::Refused { code } => Observation::Failure {
                status: match code {
                    RefusalCode::InvalidRequest => "invalid_request",
                    RefusalCode::UnsupportedVersion => "unsupported_version",
                    RefusalCode::Unauthorized => "unauthorized",
                    RefusalCode::Unavailable => "unavailable",
                    RefusalCode::Limit => "limit",
                    RefusalCode::Capacity => "capacity",
                },
                detail: "Chronos alignment owner refused exact observation",
            },
        },
        Err(error) => Observation::Failure {
            status: match error {
                chronos_local::CallError::NotSent => "not_sent",
                chronos_local::CallError::OutcomeUnknown => "outcome_unknown",
                chronos_local::CallError::InvalidResponse => "invalid_reply",
            },
            detail: "Chronos alignment endpoint did not return a definitive observation",
        },
    }
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "pantheon chronos-alignment-observe DIRECTORY SCOPE";
    let directory = PathBuf::from(args.next().ok_or(usage)?);
    let scope = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "Chronos alignment scope must be UTF-8")?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let (status, complete, owner, next_action) = match observe(directory, scope) {
        Observation::Current(observation) => (
            "current",
            true,
            json!({"observation": observation}),
            "use the current owner frontier as evidence; route material judgment through attributed proposal and human-gate contracts",
        ),
        Observation::Failure { status, detail } => (
            status,
            false,
            json!({"detail": detail}),
            "verify the deployment-fixed Chronos alignment owner, exact scope, and current upstream owners; do not fall back to aggregate status or a retained proposal",
        ),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "chronos-alignment",
            "capability": CAPABILITY,
            "complete": complete,
            "status": status,
            "owner": owner,
            "next_action": next_action,
            "boundary": "one deployment-pinned scope returns Chronos's content-free current goal, task, cohort, budget, evidence, dissent, human-gate, proposal and currentness facts; it grants no owner configuration, Vellum credential, proposal creation or application, task claim or mutation, reprioritization, acceptance, sign-off, role activation, or agent execution authority",
        }))?
    );
    if complete {
        Ok(())
    } else {
        Err(std::io::Error::other("Chronos alignment observation unavailable").into())
    }
}
