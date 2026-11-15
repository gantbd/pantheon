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
        "af7b74fde8c77c8e89ed2df91361da497d7a5a00"
    );
    assert_eq!(
        source["inputs"]["chronosTui"]["revision"],
        "3352a9972d7f0d5a383c0d42845649872d5819a9"
    );
    assert_eq!(
        source["inputs"]["agoraTui"]["revision"],
        "f7103d85c8bed043fcf63924bc2251ce85b2d038"
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
    assert!(manifest.contains("agora-tui-local-client"));
    assert!(manifest.contains("chronos-local"));
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
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "live gate omits {required}");
    }
    for forbidden in ["cargo publish", "git push", "gh release", "curl "] {
        assert!(!gate.contains(forbidden), "live gate performs {forbidden}");
    }
}
