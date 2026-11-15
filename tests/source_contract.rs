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
        serde_json::json!([
            "agora", "chronos", "vellum", "moira", "daimon", "hermes", "aegis"
        ])
    );
    assert_eq!(
        source["inputs"]["vellumTui"]["revision"],
        "e5f2c3b5317091f392edb71cc3c8df94624791a3"
    );
    assert_eq!(
        source["inputs"]["chronosTui"]["revision"],
        "f0cf0653ef0636a2665c9349a9194a3e05f310ec"
    );
    assert_eq!(
        source["inputs"]["agoraTui"]["revision"],
        "a59c292c39cc17f545859f4c99aad96a7c2e0931"
    );
    assert_eq!(
        source["inputs"]["moiraTui"]["revision"],
        "62f3626d837e55d12e7381fbb82360455a0fa7a0"
    );
    assert_eq!(
        source["inputs"]["daimonTui"]["revision"],
        "b4e7412e0d4dcc9f8fce928b83eac41a84bcc6f9"
    );
    assert_eq!(
        source["inputs"]["hermesTui"]["revision"],
        "44154d406570ffe722b58a98dc9af11f2fc8d1db"
    );
    assert_eq!(
        source["inputs"]["aegisTui"]["revision"],
        "d432f5b7796ab91bd2a0febc0b29dc580c693644"
    );
    for input in [
        "agoraTui",
        "chronosTui",
        "vellumTui",
        "moiraTui",
        "daimonTui",
        "hermesTui",
        "aegisTui",
    ] {
        assert_eq!(source["inputs"][input]["workingTreeClean"], true);
    }
    for input in [
        "agora", "chronos", "vellum", "moira", "daimon", "hermes", "aegis",
    ] {
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
    assert!(manifest.contains("moira-tui"));
    assert!(manifest.contains("daimon-tui"));
    assert!(manifest.contains("hermes-tui"));
    assert!(manifest.contains("aegis-tui"));
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
        "moira-tui-local-client",
        "daimon-client",
        "hermes-tui-local-client",
        "aegis-tui-local-client",
        "chronos-local",
        "chronos-standalone",
        "vellum-adapter-local-authentication",
        "vellum-daemon",
    ] {
        assert!(cli_manifest.contains(required), "CLI omits {required}");
    }
    assert!(cli_manifest.contains("default = [\"native-chronos\"]"));
    assert!(cli_manifest.contains("chronos-standalone = {"));
    assert!(cli_manifest.contains("optional = true"));
    let agora_qualifier = fs::read_to_string(root.join("qualification/agora-host/Cargo.toml"))
        .expect("isolated Agora host qualifier");
    assert!(agora_qualifier.contains("[workspace]"));
    assert!(agora_qualifier.contains("workbench-tui-cli = { path = \"../../cli\" }"));
    assert!(agora_qualifier.contains("workbench-tui = { path = \"../..\" }"));
    assert!(agora_qualifier.contains("agora-tui ="));
    assert!(agora_qualifier.contains("agora-local-daemon"));
    let qualification_script = fs::read_to_string(root.join("qualification/agora-host/qualify.sh"))
        .expect("three-owner qualification command");
    assert!(qualification_script.contains("chronos/third_party/keystone-storage-0.0.1"));
    assert!(qualification_script.contains("keystone/crates/keystone-storage"));
    assert!(qualification_script.contains("patch.\\\"file://$vendor\\\""));
    assert!(manifest.contains("exclude = [\"qualification/agora-host\"]"));
    let cli_entry = fs::read_to_string(root.join("cli/src/main.rs")).expect("CLI entry point");
    let headless = fs::read_to_string(root.join("cli/src/headless.rs"))
        .expect("reusable headless composition");
    assert!(cli_entry.contains("observe_headless(HeadlessRequest"));
    assert!(!cli_entry.contains("fn observe_agora"));
    for peer_observer in ["observe_agora", "observe_chronos", "observe_vellum"] {
        assert!(
            headless.contains(peer_observer),
            "headless composition omits {peer_observer}"
        );
    }
    for peer_manifest in [
        "agora_tui::CAPABILITIES.headless",
        "chronos_tui::CAPABILITIES.headless",
        "vellum_tui::CAPABILITIES.headless",
    ] {
        assert!(
            headless.contains(peer_manifest),
            "headless composition does not negotiate {peer_manifest}"
        );
    }
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
        "attached Agora + Chronos + Vellum",
        "with attached Agora",
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "live gate omits {required}");
    }
    for forbidden in ["cargo publish", "git push", "gh release", "curl "] {
        assert!(!gate.contains(forbidden), "live gate performs {forbidden}");
    }
}

#[test]
fn agora_live_gate_runs_the_isolated_real_daemon_owner_qualification() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/agora_host.sh")).expect("Agora live gate");
    for required in ["qualification/agora-host/qualify.sh", "--target-dir"] {
        assert!(gate.contains(required), "Agora live gate omits {required}");
    }
}

#[test]
fn aegis_live_gate_reuses_the_real_daemon_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/aegis_attach.sh")).expect("Aegis attach gate");
    for required in [
        "real_daemon_fixture",
        "authority.observe.v1",
        "headless",
        "interactive",
        "epoch: 0",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Aegis attach gate omits {required}"
        );
    }
    for forbidden in ["cargo publish", "git push", "gh release", "curl "] {
        assert!(
            !gate.contains(forbidden),
            "Aegis attach gate performs {forbidden}"
        );
    }
}
