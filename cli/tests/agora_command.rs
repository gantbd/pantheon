#![cfg(unix)]

use std::process::Command;

#[test]
fn create_room_refuses_bad_confirmation_before_connecting() {
    let missing = std::env::temp_dir().join(format!(
        "workbench-agora-create-missing-{}",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agora-create-room",
            missing.to_str().unwrap(),
            "operator",
            "conversation",
            "delivery",
            "Delivery Room",
            "invite-only",
            "7",
            "CREATE SOMETHING ELSE",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["capability"], "rooms.create.v2");
    assert_eq!(report["status"], "confirmation_refused");
    assert_eq!(
        report["required_confirmation"],
        "CREATE AGORA ROOM delivery NAMED Delivery Room ADMISSION INVITE ONLY AT REVISION 7"
    );
}
