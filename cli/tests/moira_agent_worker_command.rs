#![cfg(unix)]

use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const PARTICIPATION: &str = "01010101010101010101010101010101";

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(receipt_participation: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "pantheon-moira-worker-command-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("agent.json"), b"{}\n").unwrap();
        let worker = root.join("worker.sh");
        fs::write(
            &worker,
            format!(
                "#!/bin/sh\nprintf '%s\\n' '{{\"outcome\":\"observed\",\"participation\":\"{receipt_participation}\",\"participation_revision\":7}}'\n"
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&worker).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&worker, permissions).unwrap();
        fs::write(
            root.join("profile.json"),
            format!(
                r#"{{
  "version": 1,
  "name": "Headless Moira worker",
  "surfaces": ["moira"],
  "moira": {{
    "socket": "saga.sock",
    "participation_socket": "participation.sock",
    "participation": "{PARTICIPATION}",
    "agent_worker": {{"executable": "worker.sh", "config": "agent.json"}}
  }}
}}"#
            ),
        )
        .unwrap();
        Self { root }
    }

    fn profile(&self) -> PathBuf {
        self.root.join("profile.json")
    }

    fn run(&self, confirmation: &str) -> Output {
        let profile = self.profile();
        Command::new(env!("CARGO_BIN_EXE_pantheon"))
            .args([
                "moira-participation-worker-run",
                profile.to_str().unwrap(),
                confirmation,
            ])
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn exact_confirmation_runs_profile_bound_worker_and_emits_receipt() {
    let fixture = Fixture::new(PARTICIPATION);
    let refused = fixture.run("RUN SOMETHING ELSE");
    assert!(!refused.status.success());
    let refused = report(&refused);
    assert_eq!(refused["status"], "confirmation_refused");
    assert_eq!(
        refused["required_confirmation"],
        format!("RUN MOIRA PARTICIPATION WORKER {PARTICIPATION}")
    );

    let accepted = fixture.run(&format!("RUN MOIRA PARTICIPATION WORKER {PARTICIPATION}"));
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let accepted = report(&accepted);
    assert_eq!(accepted["capability"], "agent.worker.step.v1");
    assert_eq!(accepted["status"], "observed");
    assert_eq!(accepted["participation"], PARTICIPATION);
    assert_eq!(accepted["participation_revision"], 7);
}

#[test]
fn mismatched_owner_receipt_fences_headless_retry() {
    let fixture = Fixture::new("02020202020202020202020202020202");
    let output = fixture.run(&format!("RUN MOIRA PARTICIPATION WORKER {PARTICIPATION}"));
    assert!(!output.status.success());
    let report = report(&output);
    assert_eq!(report["status"], "outcome_unknown");
    assert!(
        report["detail"]
            .as_str()
            .unwrap()
            .contains("different participation")
    );
    assert!(
        report["next_action"]
            .as_str()
            .unwrap()
            .contains("do not retry")
    );
}
