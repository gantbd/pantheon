use serde_json::Value;
use std::{fs, path::Path};

#[test]
fn composition_source_is_exact_unpublished_and_has_no_engine_implementation_dependency() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source: Value = serde_json::from_str(
        &fs::read_to_string(root.join("release/source.json")).expect("source contract"),
    )
    .expect("source JSON");
    assert_eq!(source["schema"], 2);
    assert_eq!(
        source["components"],
        serde_json::json!(["agora", "chronos", "vellum"])
    );
    assert_eq!(
        source["inputs"]["vellumTui"]["revision"],
        "4d9f81625fc4898d855110cf3541eb43584ba631"
    );
    assert_eq!(
        source["inputs"]["chronosTui"]["revision"],
        "3928f5bf50db36b66e7b4974deda3527b67b931c"
    );
    assert_eq!(
        source["inputs"]["agoraTui"]["revision"],
        "b8d1074153d493aaef32249f7f81030523f31ac6"
    );
    for input in ["agoraTui", "chronosTui", "vellumTui"] {
        assert_eq!(source["inputs"][input]["workingTreeClean"], true);
    }
    for input in ["agora", "chronos", "vellum"] {
        assert_eq!(source["inputs"][input]["workingTreeClean"], false);
    }
    assert_eq!(source["coordinatedWorkingTreeQualified"], false);
    assert_eq!(source["ci"]["qualified"], false);
    assert_eq!(source["ci"]["blockers"].as_array().unwrap().len(), 2);
    assert_eq!(source["remoteReachabilityQualified"], false);
    assert_eq!(source["cratesIoPublished"], false);

    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("vellum-tui"));
    assert!(manifest.contains("vellum-sdk-client"));
    assert!(manifest.contains("chronos-tui"));
    assert!(manifest.contains("agora-tui"));
    assert!(!manifest.contains("agora-tui-local-client"));
    assert!(!manifest.contains("chronos-local"));
    for cli_owned in ["main.rs", "profile.rs", "terminal.rs", "hosted.rs"] {
        assert!(
            !root.join("src").join(cli_owned).exists(),
            "reusable package retains CLI-owned {cli_owned}"
        );
        assert!(
            root.join("cli/src").join(cli_owned).is_file(),
            "CLI package omits {cli_owned}"
        );
    }
    let cli_manifest =
        fs::read_to_string(root.join("cli/Cargo.toml")).expect("CLI package manifest");
    for required in [
        "name = \"workbench-tui-cli\"",
        "name = \"workbench-tui\"",
        "workbench-tui =",
        "agora-tui-local-client",
        "chronos-local",
        "chronos-standalone",
        "vellum-adapter-local-authentication",
        "vellum-daemon",
    ] {
        assert!(cli_manifest.contains(required), "CLI omits {required}");
    }
    let cli_entry = fs::read_to_string(root.join("cli/src/main.rs")).expect("CLI entry point");
    for peer_manifest in [
        "agora_tui::CAPABILITIES",
        "chronos_tui::CAPABILITIES",
        "vellum_tui::CAPABILITIES",
    ] {
        assert!(
            cli_entry.contains(peer_manifest),
            "CLI does not negotiate {peer_manifest}"
        );
    }
    for forbidden in [
        "vellum-engine",
        "vellum-host",
        "chronos-engine",
        "chronos-service",
        "agora-engine",
        "agora-service",
        "agora-local-daemon",
    ] {
        assert!(!manifest.contains(forbidden), "imports {forbidden}");
    }
    assert!(manifest.contains("publish = false"));
    assert!(manifest.contains("repository = \"https://github.com/gantbd/workbench-tui\""));
    assert!(manifest.contains("readme = \"README.md\""));
    assert!(manifest.contains("resolver = \"3\""));
    let architecture = fs::read_to_string(root.join("COMPONENT_ARCHITECTURE.md"))
        .expect("component architecture law");
    assert!(architecture.contains("Version: 2"));
    let qualifier =
        fs::read_to_string(root.join("scripts/qualify-source.py")).expect("source qualifier");
    assert!(qualifier.contains("\"agora\","));
    assert!(qualifier.contains("\"agora-tui\","));
    assert!(
        architecture.contains("github.com/gantbd/keystone/blob/main/COMPONENT_ARCHITECTURE.md")
    );
}

#[test]
fn live_gate_owns_both_real_service_lifecycles_without_publication() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/dual_service.sh")).expect("live gate");
    for required in [
        "chronos-service",
        "vellum-daemon",
        "workbench-tui",
        "--status",
        "--headless",
        "--shutdown",
        "\"deployment\":\"hosted\"",
        "hosted Chronos + Vellum",
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "live gate omits {required}");
    }
    for forbidden in ["cargo publish", "git push", "gh release", "curl "] {
        assert!(!gate.contains(forbidden), "live gate performs {forbidden}");
    }
}
