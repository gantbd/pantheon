use serde_json::Value;
use std::{fs, path::Path};

#[test]
fn composition_source_is_exact_unpublished_and_has_no_engine_implementation_dependency() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source: Value = serde_json::from_str(
        &fs::read_to_string(root.join("release/source.json")).expect("source contract"),
    )
    .expect("source JSON");
    assert_eq!(
        source["vellumTuiRevision"],
        "bf4e4fe6e1631378e566408798df94c20ebea679"
    );
    assert_eq!(
        source["chronosWorkspaceRevision"],
        "03c468463fd4bda6af77414730c2f76405c3a73a"
    );
    assert_eq!(source["remoteReachabilityQualified"], false);
    assert_eq!(source["cratesIoPublished"], false);

    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("vellum-tui"));
    assert!(manifest.contains("vellum-sdk-client"));
    assert!(manifest.contains("chronos-tui"));
    assert!(manifest.contains("chronos-local"));
    for forbidden in [
        "vellum-engine",
        "vellum-host",
        "chronos-engine",
        "chronos-service",
    ] {
        assert!(!manifest.contains(forbidden), "imports {forbidden}");
    }
    assert!(manifest.contains("publish = false"));
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
        "--snapshot",
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
