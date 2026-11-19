//! Shared behavior for one exact Moira participation-worker step.
//!
//! The worker executable remains Moira-owned. Pantheon launches it only from a
//! validated deployment profile and accepts one bounded receipt naming the
//! exact configured participation. A process started without a definitive
//! receipt is fenced; this facade never retries it.

use crate::profile::WorkspaceProfile;
use moira_protocol::agent_participation::ParticipationIdV1;
use serde::Deserialize;
use serde_json::json;
use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const CAPABILITY: &str = "agent.worker.step.v1";

#[derive(Clone, Debug)]
pub struct MoiraAgentWorkerConfig {
    pub executable: PathBuf,
    pub config: PathBuf,
    pub participation: ParticipationIdV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Idle,
    Observed,
    FencedUnknown,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    pub(crate) outcome: Outcome,
    participation: String,
    pub(crate) participation_revision: u64,
}

#[derive(Debug)]
pub(crate) enum ProcessError {
    NotStarted(String),
    OutcomeUnknown(String),
}

pub(crate) fn execute(config: &MoiraAgentWorkerConfig) -> Result<Receipt, ProcessError> {
    let child = Command::new(&config.executable)
        .arg(&config.config)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            ProcessError::NotStarted(format!("owner worker could not start: {error}"))
        })?;
    let output = child.wait_with_output().map_err(|error| {
        ProcessError::OutcomeUnknown(format!("owner worker outcome unknown: {error}"))
    })?;
    if output.stdout.len() > 4_096 || output.stderr.len() > 4_096 {
        return Err(ProcessError::OutcomeUnknown(
            "owner worker output exceeded 4096 bytes".into(),
        ));
    }
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(ProcessError::OutcomeUnknown(format!(
            "owner worker exited {}: {}",
            output.status,
            detail.trim()
        )));
    }
    let receipt: Receipt = serde_json::from_slice(&output.stdout).map_err(|_| {
        ProcessError::OutcomeUnknown("owner worker returned an invalid receipt".into())
    })?;
    if receipt.participation != hex(&config.participation.0) {
        return Err(ProcessError::OutcomeUnknown(
            "owner worker receipt named a different participation".into(),
        ));
    }
    Ok(receipt)
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let profile_path = PathBuf::from(
        args.next()
            .ok_or("worker step requires WORKSPACE_PROFILE CONFIRMATION")?,
    );
    let supplied = args
        .next()
        .ok_or("worker step requires CONFIRMATION")?
        .into_string()
        .map_err(|_| "worker confirmation must be UTF-8")?;
    if args.next().is_some() {
        return Err("unexpected worker-step argument".into());
    }
    let profile = WorkspaceProfile::load(&profile_path)?;
    let moira = profile
        .moira
        .ok_or("workspace profile does not configure Moira")?;
    let worker = moira
        .agent_worker
        .ok_or("workspace profile does not configure the Moira owner worker")?;
    let participation = ParticipationIdV1(parse_hex(
        moira
            .participation
            .as_deref()
            .ok_or("workspace profile does not select a Moira participation")?,
    )?);
    let required = format!("RUN MOIRA PARTICIPATION WORKER {}", hex(&participation.0));
    if supplied != required {
        emit(json!({
            "version": 1,
            "peer": "moira",
            "capability": CAPABILITY,
            "complete": false,
            "status": "confirmation_refused",
            "required_confirmation": required,
            "next_action": "inspect the exact prepared participation and provide the displayed confirmation",
            "boundary": "confirmation starts no worker and grants no authority",
        }))?;
        return Err("exact participation-bound confirmation required".into());
    }
    let config = MoiraAgentWorkerConfig {
        executable: worker.executable,
        config: worker.config,
        participation,
    };
    match execute(&config) {
        Ok(receipt) => {
            let complete = receipt.outcome != Outcome::FencedUnknown;
            emit(json!({
                "version": 1,
                "peer": "moira",
                "capability": CAPABILITY,
                "complete": complete,
                "status": match receipt.outcome {
                    Outcome::Idle => "idle",
                    Outcome::Observed => "observed",
                    Outcome::FencedUnknown => "fenced_unknown",
                },
                "participation": hex(&participation.0),
                "participation_revision": receipt.participation_revision,
                "next_action": match receipt.outcome {
                    Outcome::Idle => "observe the participation; no prepared owner effect was dispatched",
                    Outcome::Observed => "re-observe the participation through Moira before preparing another effect",
                    Outcome::FencedUnknown => "inspect and reconcile this exact participation; do not retry blindly",
                },
                "boundary": "one owner-worker step does not activate a task-agent journey, accept work, or authorize another step",
            }))?;
            if complete {
                Ok(())
            } else {
                Err("Moira owner worker returned fenced_unknown".into())
            }
        }
        Err(ProcessError::NotStarted(detail)) => failure("not_started", detail),
        Err(ProcessError::OutcomeUnknown(detail)) => failure("outcome_unknown", detail),
    }
}

fn failure(status: &str, detail: String) -> Result<(), Box<dyn std::error::Error>> {
    emit(json!({
        "version": 1,
        "peer": "moira",
        "capability": CAPABILITY,
        "complete": false,
        "status": status,
        "detail": detail,
        "next_action": if status == "outcome_unknown" {
            "inspect and reconcile the exact participation; do not retry blindly"
        } else {
            "repair the deployment-fixed worker executable or configuration before retrying"
        },
        "boundary": "Pantheon did not infer a worker result",
    }))?;
    Err("Moira owner worker did not return a definitive successful receipt".into())
}

fn parse_hex(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("participation must be 32 hexadecimal digits".into());
    }
    let mut result = [0; 16];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn emit(value: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
