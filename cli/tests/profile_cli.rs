use std::{fs, path::Path, process::Command};

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
