//! Exact, read-only Chronos task observation for bounded agent tool profiles.

use chronos_client::WorkflowWireClientV9;
use chronos_protocol::{Limits, workflow_v9 as v9};
use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

pub const CAPABILITY: &str = "tasks.workflow.observe.v9";

const LIMITS: Limits = Limits {
    max_frame_bytes: 64 * 1024,
    max_id_bytes: 256,
    max_graph_tasks: 1,
    max_dependencies_per_task: 1,
    max_capabilities_per_task: 1,
    max_capability_bytes: 1,
    max_page_entries: 1,
};

pub fn observe(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let directory = PathBuf::from(required(
        &mut args,
        "chronos task observation requires SERVICE_DIRECTORY TASK",
    )?);
    let task = required_text(
        &mut args,
        "chronos task observation requires SERVICE_DIRECTORY TASK",
    )?;
    if args.next().is_some() {
        return Err("unexpected Chronos task observation argument".into());
    }

    let mut client = chronos_local::LocalClient::new(directory);
    match observe_with(&mut client, &task) {
        Ok(value) => emit(value),
        Err(failure) => fail(failure),
    }
}

#[derive(Debug)]
struct Failure {
    status: &'static str,
    detail: String,
    next_action: &'static str,
}

fn observe_with<C>(client: &mut C, task: &str) -> Result<Value, Failure>
where
    C: WorkflowWireClientV9,
    C::Error: TransportFailure,
{
    let request = v9::Request {
        version: 9,
        command: v9::Command::Snapshot {
            task: task.to_owned(),
        },
    };
    let response = client
        .request_workflow_v9(request.clone())
        .map_err(TransportFailure::failure)?;
    response
        .validate_for(&request, LIMITS)
        .map_err(|error| Failure {
            status: "invalid_reply",
            detail: format!("Chronos returned an invalid V9 task snapshot: {error:?}"),
            next_action: "stop agent admission and inspect Chronos protocol compatibility",
        })?;

    match response.outcome {
        v9::Outcome::Snapshot {
            value: Some(entry),
            revision: Some(revision),
        } if entry.reference.task == task => Ok(json!({
            "version": 1,
            "peer": "chronos",
            "capability": CAPABILITY,
            "complete": true,
            "status": "current",
            "revision": revision,
            "task": entry.reference.task,
            "incarnation": entry.reference.incarnation,
            "workflow_status": entry.snapshot.status,
            "execution_phase": entry.snapshot.execution_phase,
            "attempt_number": entry.snapshot.attempt_number,
            "priority": entry.snapshot.priority,
            "dependency_count": entry.snapshot.dependency_count,
            "required_capability_count": entry.snapshot.required_capability_count,
            "blocker": entry.snapshot.blocker,
            "boundary": "exact content-minimized task observation only; no board, sibling task, mutation, lease, delegation, approval, or completion authority",
        })),
        v9::Outcome::Snapshot { value: Some(_), .. } => Err(Failure {
            status: "invalid_reply",
            detail: "Chronos returned a different task than requested".into(),
            next_action: "stop agent admission and inspect the Chronos owner boundary",
        }),
        v9::Outcome::Snapshot {
            value: None,
            revision: None,
        } => Err(Failure {
            status: "not_found",
            detail: format!("Chronos does not report task {task}"),
            next_action: "re-observe the authoritative task selection; do not substitute another task",
        }),
        v9::Outcome::Refused { code } => Err(Failure {
            status: "refused",
            detail: format!("Chronos refused exact task observation: {code:?}"),
            next_action: "verify the exact task identifier and owner policy; do not broaden the grant",
        }),
        _ => Err(Failure {
            status: "invalid_reply",
            detail: "Chronos returned a non-snapshot outcome".into(),
            next_action: "stop agent admission and inspect Chronos protocol compatibility",
        }),
    }
}

trait TransportFailure {
    fn failure(self) -> Failure;
}

impl TransportFailure for chronos_local::CallError {
    fn failure(self) -> Failure {
        match self {
            Self::NotSent => Failure {
                status: "not_sent",
                detail: "Chronos request bytes were not sent".into(),
                next_action: "verify the selected Chronos service directory and retry the read-only observation",
            },
            Self::OutcomeUnknown => Failure {
                status: "outcome_unknown",
                detail: "Chronos reply was not observed after the read-only request was sent"
                    .into(),
                next_action: "inspect service health, then safely re-observe this read-only task snapshot",
            },
            Self::InvalidResponse => Failure {
                status: "invalid_reply",
                detail: "Chronos returned an undecodable or contract-invalid V9 reply".into(),
                next_action: "stop agent admission and inspect the running Chronos protocol version",
            },
        }
    }
}

fn fail(failure: Failure) -> Result<(), Box<dyn std::error::Error>> {
    emit(json!({
        "version": 1,
        "peer": "chronos",
        "capability": CAPABILITY,
        "complete": false,
        "status": failure.status,
        "detail": failure.detail,
        "next_action": failure.next_action,
        "boundary": "no agent action should proceed from missing, refused, or invalid task evidence",
    }))?;
    Err(std::io::Error::other("Chronos task observation unavailable").into())
}

fn emit(value: Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use chronos_protocol::{Reference, Revision};
    use std::collections::VecDeque;

    struct Client(VecDeque<v9::Response>);

    impl WorkflowWireClientV9 for Client {
        type Error = ();

        fn request_workflow_v9(&mut self, _: v9::Request) -> Result<v9::Response, Self::Error> {
            Ok(self.0.pop_front().expect("fixture response"))
        }
    }

    impl TransportFailure for () {
        fn failure(self) -> Failure {
            Failure {
                status: "not_sent",
                detail: "fixture transport failure".into(),
                next_action: "repair fixture",
            }
        }
    }

    fn response(task: &str) -> v9::Response {
        v9::Response {
            version: 9,
            outcome: v9::Outcome::Snapshot {
                value: Some(v9::Entry {
                    reference: Reference {
                        task: task.into(),
                        incarnation: "7".into(),
                    },
                    snapshot: v9::Snapshot {
                        status: v9::Status::InProgress,
                        blocker: None,
                        execution_phase: v9::ExecutionPhase::Running,
                        attempt_number: Some(2),
                        dependency_count: 0,
                        required_capability_count: 3,
                        priority: 5,
                        max_attempts: 4,
                    },
                }),
                revision: Some(Revision::Finite("11".into())),
            },
        }
    }

    #[test]
    fn exact_snapshot_exposes_only_one_task_and_no_authority() {
        let mut client = Client(VecDeque::from([response("work-1")]));
        let value = observe_with(&mut client, "work-1").unwrap();
        assert_eq!(value["capability"], CAPABILITY);
        assert_eq!(value["task"], "work-1");
        assert_eq!(value["workflow_status"], "in_progress");
        assert_eq!(value["attempt_number"], 2);
        assert!(value.get("tasks").is_none());
        assert!(value["boundary"].as_str().unwrap().contains("no board"));
    }

    #[test]
    fn mismatched_task_fails_closed() {
        let mut client = Client(VecDeque::from([response("other")]));
        let failure = observe_with(&mut client, "work-1").unwrap_err();
        assert_eq!(failure.status, "invalid_reply");
    }
}
