use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

#[test]
fn example_workspace_profile_passes_bounded_check_without_connecting() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CLI package is nested under the workspace root");
    let output = Command::new(env!("CARGO_BIN_EXE_workbench-tui"))
        .arg("interactive")
        .arg("--workspace-profile")
        .arg(root.join("config/workspace.example.json"))
        .arg("--check-profile")
        .output()
        .expect("run profile check");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "workspace profile valid: 8 surfaces\n"
    );
    assert!(output.stderr.is_empty());
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

    let interactive = Command::new(env!("CARGO_BIN_EXE_workbench-tui"))
        .args(["interactive", "--workspace-profile"])
        .arg(&path)
        .arg("--check-profile")
        .output()
        .unwrap();
    assert!(interactive.status.success());

    let headless = Command::new(env!("CARGO_BIN_EXE_workbench-tui"))
        .args(["headless", "--workspace-profile"])
        .arg(&path)
        .arg("--check-profile")
        .output()
        .unwrap();
    assert!(!headless.status.success());
    assert!(
        String::from_utf8_lossy(&headless.stderr)
            .contains("agora does not support required headless capability messages.read.v2")
    );

    fs::remove_file(path).unwrap();
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

    let output = Command::new(env!("CARGO_BIN_EXE_workbench-tui"))
        .args(["headless", "--workspace-profile"])
        .arg(&profile)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], true);
    assert_eq!(report["peers"][0]["peer"], "chronos");
    assert_eq!(report["peers"][0]["status"], "current");
    assert_eq!(report["peers"][1]["peer"], "vellum");
    assert_eq!(report["peers"][1]["status"], "current");
    assert_eq!(report["peers"][1]["documents"], 1);
    assert_eq!(report["hosted_shutdown"]["status"], "clean");
    assert_eq!(report["hosted_shutdown"]["receipts"][0]["peer"], "vellum");
    assert_eq!(report["hosted_shutdown"]["receipts"][1]["peer"], "chronos");
    assert!(!scratch.join("chronos/chronos.sock").exists());
    assert!(!scratch.join("chronos/chronos.control.sock").exists());
    assert!(!scratch.join("vellum.sock").exists());
    assert!(!scratch.join("vellum.sock.control").exists());
    fs::remove_dir_all(scratch).unwrap();
}
