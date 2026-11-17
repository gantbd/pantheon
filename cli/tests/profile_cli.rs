use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

fn read_event(reader: &mut impl BufRead) -> serde_json::Value {
    let mut line = String::new();
    assert_ne!(reader.read_line(&mut line).unwrap(), 0, "management EOF");
    serde_json::from_str(&line).unwrap()
}

#[test]
fn example_workspace_profile_passes_bounded_check_without_connecting() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CLI package is nested under the workspace root");
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("interactive")
        .arg("--workspace-profile")
        .arg(root.join("config/workspace.example.json"))
        .arg("--check-profile")
        .output()
        .expect("run profile check");
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.starts_with("workspace profile valid: 9 surfaces\n"));
    for expected in [
        "agora: observation=rooms.read.v2 operator=messages.read.v2 management=topology.attach.v1",
        "moira: observation=sagas.inspect.v1 operator=sagas.reconciliation.v1,sagas.evidence.v1,sagas.operator-receipts.v1 management=topology.attach.v1",
        "daimon: observation=agents.presence.v1 operator=agents.session.inspect.v1,agents.session-set.inspect.v1,agents.session-set.control.v1,agents.session-set.prompt.v1,agents.session-set.permission.v1,agents.session-set.reattach.v1 management=topology.attach.v1",
        "chronos: observation=tasks.observe.v1 operator=attempts.moira.inspect.v1,attempts.daimon.inspect.v1 management=topology.attach.v1",
        "vellum: observation=workspace.inventory.v1 operator=workspace.edit.v1 control=workspace.save.v1 management=topology.attach.v1",
    ] {
        assert!(
            report.contains(expected),
            "capability plan omits {expected}"
        );
    }
    assert!(output.stderr.is_empty());
}

#[test]
fn digest_bound_replacement_refuses_profile_bytes_changed_after_preflight() {
    let scratch = Path::new("/private/tmp").join(format!(
        "pantheon-profile-digest-refusal-{}",
        std::process::id()
    ));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).unwrap();
    }
    fs::create_dir(&scratch).unwrap();
    let profile = scratch.join("workspace.json");
    fs::write(
        &profile,
        r#"{"version":1,"name":"Digest refusal","surfaces":["overview"],"agora":null,"chronos":null,"vellum":null}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["interactive", "--workspace-profile"])
        .arg(&profile)
        .arg("--check-profile")
        .env("PANTHEON_EXPECT_PROFILE_SHA256", "00".repeat(32))
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("workspace profile changed after interactive preflight")
    );
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn profile_capabilities_are_negotiated_for_the_selected_mode() {
    let path = std::env::temp_dir().join(format!(
        "workbench-capabilities-{}.json",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"{"version":1,"name":"Capability gate","surfaces":["agora"],"agora":{"socket":"agora.sock","participant":"operator","requires":["messages.read.v2"]},"chronos":null,"vellum":null}"#,
    )
    .unwrap();

    let interactive = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["interactive", "--workspace-profile"])
        .arg(&path)
        .arg("--check-profile")
        .output()
        .unwrap();
    assert!(interactive.status.success());

    let headless = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["headless", "--workspace-profile"])
        .arg(&path)
        .arg("--check-profile")
        .output()
        .unwrap();
    assert!(!headless.status.success());
    let refusal: serde_json::Value = serde_json::from_slice(&headless.stdout).unwrap();
    assert_eq!(refusal["version"], 1);
    assert_eq!(refusal["mode"], "headless");
    assert_eq!(refusal["complete"], false);
    assert_eq!(refusal["status"], "configuration_refused");
    assert_eq!(refusal["peers"], serde_json::json!([]));
    assert!(
        String::from_utf8_lossy(&headless.stderr)
            .contains("agora does not deliver required headless route capability messages.read.v2")
    );

    fs::remove_file(path).unwrap();
}

