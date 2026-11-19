#![cfg(unix)]

use moira_core::{agent_participation::ProfileIdV1, task_agent_journey as domain};
use moira_local_daemon::unix::{Authenticator, TaskAgentJourneyDaemon};
use moira_service::{
    AccessV1,
    task_agent_journey::{TaskAgentJourneyService, TrustedTimeV1},
};
use moira_sqlite::task_agent_journey::{
    TaskAgentJourneySqliteStore, TaskAgentJourneyStoreLimitsV1,
};
use moira_wire::LimitsV1;
use serde_json::{Value, json};
use std::{
    convert::Infallible,
    fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "pantheon-task-agent-command-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self {
            database: root.join("task-agent.db"),
            socket: root.join("task-agent.sock"),
            root,
        }
    }

    fn store(&self) -> TaskAgentJourneySqliteStore {
        TaskAgentJourneySqliteStore::open(
            &self.database,
            domain::OwnerIdV1([1; 16]),
            store_limits(),
            [template()],
        )
        .unwrap()
    }

    fn request_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Authentication;
impl Authenticator for Authentication {
    type Error = Infallible;
    fn authenticate(&mut self, _: &UnixStream) -> Result<AccessV1, Self::Error> {
        Ok(AccessV1 {
            read: true,
            mutate: true,
        })
    }
}

struct Time(u64);
impl TrustedTimeV1 for Time {
    type Error = Infallible;
    fn now(&mut self) -> Result<u64, Self::Error> {
        let result = self.0;
        self.0 += 1;
        Ok(result)
    }
}

fn store_limits() -> TaskAgentJourneyStoreLimitsV1 {
    let domain = domain::LimitsV1 {
        max_templates: 2,
        max_previews: 4,
        max_journeys: 4,
        max_preview_ttl: 60,
    };
    TaskAgentJourneyStoreLimitsV1 {
        codec: domain::persistence::CodecLimitsV1 {
            max_bytes: 65_536,
            domain,
        },
        domain,
        max_database_pages: 64,
    }
}

fn template() -> domain::TemplateV1 {
    domain::TemplateV1 {
        id: domain::TemplateIdV1([2; 16]),
        version: 1,
        digest: [3; 32],
        work_mode: domain::WorkModeV1::Collaborator,
        profile: ProfileIdV1([4; 16]),
        room_policy: domain::RoomPolicyV1::SharedContext,
        budget: domain::BudgetV1 {
            max_context_bytes: 4_096,
            max_output_tokens: 512,
            max_elapsed_millis: 10_000,
        },
        grounding_policy_digest: [5; 32],
        authority_policy_digest: [6; 32],
        consequence_digest: [7; 32],
        retired: false,
    }
}

