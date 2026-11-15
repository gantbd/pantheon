use std::{path::Path, process::Command};

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
