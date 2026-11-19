#![cfg(unix)]

use chronos_protocol::{
    Phase, Reference, Revision, Snapshot,
    alignment_v1::{
        Cohort, ControlTaskObservation, Currentness, Observation, Outcome, OwnerEvidenceReference,
        Recommendation, Response, SourceReference, SourceTarget,
    },
};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
    process::Command,
};

fn directory(label: &str) -> PathBuf {
    PathBuf::from("/tmp").join(format!(
        "pantheon-alignment-command-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn source(document: &str) -> SourceReference {
    SourceReference {
        namespace: "product-delivery".into(),
        document: document.into(),
        evidence_domain: "vellum-workspace-source-revision-v1".into(),
        evidence_revision: "1".into(),
        byte_length: "64".into(),
        target: SourceTarget::Document,
    }
}

fn observation() -> Observation {
    Observation {
        version: 1,
        record_revision: "0".into(),
        chronos_scope: "product-delivery".into(),
        goal: source("goal"),
        control_task: ControlTaskObservation {
            reference: Reference {
                task: "pm-control".into(),
                incarnation: "0".into(),
            },
            revision: Revision::Finite("7".into()),
            snapshot: Snapshot {
                phase: Phase::Ready,
                attempt_number: None,
                dependency_count: 0,
                required_capability_count: 1,
                priority: 10,
                max_attempts: 1,
            },
        },
        cohort: Cohort {
            external_advocate: "human-sponsor".into(),
            product_manager: "pm-agent".into(),
            frontier_supervisor: "frontier-agent".into(),
        },
        budget_policy: OwnerEvidenceReference {
            owner: "vellum".into(),
            resource: "budget-policy".into(),
            revision: "4".into(),
        },
        evidence_frontier: vec![],
        unresolved_dissent: vec![],
        pending_human_gates: vec![],
        proposal: source("pm-proposal"),
        author: "pm-agent".into(),
        recommendation: Recommendation::EscalateHuman,
        recommended_next: vec![],
        acceptance_evidence: vec![],
        currentness: Currentness::Current,
    }
}

#[test]
fn exact_alignment_owner_projection_is_independently_agent_readable() {
    let directory = directory("current");
    std::fs::create_dir(&directory).unwrap();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = chronos_local::alignment_socket_path(&directory);
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut frame = Vec::new();
        BufReader::new(&stream)
            .read_until(b'\n', &mut frame)
            .unwrap();
        assert!(String::from_utf8_lossy(&frame).contains("product-delivery"));
        serde_json::to_writer(
            &mut stream,
            &Response {
                version: 1,
                outcome: Outcome::Observed {
                    observation: Box::new(observation()),
                },
            },
        )
        .unwrap();
        stream.write_all(b"\n").unwrap();
    });

    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "chronos-alignment-observe",
            directory.to_str().unwrap(),
            "product-delivery",
        ])
        .output()
        .expect("run Chronos alignment agent gateway");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["capability"], "product-alignment.observe.v1");
    assert_eq!(value["status"], "current");
    assert_eq!(value["complete"], true);
    assert_eq!(
        value["owner"]["observation"]["chronos_scope"],
        "product-delivery"
    );
    assert_eq!(value["owner"]["observation"]["author"], "pm-agent");
    assert_eq!(value["owner"]["observation"]["currentness"], "current");
    assert!(
        value["boundary"]
            .as_str()
            .unwrap()
            .contains("grants no owner configuration")
    );
    server.join().unwrap();
    std::fs::remove_file(socket).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn alignment_gateway_accepts_only_one_deployment_pinned_scope() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "chronos-alignment-observe",
            "/not/contacted",
            "product-delivery",
            "another-scope",
        ])
        .output()
        .expect("run malformed Chronos alignment gateway");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("pantheon chronos-alignment-observe DIRECTORY SCOPE")
    );
}
