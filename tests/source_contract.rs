use serde_json::Value;
use std::collections::BTreeSet;
use std::{fs, path::Path};

#[test]
fn composition_status_is_closed_and_does_not_infer_stronger_cells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let status: Value = serde_json::from_str(
        &fs::read_to_string(root.join("composition-status.json"))
            .expect("composition status contract"),
    )
    .expect("composition status JSON");
    assert_eq!(status["schema"], 1);
    assert_eq!(status["owner"], "workbench-tui");
    assert_eq!(status["priorArt"], serde_json::json!(["axon", "engram"]));

    let states = status["states"]
        .as_array()
        .expect("closed state vocabulary")
        .iter()
        .map(|state| state.as_str().expect("state string"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        states,
        BTreeSet::from([
            "absent",
            "implemented",
            "planned",
            "qualified",
            "release-qualified",
        ])
    );

    let peers = status["peers"].as_object().expect("peer matrix");
    assert_eq!(
        peers.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "aegis",
            "agora",
            "chronos",
            "daimon",
            "grounding",
            "hermes",
            "moira",
            "vellum",
        ])
    );
    for peer in peers.values() {
        for topology in ["standalone", "attach", "host"] {
            for mode in ["interactive", "headless"] {
                let state = peer[topology][mode]
                    .as_str()
                    .unwrap_or_else(|| panic!("{topology}.{mode} state"));
                assert!(states.contains(state), "unknown state {state}");
            }
        }
        let mutation = peer["mutation"].as_str().expect("mutation state");
        assert!(
            states.contains(mutation),
            "unknown mutation state {mutation}"
        );

        for mode in ["interactive", "headless"] {
            if matches!(
                peer["attach"][mode].as_str(),
                Some("qualified" | "release-qualified")
            ) {
                assert!(
                    matches!(
                        peer["standalone"][mode].as_str(),
                        Some("qualified" | "release-qualified")
                    ),
                    "qualified attach must retain standalone parity"
                );
            }
        }
    }

    let capabilities = [
        ("aegis", aegis_tui::CAPABILITIES.headless[0]),
        ("agora", agora_tui::CAPABILITIES.headless[0]),
        ("chronos", chronos_tui::CAPABILITIES.headless[0]),
        ("daimon", daimon_tui::CAPABILITIES.headless[0]),
        ("hermes", hermes_tui::CAPABILITIES.headless[0]),
        ("grounding", grounding_tui::CAPABILITIES.headless[0]),
        ("moira", moira_tui::CAPABILITIES.headless[0]),
        ("vellum", vellum_tui::CAPABILITIES.headless[0]),
    ];
    for (peer, capability) in capabilities {
        assert_eq!(peers[peer]["readCapability"], capability);
    }
    assert_eq!(peers["grounding"]["presentation"], "grounding-tui");
}

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
            "agora",
            "chronos",
            "vellum",
            "moira",
            "daimon",
            "hermes",
            "aegis",
            "grounding"
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
        "608b13f64be9714a98d288c66694e470c7785ecc"
    );
    assert_eq!(
        source["inputs"]["daimonTui"]["revision"],
        "b4e7412e0d4dcc9f8fce928b83eac41a84bcc6f9"
    );
    assert_eq!(
        source["inputs"]["hermesTui"]["revision"],
        "d4f92b194b7d335386dae80b7a8304a006acc8db"
    );
    assert_eq!(
        source["inputs"]["aegisTui"]["revision"],
        "d432f5b7796ab91bd2a0febc0b29dc580c693644"
    );
    assert_eq!(
        source["inputs"]["groundingTui"]["revision"],
        "11d75c218e2e9614ae6bef6bde05b9eb96222c62"
    );
    for input in [
        "agoraTui",
        "chronosTui",
        "vellumTui",
        "moiraTui",
        "daimonTui",
        "hermesTui",
        "aegisTui",
        "groundingTui",
    ] {
        assert_eq!(source["inputs"][input]["workingTreeClean"], true);
    }
    for input in [
        "agora", "chronos", "vellum", "moira", "daimon", "hermes", "aegis", "dendrite",
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
    assert!(manifest.contains("grounding-tui"));
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
        "grounding-tui-local-client",
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

#[test]
fn grounding_live_gate_uses_typed_dendrite_boundary_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/grounding_attach.sh"))
        .expect("Grounding attach gate");
    for required in [
        "grounding_fixture.exs",
        "grounding.observe.v1",
        "headless",
        "interactive",
        "fixture/sample",
        "healthy Grounding evidence survives failed Aegis peer",
        "connect_refused",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Grounding live gate omits {required}"
        );
    }
    for forbidden in [
        "tools/call",
        "jsonrpc",
        "cargo publish",
        "git push",
        "gh release",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Grounding live gate performs {forbidden}"
        );
    }
}

#[test]
fn hermes_live_gate_carries_the_opaque_stream_selector_through_workbench() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/hermes_attach.sh")).expect("Hermes attach gate");
    for required in [
        "ledger.observe.v1",
        "05050505050505050505050505050505",
        "retained_events",
    ] {
        assert!(
            gate.contains(required),
            "Hermes attach gate omits {required}"
        );
    }
    for forbidden in ["AppendClient", "ConsumerClient", "OperatorClient"] {
        assert!(
            !gate.contains(forbidden),
            "Hermes attach gate acquires {forbidden}"
        );
    }
}

#[test]
fn moira_live_gate_reuses_selected_saga_evidence_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/moira_attach.sh")).expect("Moira attach gate");
    for required in [
        "real_daemon_fixture",
        "sagas.inspect.v1",
        "headless",
        "interactive",
        "FencedUncertain",
        "evidence page: 1",
        "operator receipts: 1",
        "authority 9/10",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Moira attach gate omits {required}"
        );
    }
    for forbidden in [
        "SagaOperatorClient",
        "SagaCommandV1",
        "cargo publish",
        "git push",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Moira attach gate acquires or performs {forbidden}"
        );
    }
}

#[test]
fn operational_live_gate_composes_all_read_only_peer_daemons() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/operational_attach.sh"))
        .expect("operational attach gate");
    for required in [
        "moira-tui-cli",
        "daimon-tui-cli",
        "hermes-tui-cli",
        "aegis-tui-cli",
        "grounding_fixture.exs",
        "sagas.inspect.v1",
        "fenced",
        "agents.presence.v1",
        "ledger.observe.v1",
        "authority.observe.v1",
        "grounding.observe.v1",
        "configured_stream",
        "05050505050505050505050505050505",
        "one Workbench headless composition",
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "operational gate omits {required}");
    }
    for forbidden in ["cargo publish", "git push", "gh release", "curl "] {
        assert!(
            !gate.contains(forbidden),
            "operational gate performs {forbidden}"
        );
    }
}
