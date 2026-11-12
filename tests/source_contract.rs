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
        "5a3479f84d1a5a118199c807b945ca3d34dc9974"
    );
    assert_eq!(
        source["chronosWorkspaceRevision"],
        "03c468463fd4bda6af77414730c2f76405c3a73a"
    );
    assert_eq!(source["remoteReachabilityQualified"], false);
    assert_eq!(source["cratesIoPublished"], false);

    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("vellum-tui"));
    assert!(manifest.contains("chronos-tui"));
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