#[test]
fn aggregate_profiles_refuse_capabilities_available_only_on_separate_commands() {
    let cases = [
        (
            "moira",
            r#"{"version":1,"name":"Moira route","surfaces":["moira"],"agora":null,"moira":{"socket":"moira.sock","requires":["agent.prompt.submit.v1"]},"chronos":null,"vellum":null}"#,
            "moira does not deliver required interactive route capability agent.prompt.submit.v1",
        ),
        (
            "vellum",
            r#"{"version":1,"name":"Vellum route","surfaces":["workspace"],"agora":null,"chronos":null,"vellum":{"socket":"vellum.sock","credential":null,"requires":["workspace.durable-save.execute.v1"]}}"#,
            "vellum does not deliver required interactive route capability workspace.durable-save.execute.v1",
        ),
    ];
    for (peer, profile, expected) in cases {
        let path = std::env::temp_dir().join(format!(
            "pantheon-separate-route-{peer}-{}.json",
            std::process::id()
        ));
        fs::write(&path, profile).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
            .args(["interactive", "--workspace-profile"])
            .arg(&path)
            .arg("--check-profile")
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{peer} separate route was accepted"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "unexpected {peer} refusal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn chronos_control_requirements_follow_selected_launch_features() {
    let path = std::env::temp_dir().join(format!(
        "pantheon-chronos-route-{}.json",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"{"version":1,"name":"Chronos route","surfaces":["workspace"],"agora":null,"chronos":{"workstreams":[{"name":"Chronos","directory":"chronos"}],"requires":["tasks.observe.v1","tasks.mutate.v1","scheduling.mutate.v5"],"moira":[],"daimon":[],"planning":null},"vellum":null}"#,
    )
    .unwrap();

    let refused = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["interactive", "--workspace-profile"])
        .arg(&path)
        .arg("--check-profile")
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains(
        "chronos does not deliver required interactive route capability tasks.mutate.v1"
    ));

    let accepted = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["interactive", "--workspace-profile"])
        .arg(&path)
        .args([
            "--chronos-allow-writes",
            "--chronos-scheduling",
            "--check-profile",
        ])
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let report = String::from_utf8(accepted.stdout).unwrap();
    assert!(report.contains("control=tasks.mutate.v1,scheduling.mutate.v5"));
    fs::remove_file(path).unwrap();
}

#[test]
fn explicit_headless_syntax_failure_is_versioned_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("headless")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["mode"], "headless");
    assert_eq!(report["complete"], false);
    assert_eq!(report["status"], "configuration_refused");
    assert_eq!(report["hosted_shutdown"], serde_json::Value::Null);
}

#[test]
fn explicit_management_syntax_failure_is_versioned_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("manage")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["mode"], "management");
    assert_eq!(report["complete"], false);
    assert_eq!(report["status"], "configuration_refused");
    assert_eq!(report["hosted_shutdown"], serde_json::Value::Null);
}

#[test]
fn explicit_management_startup_failure_is_versioned_json() {
    let scratch = Path::new("/private/tmp").join(format!(
        "pantheon-management-startup-failure-{}",
        std::process::id()
    ));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).unwrap();
    }
    fs::create_dir(&scratch).unwrap();
    let profile = scratch.join("workspace.json");
    fs::write(
        &profile,
        r#"{"version":1,"name":"Startup refusal","surfaces":["workspace"],"agora":null,"chronos":null,"vellum":{"deployment":"hosted","socket":"vellum.sock","credential":null,"roots":["missing.txt"],"requires":["workspace.inventory.v1"]}}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["manage", "--workspace-profile"])
        .arg(&profile)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["mode"], "management");
    assert_eq!(report["complete"], false);
    assert_eq!(report["status"], "startup_refused");
    assert!(
        report["detail"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn headless_operation_id_is_bounded_before_connection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CLI package is nested under the workspace root");
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["headless", "--workspace-profile"])
        .arg(root.join("config/workspace.example.json"))
        .args(["--operation-id", "not an opaque id", "--check-profile"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "configuration_refused");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("--operation-id must be 1-64 ASCII letters, digits, '.', '_', ':', or '-'")
    );
}

#[test]
fn reconfiguration_plan_is_bounded_and_does_not_claim_live_apply() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CLI package is nested under the workspace root");
    let profile = root.join("config/workspace.example.json");
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("reconfigure-plan")
        .arg(&profile)
        .arg(&profile)
        .args(["--operation-id", "qualification:reconfigure-noop"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["version"], 1);
    assert_eq!(plan["operation_id"], "qualification:reconfigure-noop");
    assert_eq!(plan["scope"], "single_user_local_machine");
    assert_eq!(plan["live_apply_supported"], false);
    assert_eq!(plan["minimum_action"], "none");
    assert_eq!(plan["changes"], serde_json::json!([]));
}