fn run(fixture: &Fixture, time: u64, arguments: &[&str]) -> Value {
    let mut daemon = TaskAgentJourneyDaemon::bind_new(
        &fixture.socket,
        TaskAgentJourneyService::new(fixture.store(), Time(time)),
        Authentication,
        LimitsV1::default(),
        Duration::from_secs(3),
        8,
    )
    .unwrap();
    let worker = thread::spawn(move || daemon.serve_next().unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(arguments)
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn headless_binary_previews_confirms_admits_and_reopens_exact_journey() {
    let fixture = Fixture::new();
    let socket = fixture.socket.to_str().unwrap();
    let catalogue = run(&fixture, 90, &["moira-task-agent-catalogue", socket]);
    assert_eq!(catalogue["status"], "current");
    assert_eq!(catalogue["storage_revision"], 0);
    assert_eq!(catalogue["templates"].as_array().unwrap().len(), 1);

    let preview_path = fixture.request_path("preview.json");
    write_json(
        &preview_path,
        &json!({
            "version": 1, "expected_storage_revision": null,
            "operation": "08".repeat(16),
            "template": {"id": "02".repeat(16), "version": 1, "digest": "03".repeat(32)},
            "task": {"external_task_key": "09".repeat(16), "incarnation": 1, "chronos_revision": 11, "evidence_digest": "0a".repeat(32)},
            "profile": "04".repeat(16), "room_policy": "shared-context",
            "initiating_principal": "0c".repeat(16), "accountable_human": "0d".repeat(16),
            "objective_digest": "0e".repeat(32), "acceptance_digest": "0f".repeat(32),
            "current_evidence_digest": "10".repeat(32), "requested_ttl": 30
        }),
    );
    let preview = run(
        &fixture,
        100,
        &[
            "moira-task-agent-preview",
            socket,
            preview_path.to_str().unwrap(),
        ],
    );
    assert_eq!(preview["status"], "previewed");
    assert_eq!(preview["storage_revision"], 1);
    let preview_sequence = preview["preview"]["sequence"].as_u64().unwrap();
    let owner_revision = preview["preview"]["owner_revision"].as_u64().unwrap();
    let confirmation = preview["required_confirmation"].as_str().unwrap();

    let instantiate_path = fixture.request_path("instantiate.json");
    write_json(
        &instantiate_path,
        &json!({
            "version": 1, "expected_storage_revision": 1,
            "operation": "11".repeat(16), "preview_owner": "01".repeat(16),
            "preview_sequence": preview_sequence, "expected_owner_revision": owner_revision,
            "accountable_human": "0d".repeat(16), "consequence_digest": "07".repeat(32)
        }),
    );
    let admitted = run(
        &fixture,
        101,
        &[
            "moira-task-agent-instantiate",
            socket,
            instantiate_path.to_str().unwrap(),
            confirmation,
        ],
    );
    assert_eq!(admitted["status"], "admitted");
    assert_eq!(admitted["storage_revision"], 2);
    let journey_sequence = admitted["journey"]["sequence"]
        .as_u64()
        .unwrap()
        .to_string();

    let observed = run(
        &fixture,
        200,
        &[
            "moira-task-agent-observe",
            socket,
            &"01".repeat(16),
            &journey_sequence,
        ],
    );
    assert_eq!(observed["status"], "current");
    assert_eq!(observed["storage_revision"], 2);
    assert_eq!(observed["journey"]["phase"], "Admitted");
    assert_eq!(
        observed["journey"]["preview"]["accountable_human"],
        "0d".repeat(16)
    );
    assert_eq!(
        observed["journey"]["preview"]["task"]["external_task_key"],
        "09".repeat(16)
    );
    assert_eq!(observed["journey"]["preview"]["task"]["incarnation"], 1);
    assert_eq!(
        observed["journey"]["preview"]["task"]["chronos_revision"],
        11
    );
    assert_eq!(
        observed["journey"]["preview"]["template"]["id"],
        "02".repeat(16)
    );
    assert_eq!(observed["journey"]["preview"]["profile"], "04".repeat(16));
    assert_eq!(
        observed["journey"]["preview"]["initiating_principal"],
        "0c".repeat(16)
    );
    assert_eq!(
        observed["journey"]["preview"]["objective_digest"],
        "0e".repeat(32)
    );
    assert_eq!(
        observed["journey"]["preview"]["acceptance_digest"],
        "0f".repeat(32)
    );
    assert_eq!(observed["journey"]["activation"]["status"], "not_published");
    assert!(observed["journey"]["activation"]["aegis_delegation"].is_null());
    assert!(observed["journey"]["activation"]["daimon_session"].is_null());
    assert!(observed["journey"]["activation"]["completed_work_receipt"].is_null());
}

#[test]
fn unavailable_owner_returns_observable_orienting_json() {
    let fixture = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "moira-task-agent-catalogue",
            fixture.socket.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], false);
    assert_eq!(report["status"], "not_sent");
    assert_eq!(
        report["detail"]["next_action"],
        "verify the selected Moira owner endpoint and bounded client contract"
    );
}
