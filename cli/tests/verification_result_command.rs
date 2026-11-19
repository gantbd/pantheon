use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

fn fixture() -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "pantheon-verification-result-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let request = root.join("request.json");
    fs::write(
        &request,
        br#"{"version":1,"operation_id":"11111111111111111111111111111111","task":{"key":"22222222222222222222222222222222","incarnation":3,"attempt":4},"source":{"owner":"vellum","workspace":5,"document":6,"revision":7,"bytes":8,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"profile":{"semantic_capability":"verification.test.run.v1","digest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},"execution":{"owner":"daimon","execution_id":"33333333333333333333333333333333","evidence_digest":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","output_digest":"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","output_bytes":9},"result":{"disposition":"passed","exit_code":0,"duration_ms":10}}"#,
    )
    .unwrap();
    fs::set_permissions(&request, fs::Permissions::from_mode(0o600)).unwrap();
    (root, request)
}

fn run(request: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("verification-result-validate")
        .arg(request)
        .output()
        .unwrap()
}

#[test]
fn closed_private_envelope_is_validated_without_claiming_publication() {
    let (root, request) = fixture();
    let output = run(&request);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["capability"],
        "verification.result-envelope.validate.v1"
    );
    assert_eq!(value["status"], "validated_not_published");
    assert_eq!(value["complete"], true);
    assert_eq!(value["published"], false);
    assert_eq!(value["task"]["attempt"], 4);
    assert_eq!(value["source"]["revision"], 7);
    assert_eq!(value["result"]["disposition"], "passed");
    assert!(
        value["request_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn insecure_or_semantically_inconsistent_documents_refuse_before_publication() {
    let (root, request) = fixture();
    fs::set_permissions(&request, fs::Permissions::from_mode(0o644)).unwrap();
    let insecure = run(&request);
    assert!(!insecure.status.success());
    let value: serde_json::Value = serde_json::from_slice(&insecure.stdout).unwrap();
    assert_eq!(value["status"], "request_refused");
    assert_eq!(value["published"], serde_json::Value::Null);

    fs::set_permissions(&request, fs::Permissions::from_mode(0o600)).unwrap();
    let bytes = fs::read_to_string(&request).unwrap();
    fs::write(
        &request,
        bytes.replace("\"exit_code\":0", "\"exit_code\":1"),
    )
    .unwrap();
    let inconsistent = run(&request);
    assert!(!inconsistent.status.success());
    let value: serde_json::Value = serde_json::from_slice(&inconsistent.stdout).unwrap();
    assert!(
        value["detail"]
            .as_str()
            .unwrap()
            .contains("closed version-1 attribution")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_fields_and_extra_arguments_are_not_accepted() {
    let (root, request) = fixture();
    let bytes = fs::read_to_string(&request).unwrap();
    fs::write(&request, bytes.replacen("{", "{\"invented\":true,", 1)).unwrap();
    let unknown = run(&request);
    assert!(!unknown.status.success());
    let value: serde_json::Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert!(value["detail"].as_str().unwrap().contains("unknown field"));

    let extra = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "verification-result-validate",
            request.to_str().unwrap(),
            "extra",
        ])
        .output()
        .unwrap();
    assert!(!extra.status.success());
    fs::remove_dir_all(root).unwrap();
}