#[test]
fn headless_profile_can_host_and_drain_real_chronos_and_vellum() {
    let scratch =
        Path::new("/private/tmp").join(format!("workbench-hosted-chronos-{}", std::process::id()));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).unwrap();
    }
    fs::create_dir(&scratch).unwrap();
    fs::set_permissions(&scratch, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(scratch.join("source.txt"), "hosted Vellum").unwrap();
    let profile = scratch.join("workspace.json");
    fs::write(
        &profile,
        r#"{"version":1,"name":"Hosted","surfaces":["workspace"],"agora":null,"chronos":{"deployment":"hosted","workstreams":[{"name":"Atlas","directory":"chronos"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":{"deployment":"hosted","socket":"vellum.sock","credential":null,"roots":["source.txt"],"requires":["workspace.inventory.v1"]}}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["headless", "--workspace-profile"])
        .arg(&profile)
        .args(["--operation-id", "qualification:hosted-chronos-vellum"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], true);
    assert_eq!(
        report["operation_id"],
        "qualification:hosted-chronos-vellum"
    );
    assert_eq!(report["peers"][0]["peer"], "chronos");
    assert_eq!(report["peers"][0]["status"], "current");
    assert_eq!(report["peers"][1]["peer"], "vellum");
    assert_eq!(report["peers"][1]["status"], "current");
    assert_eq!(report["peers"][1]["documents"], 1);
    assert_eq!(report["management"]["scope"], "single_user_local_machine");
    assert_eq!(
        report["management"]["operation_id"],
        "qualification:hosted-chronos-vellum"
    );
    assert_eq!(
        report["management"]["topology_lifecycle"],
        "local_host_stopped"
    );
    assert_eq!(report["management"]["peers"][0]["peer"], "chronos");
    assert_eq!(report["management"]["peers"][0]["instance"], "Atlas");
    assert_eq!(report["management"]["peers"][0]["deployment"], "hosted");
    assert_eq!(
        report["management"]["peers"][0]["observed_readiness"],
        "ready"
    );
    assert_eq!(report["management"]["peers"][0]["certainty"], "confirmed");
    assert_eq!(report["management"]["peers"][0]["lifecycle"], "stopped");
    assert_eq!(
        report["management"]["peers"][0]["shutdown_disposition"],
        "clean"
    );
    assert_eq!(
        report["management"]["peers"][1]["authority_mode"],
        "peer_owned_read_only"
    );
    assert_eq!(
        report["management"]["peers"][1]["reconfiguration"],
        "restart_required"
    );
    assert_eq!(report["hosted_shutdown"]["status"], "clean");
    assert_eq!(report["hosted_shutdown"]["receipts"][0]["peer"], "vellum");
    assert_eq!(report["hosted_shutdown"]["receipts"][1]["peer"], "chronos");
    assert!(!scratch.join("chronos/chronos.sock").exists());
    assert!(!scratch.join("chronos/chronos.control.sock").exists());
    assert!(!scratch.join("vellum.sock").exists());
    assert!(!scratch.join("vellum.sock.control").exists());
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn management_session_observes_live_host_and_drains_on_command() {
    let scratch =
        Path::new("/private/tmp").join(format!("pantheon-management-{}", std::process::id()));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).unwrap();
    }
    fs::create_dir(&scratch).unwrap();
    fs::set_permissions(&scratch, fs::Permissions::from_mode(0o700)).unwrap();
    let profile = scratch.join("workspace.json");
    fs::write(
        &profile,
        r#"{"version":1,"name":"Managed","surfaces":["overview"],"agora":null,"chronos":{"deployment":"hosted","workstreams":[{"name":"Atlas","directory":"chronos"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":null}"#,
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["manage", "--workspace-profile"])
        .arg(&profile)
        .args(["--operation-id", "management-session-1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"status\nobserve management-observation-1\ndrain\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0]["event"], "ready");
    assert_eq!(events[0]["request_flow"], "one_in_flight");
    assert_eq!(events[0]["topology"]["lifecycle"], "ready");
    assert_eq!(events[0]["topology"]["hosted_peers"][0]["peer"], "chronos");
    assert_eq!(events[1]["event"], "status");
    assert_eq!(events[1]["topology"]["reconfiguration_allowed"], true);
    assert_eq!(events[2]["event"], "observation");
    assert_eq!(events[2]["operation_id"], "management-observation-1");
    assert_eq!(events[2]["complete"], true);
    assert_eq!(
        events[2]["management"]["topology_lifecycle"],
        "local_host_ready"
    );
    assert_eq!(events[2]["management"]["peers"][0]["lifecycle"], "running");
    assert_eq!(events[3]["event"], "drain");
    assert_eq!(events[3]["complete"], true);
    assert_eq!(events[3]["shutdown"]["receipts"][0]["peer"], "chronos");
    assert_eq!(events[3]["shutdown"]["receipts"][0]["instance"], "Atlas");
    assert!(!scratch.join("chronos/chronos.sock").exists());

    let mut failed = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["manage", "--workspace-profile"])
        .arg(&profile)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut oversized = vec![b'x'; 4 * 1_024 + 1];
    oversized.push(b'\n');
    failed.stdin.take().unwrap().write_all(&oversized).unwrap();
    let failed = failed.wait_with_output().unwrap();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("management command exceeds 4 KiB"));
    assert!(!scratch.join("chronos/chronos.sock").exists());
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn management_session_preflights_drains_and_execs_a_replacement_profile() {
    let scratch = Path::new("/private/tmp").join(format!(
        "pantheon-management-reconfigure-{}",
        std::process::id()
    ));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).unwrap();
    }
    fs::create_dir(&scratch).unwrap();
    fs::set_permissions(&scratch, fs::Permissions::from_mode(0o700)).unwrap();
    let first = scratch.join("first.json");
    let second = scratch.join("second profile.json");
    fs::write(
        &first,
        r#"{"version":1,"name":"First","surfaces":["overview"],"agora":null,"chronos":{"deployment":"hosted","workstreams":[{"name":"Atlas","directory":"chronos-a"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":null}"#,
    )
    .unwrap();
    fs::write(
        &second,
        r#"{"version":1,"name":"Second","surfaces":["overview"],"agora":null,"chronos":{"deployment":"hosted","workstreams":[{"name":"Borealis","directory":"chronos-b"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":null}"#,
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args(["manage", "--workspace-profile"])
        .arg(&first)
        .args(["--operation-id", "management-before-reconfigure"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut errors = child.stderr.take().unwrap();

    let ready = read_event(&mut output);
    assert_eq!(ready["event"], "ready");
    assert_eq!(ready["request_flow"], "one_in_flight");
    assert_eq!(ready["session_id"], "management-before-reconfigure");
    assert!(scratch.join("chronos-a/chronos.sock").exists());

    writeln!(
        input,
        "reconfigure invalid-op {}",
        scratch.join("missing.json").display()
    )
    .unwrap();
    input.flush().unwrap();
    let refused = read_event(&mut output);
    assert_eq!(refused["event"], "refusal");
    assert_eq!(refused["status"], "replacement_profile_refused");
    assert!(scratch.join("chronos-a/chronos.sock").exists());

    writeln!(
        input,
        "reconfigure qualification:profile-replacement {}",
        second.display()
    )
    .unwrap();
    input.flush().unwrap();
    let handoff = read_event(&mut output);
    assert_eq!(handoff["event"], "reconfigure");
    assert_eq!(handoff["status"], "old_topology_drained");
    assert_eq!(handoff["complete"], false);
    assert_eq!(
        handoff["plan"]["minimum_action"],
        "full_local_topology_restart"
    );
    assert_eq!(handoff["shutdown"]["receipts"][0]["instance"], "Atlas");

    let replacement = read_event(&mut output);
    assert_eq!(replacement["event"], "ready");
    assert_eq!(
        replacement["session_id"],
        "qualification:profile-replacement"
    );
    assert_eq!(
        replacement["topology"]["hosted_peers"][0]["instance"],
        "Borealis"
    );
    assert!(!scratch.join("chronos-a/chronos.sock").exists());
    assert!(scratch.join("chronos-b/chronos.sock").exists());

    writeln!(input, "observe qualification:replacement-observation").unwrap();
    input.flush().unwrap();
    let observation = read_event(&mut output);
    assert_eq!(observation["event"], "observation");
    assert_eq!(observation["workspace"], "Second");
    assert_eq!(observation["peers"][0]["instance"], "Borealis");
    writeln!(input, "drain").unwrap();
    input.flush().unwrap();
    let drain = read_event(&mut output);
    assert_eq!(drain["event"], "drain");
    assert_eq!(drain["complete"], true);
    drop(input);
    drop(output);
    let status = child.wait().unwrap();
    let mut stderr = String::new();
    errors.read_to_string(&mut stderr).unwrap();
    assert!(status.success(), "stderr: {stderr}");
    assert!(!scratch.join("chronos-b/chronos.sock").exists());
    fs::remove_dir_all(scratch).unwrap();
}
