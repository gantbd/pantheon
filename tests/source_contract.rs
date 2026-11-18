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
    assert_eq!(status["schema"], 2);
    assert_eq!(status["owner"], "pantheon");
    assert_eq!(status["priorArt"], serde_json::json!(["axon", "engram"]));
    assert_eq!(
        status["capabilityPlanes"],
        serde_json::json!(["observation", "operator", "control", "management"])
    );

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
            "not-applicable",
            "planned",
            "qualified",
            "release-qualified",
        ])
    );

    let plane_qualification = status["planeQualification"]
        .as_object()
        .expect("plane qualification matrix");
    assert_eq!(
        plane_qualification
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["control", "management", "observation", "operator"])
    );
    for (plane, qualification) in plane_qualification {
        let state = qualification["state"]
            .as_str()
            .unwrap_or_else(|| panic!("{plane} qualification state"));
        assert!(states.contains(state), "unknown {plane} state {state}");
        let evidence = qualification["evidence"]
            .as_array()
            .unwrap_or_else(|| panic!("{plane} qualification evidence"));
        assert!(!evidence.is_empty(), "{plane} has no executable evidence");
        for path in evidence {
            let path = path.as_str().expect("qualification evidence path");
            assert!(root.join(path).is_file(), "missing {plane} evidence {path}");
        }
    }
    assert_eq!(plane_qualification["management"]["state"], "qualified");
    assert_eq!(
        plane_qualification["management"]["profile"],
        "single-user-local-stdio"
    );
    assert!(
        !plane_qualification["management"]["remaining"]
            .as_array()
            .expect("management remaining work")
            .is_empty()
    );
    assert_eq!(status["releaseQualification"]["state"], "qualified");
    assert_eq!(
        status["releaseQualification"]["profile"],
        "unsigned-local-aarch64-apple-darwin"
    );
    assert!(
        root.join(
            status["releaseQualification"]["sourceContract"]
                .as_str()
                .expect("release source contract")
        )
        .is_file()
    );
    for evidence in status["releaseQualification"]["evidence"]
        .as_array()
        .expect("release evidence")
    {
        assert!(
            root.join(evidence.as_str().expect("release evidence path"))
                .is_file()
        );
    }
    assert_eq!(
        status["releaseQualification"]["broaderDistribution"]["state"],
        "planned"
    );
    assert!(
        !status["releaseQualification"]["broaderDistribution"]["remaining"]
            .as_array()
            .expect("broader release remaining work")
            .is_empty()
    );

    let peers = status["peers"].as_object().expect("peer matrix");
    assert_eq!(
        peers.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "aegis", "agora", "chronos", "daimon", "tellus", "hermes", "moira", "vellum",
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
            let mut identifiers = BTreeSet::new();
            let routes = peer["pantheonRoutes"][mode]
                .as_array()
                .unwrap_or_else(|| panic!("missing Pantheon {mode} route catalog"));
            assert!(!routes.is_empty(), "empty Pantheon {mode} route catalog");
            for route in routes {
                let identifier = route["id"].as_str().expect("route capability id");
                assert!(identifiers.insert(identifier), "duplicate route capability");
                assert!(
                    status["capabilityPlanes"]
                        .as_array()
                        .expect("capability planes")
                        .iter()
                        .any(|plane| plane.as_str() == route["plane"].as_str()),
                    "unknown route capability plane"
                );
                assert!(
                    matches!(route["state"].as_str(), Some("implemented" | "qualified")),
                    "Pantheon route must be implemented or qualified"
                );
            }
            assert!(
                identifiers.contains(peer["readCapability"].as_str().expect("read capability")),
                "Pantheon route omits primary read capability"
            );
        }
        let non_applicable = peer.get("nonApplicable").and_then(Value::as_object);
        for topology in ["standalone", "attach", "host"] {
            let selected = ["interactive", "headless"]
                .iter()
                .any(|mode| peer[topology][*mode].as_str() == Some("not-applicable"));
            if selected {
                assert!(
                    non_applicable
                        .and_then(|reasons| reasons.get(topology))
                        .and_then(Value::as_str)
                        .is_some_and(|reason| !reason.is_empty()),
                    "not-applicable {topology} requires an owner rationale"
                );
            }
        }
        if mutation == "not-applicable" {
            assert!(
                non_applicable
                    .and_then(|reasons| reasons.get("mutation"))
                    .and_then(Value::as_str)
                    .is_some_and(|reason| !reason.is_empty()),
                "not-applicable mutation requires an owner rationale"
            );
        }
        if let Some(capabilities) = peer.get("mutationCapabilities") {
            for (capability, state) in capabilities
                .as_object()
                .expect("mutation capability matrix")
            {
                assert!(!capability.is_empty(), "empty mutation capability");
                let state = state.as_str().expect("mutation capability state");
                assert!(
                    states.contains(state),
                    "unknown mutation capability state {state}"
                );
            }
        }
        if let Some(capabilities) = peer.get("disclosureCapabilities") {
            for (capability, state) in capabilities
                .as_object()
                .expect("disclosure capability matrix")
            {
                assert!(!capability.is_empty(), "empty disclosure capability");
                let state = state.as_str().expect("disclosure capability state");
                assert!(states.contains(state), "unknown disclosure state {state}");
            }
        }
        if let Some(capabilities) = peer.get("uncomposedPeerCapabilities") {
            for (capability, classification) in capabilities
                .as_object()
                .expect("uncomposed peer capability matrix")
            {
                assert!(!capability.is_empty(), "empty uncomposed capability");
                assert_eq!(classification["peer"], "qualified");
                assert_eq!(classification["pantheon"], "absent");
            }
        }

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

    assert_eq!(
        peers["aegis"]["mutationCapabilities"]["authority.revoke-epoch.v1"],
        "qualified"
    );
    assert_eq!(
        peers["aegis"]["mutationCapabilities"]["authority.attenuate-authorize-once.v1"],
        "qualified"
    );
    assert_eq!(
        peers["aegis"]["mutationCapabilities"]["aegis.admin-session.v1"],
        "qualified"
    );
    assert!(
        peers["aegis"]["pantheonRoutes"]["interactive"]
            .as_array()
            .expect("Aegis interactive routes")
            .iter()
            .any(|route| route["id"] == "aegis.admin-session.v1" && route["plane"] == "management")
    );
    assert_eq!(
        peers["hermes"]["disclosureCapabilities"]["ledger.events.disclose.v1"],
        "qualified"
    );
    assert_eq!(
        peers["hermes"]["interactiveControlComposition"]["ledger.events.disclose.v1"]["state"],
        "composed"
    );
    assert_eq!(
        peers["hermes"]["interactiveControlComposition"]["ledger.compact.v1"]["state"],
        "composed"
    );
    assert_eq!(
        peers["hermes"]["producerIntegrations"]["moira.workflow-event.v1"]["state"],
        "qualified"
    );
    assert_eq!(
        peers["hermes"]["producerIntegrations"]["agora.minimized-wake.v1"]["state"],
        "qualified"
    );
    assert_eq!(
        peers["hermes"]["producerIntegrations"]["agora.minimized-wake.v1"]["owner"],
        "agora-hermes-adapter"
    );
    assert_eq!(
        peers["daimon"]["mutationCapabilities"]["agents.session.prompt.v1"],
        "qualified"
    );
    assert_eq!(
        peers["daimon"]["mutationCapabilities"]["agents.session.permission.v1"],
        "qualified"
    );
    assert_eq!(peers["moira"]["host"]["interactive"], "qualified");
    assert_eq!(peers["moira"]["host"]["headless"], "qualified");
    assert_eq!(peers["moira"]["client"]["ownership"], "peer");
    assert_eq!(
        peers["moira"]["uncomposedPeerCapabilities"]["deliberation.inspect.v1"]["pantheon"],
        "absent"
    );

    let capabilities = [
        ("aegis", aegis_tui::CAPABILITIES.headless[0]),
        ("agora", agora_tui::CAPABILITIES.headless[0]),
        ("chronos", chronos_tui::CAPABILITIES.headless[0]),
        ("daimon", daimon_tui::CAPABILITIES.headless[0]),
        ("hermes", hermes_tui::CAPABILITIES.headless[0]),
        ("tellus", tellus::CAPABILITIES.headless[0]),
        ("moira", moira_tui::CAPABILITIES.headless[0]),
        ("vellum", vellum_tui::CAPABILITIES.headless[0]),
    ];
    for (peer, capability) in capabilities {
        assert_eq!(peers[peer]["readCapability"], capability);
    }
    assert_eq!(peers["tellus"]["presentation"], "tellus");
}

#[test]
fn hermes_interactive_disclosure_reuses_owner_sheet_and_keeps_content_out_of_telemetry() {
    let terminal = include_str!("../cli/src/terminal.rs");
    let disclosure = include_str!("../../hermes-tui/src/disclosure_control.rs");
    assert!(terminal.contains("hermes_tui::DisclosureComponent"));
    assert!(terminal.contains("peer_read_receipt"));
    assert!(terminal.contains("ledger.events.disclose.v1"));
    assert!(!terminal.contains("hermes_disclosure_command::disclose"));
    assert!(!terminal.contains("payload_preview_hex"));
    assert!(disclosure.contains("PREVIEW_BYTES: usize = 256"));
    assert!(disclosure.contains("DISCLOSURE_BATCH_LIMIT: u64 = 16"));
    assert!(disclosure.contains("selected_event"));
}

#[test]
fn aegis_interactive_management_keeps_handles_private_and_bounded() {
    let terminal = include_str!("../cli/src/terminal.rs");
    let managed = include_str!("../../aegis-tui/src/managed_control.rs");
    assert!(terminal.contains("aegis_tui::ManagedAuthorityComponent"));
    assert!(terminal.contains("aegis.authority.manage"));
    for required in [
        "ManagedSessionV1",
        "max_grants: 8",
        "max_scopes_per_grant: 16",
        "ManagedAuthorityDisposition::Fenced",
        "exact confirmation",
    ] {
        assert!(
            managed.contains(required),
            "managed Aegis sheet omits {required}"
        );
    }
    for forbidden in ["GrantHandleV1", "CommandV1", "AdminClient"] {
        assert!(
            !managed.contains(forbidden),
            "managed Aegis sheet exposes owner detail {forbidden}"
        );
    }
}

#[test]
fn hermes_interactive_compaction_reuses_owner_sheet_and_exact_range() {
    let terminal = include_str!("../cli/src/terminal.rs");
    let compaction = include_str!("../../hermes-tui/src/compaction_control.rs");
    assert!(terminal.contains("hermes_tui::CompactionComponent"));
    assert!(terminal.contains("ledger.compact.v1"));
    assert!(terminal.contains("hermes_compaction_post_receipt_refresh"));
    assert!(!terminal.contains("hermes_operator_command::compact"));
    for required in [
        "PreviousPage",
        "NextPage",
        "expected_retained_from",
        "expected_published_edge",
        "OutcomeUnknown",
    ] {
        assert!(
            compaction.contains(required),
            "compaction sheet omits {required}"
        );
    }
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
            "agora", "chronos", "vellum", "moira", "daimon", "hermes", "aegis", "tellus"
        ])
    );
    assert_eq!(
        source["inputs"]["vellumTui"]["revision"],
        "3b4b2bb5eea86b0bf1a0a95b98d59ab77a51398c"
    );
    assert_eq!(
        source["inputs"]["chronosTui"]["revision"],
        "76cf8cf2a45361b065adf262cdffe78933edb7c7"
    );
    assert_eq!(
        source["inputs"]["chronos"]["revision"],
        "43fdf6f33fa526899639f03cc32e398bcea86010"
    );
    assert_eq!(
        source["inputs"]["agoraTui"]["revision"],
        "36c6c624360de88ee24c0c9035d5ad36bb62e9a5"
    );
    assert_eq!(
        source["inputs"]["moiraTui"]["revision"],
        "335ca2f1bbd2002fdb7b538062bb28b717accd54"
    );
    assert_eq!(
        source["inputs"]["daimonTui"]["revision"],
        "c6888396d2c044c290c6f688cb5c1f613464f0cc"
    );
    assert_eq!(
        source["inputs"]["hermesTui"]["revision"],
        "431f6b565b34398c76894f68a443c01b5ae7f9d1"
    );
    assert_eq!(
        source["inputs"]["hermes"]["revision"],
        "e666416071d1b5b6f4bdb7669a3984ef6d8a3162"
    );
    assert_eq!(
        source["inputs"]["aegisTui"]["revision"],
        "3433dd2f306a1c9b8af4276b88d89c546eab2d30"
    );
    assert_eq!(
        source["inputs"]["aegis"]["revision"],
        "0446e4fcf7412be9e0313523235d4e674d2fdd94"
    );
    assert_eq!(
        source["inputs"]["moira"]["revision"],
        "a43991b430a3a38961c6e1a40f2e5aca8d733696"
    );
    assert_eq!(
        source["inputs"]["daimon"]["revision"],
        "51b4dcb00480f017adca2e324daf6f6c96c21efb"
    );
    assert_eq!(
        source["inputs"]["dendrite"]["revision"],
        "0600534410aaf6b6ed7d6b441a9c02ccae224496"
    );
    assert_eq!(
        source["inputs"]["vellum"]["revision"],
        "81881c01517af77e3bdc25d7998e74bb4087e882"
    );
    assert_eq!(
        source["inputs"]["agora"]["revision"],
        "9ab9e0f2c467c3b6501e08c44f10f679b8ed6cd8"
    );
    assert_eq!(
        source["inputs"]["tellusTui"]["revision"],
        "55e125c794c4037e355ab2781edca8ea5c34ff35"
    );
    assert_eq!(
        source["inputs"]["keystone"]["revision"],
        "2e4faf10ec0a70805aa1db1555f93c77fcf0a1e4"
    );
    assert_eq!(
        source["inputs"]["keystoneGit"]["revision"],
        "2e4faf10ec0a70805aa1db1555f93c77fcf0a1e4"
    );
    for input in [
        "agoraTui",
        "agora",
        "chronosTui",
        "chronos",
        "vellumTui",
        "daimonTui",
        "hermes",
        "hermesTui",
        "aegisTui",
        "aegis",
        "tellusTui",
        "keystone",
        "keystoneGit",
        "vellum",
        "daimon",
        "dendrite",
        "moira",
        "moiraTui",
    ] {
        assert_eq!(source["inputs"][input]["workingTreeClean"], true);
    }
    assert_eq!(source["coordinatedWorkingTreeQualified"], true);
    assert_eq!(source["localRelease"]["qualified"], true);
    assert_eq!(source["localRelease"]["target"], "aarch64-apple-darwin");
    assert_eq!(source["localRelease"]["profileSchema"], 1);
    assert_eq!(source["ci"]["qualified"], true);
    assert!(source["ci"]["blockers"].as_array().unwrap().is_empty());
    assert_eq!(source["remoteReachabilityQualified"], true);
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
    assert!(manifest.contains("tellus"));
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
        "name = \"pantheon-cli\"",
        "name = \"pantheon\"",
        "pantheon =",
        "agora-local-client",
        "moira-tui-local-client",
        "daimon-client",
        "hermes-local-client",
        "aegis-local-client",
        "aegis-admin-session",
        "tellus-local-client",
        "chronos-local",
        "chronos-standalone",
        "vellum-adapter-local-authentication",
        "vellum-daemon",
    ] {
        assert!(cli_manifest.contains(required), "CLI omits {required}");
    }
    assert!(!cli_manifest.contains("hermes-tui-local-client"));
    assert!(!cli_manifest.contains("agora-tui-local-client"));
    assert!(!cli_manifest.contains("aegis-tui-local-client"));
    assert!(cli_manifest.contains("default = [\"native-chronos\"]"));
    assert!(cli_manifest.contains("chronos-standalone = {"));
    assert!(cli_manifest.contains("optional = true"));
    let agora_qualifier = fs::read_to_string(root.join("qualification/agora-host/Cargo.toml"))
        .expect("isolated Agora host qualifier");
    assert!(agora_qualifier.contains("[workspace]"));
    assert!(agora_qualifier.contains("pantheon-cli = { path = \"../../cli\" }"));
    assert!(agora_qualifier.contains("pantheon = { path = \"../..\" }"));
    assert!(agora_qualifier.contains("agora-tui ="));
    assert!(agora_qualifier.contains("agora-local-daemon"));
    assert!(agora_qualifier.contains("agora-local-client"));
    assert!(agora_qualifier.contains("agora-scheduled-publication-port"));
    let qualification_script = fs::read_to_string(root.join("qualification/agora-host/qualify.sh"))
        .expect("three-owner qualification command");
    assert!(qualification_script.contains("chronos/third_party/keystone-storage-0.0.1"));
    assert!(qualification_script.contains("keystone/crates/keystone-storage"));
    assert!(qualification_script.contains("patch.\\\"file://$vendor\\\""));
    assert!(manifest.contains("exclude = [\"qualification/agora-host\"]"));
    let cli_entry = fs::read_to_string(root.join("cli/src/main.rs")).expect("CLI entry point");
    let headless = fs::read_to_string(root.join("cli/src/headless.rs"))
        .expect("reusable headless composition");
    let capability_routes = fs::read_to_string(root.join("cli/src/capabilities.rs"))
        .expect("Pantheon route capability catalog");
    assert!(cli_entry.contains("fn headless_request("));
    assert!(cli_entry.contains("observe_headless(headless_request(&options, operation_id))"));
    assert!(
        cli_entry.contains("observe_headless(headless_request(&options, requested.to_owned()))")
    );
    assert!(!cli_entry.contains("fn observe_agora"));
    for peer_observer in ["observe_agora", "observe_chronos", "observe_vellum"] {
        assert!(
            headless.contains(peer_observer),
            "headless composition omits {peer_observer}"
        );
    }
    assert!(cli_entry.contains("capabilities::negotiate_observation"));
    assert!(cli_entry.contains("not_exercised_profile_requirements"));
    assert!(headless.contains("not_exercised_profile_requirements"));
    assert!(
        !headless.contains("capabilities::negotiate("),
        "observer must not independently redefine already validated workspace requirements"
    );
    for peer_route in [
        "capabilities::agora",
        "capabilities::chronos",
        "capabilities::vellum",
    ] {
        assert!(
            cli_entry.contains(peer_route),
            "CLI does not negotiate {peer_route}"
        );
    }
    for required in [
        "GUARDED_CONTROL_CAPABILITIES",
        "topology.attach.v1",
        "topology.host.v1",
        "tasks.mutate.v1",
        "scheduling.mutate.v5",
        "workspace.durable-save.execute.v1",
        "agent.prompt.submit.v1",
    ] {
        assert!(
            capability_routes.contains(required),
            "route capability catalog omits {required}"
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
    assert!(manifest.contains("repository = \"https://github.com/gantbd/pantheon\""));
    assert!(manifest.contains("readme = \"README.md\""));
    assert!(manifest.contains("resolver = \"3\""));
    let architecture = fs::read_to_string(root.join("COMPONENT_ARCHITECTURE.md"))
        .expect("component architecture law");
    assert!(architecture.contains("Version: 2"));
    let qualifier =
        fs::read_to_string(root.join("scripts/qualify-source.py")).expect("source qualifier");
    assert!(qualifier.contains("\"agora\","));
    assert!(qualifier.contains("\"agora-tui\","));
    assert!(qualifier.contains("\"keystone\","));
    assert!(qualifier.contains("prepare_cargo_source_closure("));
    assert!(qualifier.contains("f\"{keystone_revision}:refs/commit/{keystone_revision}\""));
    assert!(qualifier.contains("environment[\"CARGO_NET_OFFLINE\"] = \"true\""));
    assert!(qualifier.contains("run_live_evidence(stage, environment)"));
    assert!(qualifier.contains("prepare_live_dependency_closure(stage, environment)"));
    assert!(qualifier.contains("[\"mix\", \"deps.get\", \"--only\", \"test\"]"));
    assert!(qualifier.contains("validate_direct_path_closure(directory, stage)"));
    assert!(qualifier.contains("candidate source closure omits direct dependencies"));
    assert!(qualifier.contains("status[\"peers\"][peer][\"evidence\"]"));
    for required in [
        "--local-release",
        "ALLOWED_LICENSE_EXPRESSIONS",
        "THIRD_PARTY_LICENSES.tsv",
        "THIRD_PARTY_NOTICES.tsv",
        "no distributable legal text",
        "SHA256SUMS",
        "write_deterministic_tar_gz",
        "os.replace(replacement, deployed_binary)",
        "unsupported[\"version\"] = release[\"profileSchema\"] + 1",
        "artifact replacement modified the operator-owned profile",
    ] {
        assert!(
            qualifier.contains(required),
            "local release qualifier omits {required}"
        );
    }
    let local_release =
        fs::read_to_string(root.join("docs/local-release.md")).expect("local release policy");
    for required in [
        "aarch64-apple-darwin",
        "unsigned",
        "initial supported Pantheon artifact",
        "owns no durable engine",
        "unknown schema is refused",
    ] {
        assert!(
            local_release.contains(required),
            "local release policy omits {required}"
        );
    }
    assert!(!qualifier.contains("[\"sh\", \"tests/live/dual_service.sh\"]"));
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
        "pantheon",
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
fn chronos_mutation_gate_uses_the_peer_component_and_proves_durable_reopen() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/chronos_mutation.sh"))
        .expect("Chronos mutation live gate");
    for required in [
        "chronos-service",
        "pantheon",
        "--host-chronos",
        "--chronos-allow-writes",
        "--chronos-scheduling",
        "--chronos-cancel-receipts",
        "Task batch",
        "Review 3 exact task(s)",
        "retry=idempotent",
        "workbench-secondary",
        "Observed 3 tasks",
        "Set concurrency capacity to 3?",
        "Set not-before to 100000.000000000 for 3 exact tasks",
        "Set task pause to true for 3 exact tasks",
        "Applied 3/3 independently settled task eligibility controls",
        "chronos.schedule.eligibility-batch",
        "Set global pause to true?",
        "Cancel 3 marked tasks?",
        "Applied 3/3 independently receipted cancellations",
        "chronos.task.cancel-batch",
        "reopened task was not cancelled",
        "reopened second task was not cancelled",
        "reopened third task was not cancelled",
        "reopened eligibility did not match",
        "reopened sibling eligibility did not match",
        "--shutdown",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Chronos mutation gate omits {required}"
        );
    }
    for forbidden in [
        "chronos_engine::",
        "chronos_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Chronos mutation gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn chronos_batch_composer_reuses_one_owner_atomic_graph() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let component = fs::read_to_string(root.join("../chronos-tui/src/component.rs"))
        .expect("Chronos component source");
    let model = fs::read_to_string(root.join("../chronos/host/chronos-model/src/lib.rs"))
        .expect("Chronos model source");
    let terminal =
        fs::read_to_string(root.join("cli/src/terminal.rs")).expect("Pantheon terminal source");
    for required in [
        "MAX_SUBMIT_BATCH",
        "Input::NextItem",
        "Action::SubmitGraph(tasks)",
        "TaskPriority",
        "TaskDependencies",
        "TaskCapabilities",
        "TaskMaxAttempts",
        "RetrySafety::ConnectorIdempotent",
    ] {
        assert!(
            component.contains(required),
            "batch composer omits {required}"
        );
    }
    for required in [
        "Action::SubmitBatch(ids)",
        "Action::SubmitGraph(tasks)",
        "Command::SubmitGraph",
        ".collect()",
    ] {
        assert!(
            model.contains(required),
            "atomic model route omits {required}"
        );
    }
    for required in [
        "chronos.task.submit-batch",
        "TaskMutationDisposition::Applied",
        "TaskMutationDisposition::OutcomeUnknown",
        "Hub::mutation_recovery_required",
    ] {
        assert!(
            terminal.contains(required),
            "end-to-end Chronos instrumentation omits {required}"
        );
    }
}

#[test]
fn agora_mutation_gate_reuses_the_owner_guard_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/agora_mutation.sh"))
        .expect("Agora mutation live gate");
    for required in [
        "real_daemon_fixture",
        "agora-create-room",
        "rooms.create.v2",
        "confirmation_refused",
        "Stale Room",
        "Delivery Room",
        "revision\": 4",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Agora mutation gate omits {required}"
        );
    }
    let shell = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(shell.contains("agora_client::room_command"));
    for forbidden in [
        "agora_engine::",
        "agora_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Agora mutation gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn agora_hosted_gate_sends_real_work_on_the_owner_client_session() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/agora_hosted.sh"))
        .expect("hosted Agora live gate");
    for required in [
        "Compose an Agora message",
        "Pantheon delivered serious Agora work",
        "Agora accepted message #",
        "Inspect TUI instrumentation",
        "agora.message.send",
        "schema=2",
        "participant_count=2",
        "participant_1_id=local-agent",
        "participant_1_kind=agent",
        "peers[\"agora\"][\"revision\"] == 4",
    ] {
        assert!(
            gate.contains(required),
            "hosted Agora gate omits {required}"
        );
    }
    let terminal =
        fs::read_to_string(root.join("cli/src/terminal.rs")).expect("Pantheon terminal source");
    for required in [
        "agora.message.compose",
        "ComposeMessageComponent::new_with_handle",
        "control_handle()",
        "peer_receipt",
        "drop(agora_message)",
        "drop(agora_create)",
        "drop(agora)",
    ] {
        assert!(
            terminal.contains(required),
            "Agora workbench composition omits {required}"
        );
    }
}

#[test]
fn vellum_mutation_gate_reuses_the_editor_and_preserves_service_authority() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/vellum_mutation.sh"))
        .expect("Vellum mutation live gate");
    for required in [
        "workspace.edit.v1",
        "workspace.save.v1",
        "--vellum-allow-edit",
        "--vellum-allow-save",
        "focus=Vellum",
        "Dirty",
        "Clean",
        "Refused",
        "denied Pantheon Vellum save changed source bytes",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Vellum mutation gate omits {required}"
        );
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    assert!(terminal.contains("ServiceEditor"));
    assert!(terminal.contains("translate_service_input"));
    for forbidden in [
        "vellum_engine::",
        "vellum_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Vellum mutation gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn vellum_durable_save_gate_reuses_the_peer_guard_and_exact_historical_receipt() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/vellum_durable_save.sh"))
        .expect("Vellum durable-save live gate");
    for required in [
        "durable-save-execute",
        "vellum-durable-save lookup",
        "workspace.durable-save.execute.v1",
        "workspace.durable-save.lookup.v1",
        "--credential",
        "--allow-save",
        "--locked",
        "--offline",
        "historical lookup reused Vellum public authority",
    ] {
        assert!(
            gate.contains(required),
            "Vellum durable-save gate omits {required}"
        );
    }
    let shell = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(shell.contains("run_durable_save_v1"));
    for forbidden in [
        "vellum_engine::",
        "vellum_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Vellum durable-save gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn vellum_batch_gate_proves_independently_receipted_n_document_work() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/vellum_batch.sh"))
        .expect("Vellum batch live gate");
    for required in [
        "three.txt",
        "3 Vellum document(s) selected for bounded work",
        "Vellum durable-save workset",
        "3 exact documents",
        "Vellum run settled: 3 durable",
        "--provision-credential",
        "workspace.durable-save.execute.v1",
        "workspace.durable-save.lookup.v1",
        "exact persisted bytes",
    ] {
        assert!(
            gate.contains(required),
            "Vellum batch gate omits {required}"
        );
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    for required in [
        "DurableSaveBatchComponent",
        "prepare_selected_save_targets",
        "child_receipts",
    ] {
        assert!(
            terminal.contains(required),
            "Vellum batch host omits {required}"
        );
    }
    for forbidden in [
        "vellum_engine::",
        "vellum_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Vellum batch gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn vellum_atomic_replace_gate_proves_one_real_n_document_publication_and_trace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/vellum_atomic_replace.sh"))
        .expect("Vellum atomic replace live gate");
    for required in [
        "three.txt",
        "3 Vellum document(s) selected for bounded work",
        "literal-that-is-not-present",
        "no literal matches across 3 marked document(s)",
        "edit the query or close this sheet",
        "4 matches across 3 exact documents",
        "published atomically across 3 changed documents",
        "owner_atomic_receipt",
        "Vellum run settled: 3 durable",
        "omega\nline omega\nline one",
        "bracketed multiline paste",
        "one owner-atomic edit",
    ] {
        assert!(
            gate.contains(required),
            "Vellum atomic replace gate omits {required}"
        );
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    for required in [
        "AtomicReplaceComponent",
        "vellum.atomic-replace-selected",
        "owner_atomic_receipt",
        "vellum.atomic-replace.commit",
        "EnableBracketedPaste",
        "DisableBracketedPaste",
    ] {
        assert!(
            terminal.contains(required),
            "Vellum atomic replace host omits {required}"
        );
    }
    for forbidden in [
        "vellum_engine::",
        "vellum_service::",
        "cargo publish",
        "git push",
        "gh release",
        "curl ",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Vellum atomic replace gate imports or performs {forbidden}"
        );
    }
}

#[test]
fn vellum_request_rotation_gate_crosses_the_old_replay_capacity_ceiling() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/vellum_request_rotation.sh"))
        .expect("Vellum request rotation live gate");
    for required in [
        "x\" * 190",
        "y\" * 190",
        "vellum.client.renew-identity",
        "durable source mismatch",
        "two bounded client identities",
    ] {
        assert!(
            gate.contains(required),
            "Vellum request rotation gate omits {required}"
        );
    }
    for forbidden in ["git push", "cargo publish", "gh release", "curl "] {
        assert!(
            !gate.contains(forbidden),
            "Vellum request rotation gate performs {forbidden}"
        );
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
        "authority.revoke-epoch.v1",
        "authority.attenuate-authorize-once.v1",
        "aegis-manage-session",
        "ISSUE AEGIS OP",
        "ATTENUATE AEGIS OP",
        "AUTHORIZE AEGIS OP",
        "\"handle\" not in rendered",
        "--package aegis-daemon",
        "values[0][\"grants\"] == []",
        "kill -TERM",
        "REVOKE AEGIS EPOCH 0",
        "AUTHORIZE AEGIS EPOCH 0 hermes",
        "\"grant\" not in value",
        "confirmation_refused",
        "stale_epoch",
        "headless",
        "interactive",
        "epoch: 0",
        "active_grants\"] == 6",
        "page: 2",
        "grant #5",
        "decision #5",
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
fn tellus_live_gate_uses_typed_dendrite_boundary_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/tellus_attach.sh")).expect("Tellus attach gate");
    for required in [
        "tellus_fixture.exs",
        "tellus.observe.v1",
        "headless",
        "interactive",
        "fixture/sample",
        "healthy Tellus evidence survives failed Aegis peer",
        "connect_refused",
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "Tellus live gate omits {required}");
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
            "Tellus live gate performs {forbidden}"
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
        "ledger.events.disclose.v1",
        "DISCLOSE HERMES STREAM 05050505050505050505050505050505 FROM 0 LIMIT 1",
        "confirmation_refused",
        "05050505050505050505050505050505",
        "06060606060606060606060606060606",
        "retained_events",
        "configured_cursor",
        "next_sequence",
        "backlog",
        "consumer_daemon_fixture",
        "hermes-acknowledge",
        "ledger.acknowledge.v1",
        "previous_next_sequence",
        "ACKNOWLEDGE HERMES CONSUMER",
        "hermes-compact",
        "ledger.compact.v1",
        "COMPACT HERMES STREAM",
        "removed",
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
fn hermes_interactive_acknowledgement_reuses_the_owner_sheet() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    let shell = fs::read_to_string(root.join("src/lib.rs")).expect("shell source");
    for required in [
        "hermes_tui::AcknowledgeComponent",
        "next_acknowledgement",
        "ledger.acknowledge.v1",
        "peer_receipt",
        "ledger.acknowledge.inspect",
    ] {
        assert!(
            terminal.contains(required),
            "Hermes composition omits {required}"
        );
    }
    assert!(shell.contains("hermes_acknowledge"));
    assert!(!terminal.contains("hermes_consumer_command::acknowledge"));
    assert!(!terminal.contains("ConsumerClient"));
}

#[test]
fn hermes_hosted_gate_uses_the_owner_daemon_and_public_client() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/hermes_hosted.sh")).expect("hosted Hermes gate");
    for required in [
        "--package hermes-daemon",
        "\"deployment\": \"hosted\"",
        "\\\"mode\\\": \\\"$mode\\\"",
        "ledger.observe.v1",
        "hosted_shutdown",
        "disposition\": \"clean",
        "run_profile create",
        "run_profile open",
        "interactive",
        "streams: 0",
        "reopened_after_interactive",
    ] {
        assert!(
            gate.contains(required),
            "hosted Hermes gate omits {required}"
        );
    }
    for forbidden in [
        "--package hermes-host",
        "--package hermes-service",
        "keystone-storage",
    ] {
        assert!(
            !gate.contains(forbidden),
            "hosted Hermes gate links owner internals through {forbidden}"
        );
    }
}

#[test]
fn aegis_hosted_gate_uses_the_owner_daemon_and_public_client() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/aegis_hosted.sh")).expect("hosted Aegis gate");
    for required in [
        "--package aegis-daemon",
        "\"deployment\": \"hosted\"",
        "\\\"mode\\\": \\\"$mode\\\"",
        "authority.observe.v1",
        "\"role\": \"administrator\"",
        "hosted_shutdown",
        "disposition\": \"clean",
        "run_profile create",
        "run_profile open",
        "interactive",
        "epoch: 2",
        "Manage retained Aegis authority",
        "retained grants: 2 / 8",
        "0 in flight · 0 budget violations",
        "peer_receipt",
        "reopened_after_interactive",
    ] {
        assert!(
            gate.contains(required),
            "hosted Aegis gate omits {required}"
        );
    }
    for forbidden in [
        "--package aegis-host",
        "--package aegis-service",
        "keystone-storage",
    ] {
        assert!(
            !gate.contains(forbidden),
            "hosted Aegis gate links owner internals through {forbidden}"
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
fn moira_hosting_uses_the_sovereign_owner_and_peer_public_client() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let host = fs::read_to_string(root.join("cli/src/native_host.rs")).expect("native host");
    for required in [
        "MoiraProcessOptions",
        "start_moira_process",
        ".arg(\"--supervised\")",
        "wait_for_process_ready",
        "stop_supervised_process",
    ] {
        assert!(host.contains(required), "Moira host omits {required}");
    }
    for forbidden in ["max_database_pages", "max_sagas", "OperatorAuthorizer"] {
        assert!(
            !host.contains(forbidden),
            "Pantheon acquired Moira owner policy through {forbidden}"
        );
    }
    let main = fs::read_to_string(root.join("cli/src/main.rs")).expect("CLI main");
    assert!(main.contains("options.moira_host.is_none()"));
    assert!(main.contains("host.start_moira_process(moira.clone())"));
    assert!(main.contains("moira_client::unix::SagaUnixPort::connect"));
}

#[test]
fn moira_hosted_gate_reopens_real_durable_saga_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/moira_hosted.sh")).expect("hosted Moira gate");
    for required in [
        "moira-saga-local",
        "admit_saga",
        "--supervised",
        "headless --workspace-profile",
        "interactive",
        "census total 1",
        "hosted_shutdown",
        "disposition\": \"clean",
        "reopened=$(\"$workbench\" headless",
    ] {
        assert!(
            gate.contains(required),
            "hosted Moira gate omits {required}"
        );
    }
}

#[test]
fn moira_reconciliation_reuses_the_owner_guard_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/moira_reconciliation.sh"))
        .expect("Moira reconciliation gate");
    for required in [
        "aegis-daemon",
        "moira-saga-local",
        "moira-reconciliation-view",
        "moira-reconciliation-confirm",
        "moira-reconciliation-recover",
        "sagas.reconciliation.confirm.v1",
        "sagas.reconciliation.recovery.v1",
        "unselected saga reconciliation was accepted",
        "hosted_shutdown",
        "authority_epoch",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Moira reconciliation gate omits {required}"
        );
    }
    let manifest = fs::read_to_string(root.join("cli/Cargo.toml")).expect("CLI manifest");
    assert!(manifest.contains("moira-reconciliation-command"));
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("moira_reconciliation_command::{PrepareError, prepare}"));
    assert!(cli.contains("moira_reconciliation_command::{ConfirmError, confirm}"));
    assert!(cli.contains("moira_reconciliation_command::{RecoverError, recover}"));
    assert!(!cli.contains("SagaOperatorClient"));
    let batch_gate =
        fs::read_to_string(root.join("tests/live/moira_batch.sh")).expect("Moira batch live gate");
    for required in [
        "2 saga(s) selected for batch work",
        "Every reconciliation child durably applied",
        "Per-child durable receipts",
        "operator receipts: 1",
        "Fail selected Moira sagas terminally",
        "Cancel selected Moira sagas",
        "Compensate selected Moira sagas",
        "Admit failed evidence for selected Moira sagas",
        "Admit successful evidence for selected Moira sagas",
        "Retry failed effects for selected Moira sagas",
        "This is Moira domain time",
        "outcome: Failed",
        "outcome: Succeeded",
        "phase: Failed",
        "phase: CompensationPending",
        "phase: ForwardPending",
        "phase: ForwardSucceeded",
        "FailTerminal",
        "moira.reconciliation.cancel",
        "moira.reconciliation.compensate",
        "moira.reconciliation.admit-failed",
        "moira.reconciliation.admit-succeeded",
        "moira.reconciliation.retry",
        "0 in flight · 0 budget violations",
    ] {
        assert!(
            batch_gate.contains(required),
            "Moira batch live gate omits {required}"
        );
    }
    let batch = fs::read_to_string(root.join("../moira-tui/src/reconciliation_batch.rs"))
        .expect("Moira batch reconciliation component");
    for required in [
        "MAX_RECONCILIATION_BATCH",
        "stable_snapshot(&mut reader, proposal, None)",
        "Confirmation::OutcomeUnknown",
        "ChildState::NotSent",
        "operator_receipt(action)",
    ] {
        assert!(batch.contains(required), "Moira batch omits {required}");
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs"))
        .expect("Pantheon terminal composition");
    for required in [
        "Component::new_with_connector",
        "ReconciliationBatchComponent::new_with_connector",
        "ReconciliationBatchComponent::new_timed_with_connector",
        "OperatorDispositionV1::FailTerminal",
        "TimedReconciliationKind::Cancel",
        "TimedReconciliationKind::Compensate",
        "TimedReconciliationKind::AdmitFailed",
        "TimedReconciliationKind::AdmitSucceeded",
        "TimedReconciliationKind::Retry",
        "Component::selected_saga_views",
        "child_receipts",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon Moira batch composition omits {required}"
        );
    }
}

#[test]
fn daimon_live_gate_reuses_exact_agent_session_evidence_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_attach.sh")).expect("Daimon attach gate");
    for required in [
        "agent_daemon_fixture",
        "agents.presence.v1",
        "agents.session.inspect.v1",
        "agent_session",
        "ReattachRequired",
        "headless",
        "interactive",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon attach gate omits {required}"
        );
    }
    for forbidden in [
        "AgentSessionClient",
        "AgentCommandV1::Open",
        "AgentCommandV1::Prompt",
        "AgentCommandV1::Reattach",
    ] {
        assert!(
            !gate.contains(forbidden),
            "Daimon attach gate acquires mutation authority through {forbidden}"
        );
    }
}

#[test]
fn daimon_batch_gate_proves_bounded_real_owner_fan_out() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_batch.sh")).expect("Daimon batch gate");
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    for required in [
        "agent_set_fixture",
        "agent_sessions",
        "agents.session-set.inspect.v1",
        "agents.session-set.control.v1",
        "3 session(s) selected for bounded work",
        "Daimon run settled: 3 applied",
    ] {
        assert!(
            gate.contains(required),
            "Daimon batch gate omits {required}"
        );
    }
    for required in [
        "selected_observations",
        "AgentControlBatchComponent::new_with_connector",
        "fresh_operation_facts",
        "child_receipts",
        "refreshed_projection",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon Daimon batch composition omits {required}"
        );
    }
}

#[test]
fn daimon_interactive_prompt_gate_proves_multiline_n_session_real_work() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_prompt_batch.sh"))
        .expect("Daimon prompt batch gate");
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    let profile = fs::read_to_string(root.join("cli/src/profile.rs")).expect("profile parser");
    for required in [
        "agent_set_fixture",
        "prompt selected daimon sessions",
        "Compose Daimon prompt",
        "3 selected session(s)",
        "prompt ready session",
        "Daimon run settled: 3 applied",
        "TurnActive r13",
        "0 in flight · 0 budget violations",
        "child_receipts",
        "refreshed_projection",
        "session 3 prompted",
        "agents.session-set.prompt.v1",
    ] {
        assert!(
            gate.contains(required),
            "Daimon prompt batch gate omits {required}"
        );
    }
    for required in [
        "AgentPromptComposer",
        "daimon_prompt_requests",
        "PromptRequest",
        "fresh_identity",
        "child_receipts",
        "refreshed_projection",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon prompt composition omits {required}"
        );
    }
    for required in [
        "DaimonExecutionPolicyProfile",
        "max_elapsed_millis",
        "max_output_tokens",
        "validate_hex_identifier",
    ] {
        assert!(
            profile.contains(required),
            "Daimon prompt policy omits {required}"
        );
    }
}

#[test]
fn daimon_interactive_permission_gate_proves_challenge_native_n_session_work() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_permission_batch.sh"))
        .expect("Daimon permission batch gate");
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    for required in [
        "permission",
        "Decide Daimon permissions",
        "3 exact permission challenge(s)",
        "> RejectOnce",
        "session 1 permission AllowOnce",
        "session 2 permission RejectOnce",
        "session 3 permission AllowOnce",
        "agents.session-set.permission.v1",
        "0 in flight · 0 budget violations",
        "child_receipts",
        "refreshed_projection",
    ] {
        assert!(
            gate.contains(required),
            "Daimon permission batch gate omits {required}"
        );
    }
    for required in [
        "AgentPermissionComposer",
        "daimon_permission_challenges",
        "daimon_permission_requests",
        "AgentPermissionControlRequest",
        "AwaitingPermission",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon permission composition omits {required}"
        );
    }
}

#[test]
fn daimon_interactive_reattach_gate_proves_exact_n_session_recovery() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_reattach_batch.sh"))
        .expect("Daimon reattach batch gate");
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    for required in [
        "reattach",
        "Reattach selected Daimon sessions",
        "3 exact session controls",
        "session 1 reattached",
        "session 2 reattached",
        "session 3 reattached",
        "agents.session-set.reattach.v1",
        "0 in flight · 0 budget violations",
        "child_receipts",
        "refreshed_projection",
    ] {
        assert!(
            gate.contains(required),
            "Daimon reattach batch gate omits {required}"
        );
    }
    for required in [
        "daimon_reattach_requests",
        "AgentControlRequest::Reattach",
        "ReattachRequired",
        "resume_session",
        "daimon_execution_request",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon reattach composition omits {required}"
        );
    }
}

#[test]
fn daimon_partial_batch_recovery_inspects_exact_sent_sessions_without_retry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_recovery_batch.sh"))
        .expect("Daimon recovery batch gate");
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal host");
    let component = fs::read_to_string(root.join("../daimon-tui/src/control.rs"))
        .expect("Daimon guarded batch component");
    for required in [
        "close-unknown",
        "1 applied, 0 refused, 0 failed, 1 unknown, 1 not sent",
        "Recovery inspection complete",
        "session 1 closed",
        "session 2 closed",
        "session 3 closed",
        "child_receipts",
        "recovery_inspection",
        "0 in flight · 0 budget violations",
    ] {
        assert!(
            gate.contains(required),
            "Daimon recovery batch gate omits {required}"
        );
    }
    for required in [
        "refresh_sessions(&sessions)",
        "recovery_inspection_started",
        "complete_recovery_inspection",
        "original outcome remains unknown",
        "daimon_workset_error",
    ] {
        assert!(
            terminal.contains(required),
            "Pantheon recovery composition omits {required}"
        );
    }
    for required in [
        "recovery_sessions",
        "recovery_required",
        "Applied",
        "OutcomeUnknown",
    ] {
        assert!(
            component.contains(required),
            "Daimon recovery evidence omits {required}"
        );
    }
}

#[test]
fn daimon_phase_hints_improve_discovery_without_claiming_authority() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let presentation =
        fs::read_to_string(root.join("../daimon-tui/src/lib.rs")).expect("Daimon presentation");
    let session_set = fs::read_to_string(root.join("../daimon-tui/src/agent_set.rs"))
        .expect("Daimon session-set presentation");
    for required in [
        "owner-state candidates (authority still required)",
        "prompt · close",
        "advance · cancel",
        "decide permission · cancel",
        "reattach",
        "inspect; never retry implicitly",
        "none; session closed",
    ] {
        assert!(
            presentation.contains(required),
            "Daimon phase hints omit {required}"
        );
    }
    assert!(session_set.contains("phase_actions"));
    for gate in [
        "tests/live/daimon_prompt_batch.sh",
        "tests/live/daimon_reattach_batch.sh",
        "tests/live/daimon_recovery_batch.sh",
    ] {
        let gate = fs::read_to_string(root.join(gate)).expect("Daimon phase-hint live gate");
        assert!(gate.contains("owner-state candidates (authority still required)"));
    }
}

#[test]
fn daimon_hosting_uses_the_owner_supervisor_without_importing_provider_policy() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let host = fs::read_to_string(root.join("cli/src/native_host.rs")).expect("native host");
    for required in [
        "DaimonProcessOptions",
        "start_daimon_process",
        ".arg(\"--supervised\")",
        "wait_for_process_ready",
        "stop_supervised_process",
    ] {
        assert!(host.contains(required), "Daimon host omits {required}");
    }
    for forbidden in [
        "provider_id",
        "inherited_environment_names",
        "aegis_grant_now",
    ] {
        assert!(
            !host.contains(forbidden),
            "Pantheon acquired Daimon owner policy through {forbidden}"
        );
    }
    let main = fs::read_to_string(root.join("cli/src/main.rs")).expect("CLI main");
    assert!(main.contains("options.daimon_host.is_none()"));
    assert!(main.contains("host.start_daimon_process(daimon.clone())"));
}

#[test]
fn daimon_hosted_gate_reopens_real_durable_session_in_both_workbench_modes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_hosted.sh")).expect("hosted Daimon gate");
    for required in [
        "aegis-daemon",
        "daimon-local-agent",
        "qualification_chronos",
        "qualification_seed",
        "--supervised",
        "headless --workspace-profile",
        "interactive",
        "inspect local provider configuration",
        "Daimon validated the selected owner document",
        "commands, arguments, paths, credential names, and secrets remain redacted",
        "REFRESHED",
        "CURRENT · Daimon validated",
        "ReattachRequired",
        "hosted_shutdown",
        "disposition\": \"clean",
    ] {
        assert!(
            gate.contains(required),
            "hosted Daimon gate omits {required}"
        );
    }
}

#[test]
fn hosted_local_model_uses_its_distinct_protocol_and_preserves_qualification_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_model_hosted.sh"))
        .expect("hosted local-model gate");
    for required in [
        "daimon-local-model",
        "hestia-daemon",
        "hestiactl",
        "Hestia-owned local runtime",
        "HESTIA RUNTIME EVIDENCE",
        "Hestia-correlated human gate",
        "model_socket",
        "models.endpoint.attach.v1",
        "MODEL ENDPOINT ATTACHED",
        "qwen/qwen3-coder-next",
        "openai_chat_completions",
        "models.provider.configure.v1",
        "models.task.invoke.v1",
        "guarded exact-field edit/apply",
        "owner-retained qualification",
        "reviewed replay-fenced invocation",
        "headless/TUI receipt parity",
        "two provider calls",
        "hosted_shutdown",
        "disposition\": \"clean",
    ] {
        assert!(
            gate.contains(required),
            "hosted local-model gate omits {required}"
        );
    }
    let profile = fs::read_to_string(root.join("cli/src/profile.rs")).expect("profile source");
    assert!(profile.contains("model_socket"));
    assert!(profile.contains("exactly one model endpoint or exact agent selection"));
    let headless = fs::read_to_string(root.join("cli/src/headless.rs")).expect("headless source");
    assert!(headless.contains("connect_model"));
    assert!(headless.contains("observe_hestia"));
    assert!(headless.contains("no execution, provider health, qualification, or authority"));
    let provider =
        fs::read_to_string(root.join("cli/src/provider.rs")).expect("provider facade source");
    assert!(provider.contains("--qualify-provider"));
    assert!(provider.contains("PROVIDER_QUALIFICATION_TIMEOUT"));
    assert!(provider.contains("validate_provider_qualification"));
    assert!(provider.contains("--invoke-task"));
    assert!(provider.contains("validate_model_task_invocation_receipt"));
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    assert!(terminal.contains("models.provider.configure.v1"));
    assert!(terminal.contains("--describe-provider-edit"));
    assert!(terminal.contains("secret values never projected"));
    assert!(terminal.contains("qualification_required"));
    assert!(terminal.contains("models.provider.qualify.v1"));
    assert!(terminal.contains("workitem.model.new"));
    assert!(terminal.contains("models.task.invoke.result"));
    assert!(terminal.contains("review → arm → confirm"));
    assert!(terminal.contains("tool-continuation conformance"));
    assert!(terminal.contains("observe_provider_runtime"));
    assert!(terminal.contains("lifecycle/readiness is Hestia-owned"));
    let live = fs::read_to_string(root.join("tests/live/daimon_model_lmstudio.sh"))
        .expect("LM Studio live gate");
    for required in [
        "PANTHEON_LMSTUDIO_LIVE",
        "qwen/qwen3-coder-next",
        "qualified_completion",
        "tool continuation    not_exercised",
        "assert \"output\" not in qualification",
        "models.task.invoke.v1",
        "local-model-review",
        "MODEL RESPONSE",
        "assert \"output\" not in invocation",
        "LOCAL_MODEL_REVIEW_BEGIN",
    ] {
        assert!(live.contains(required), "LM Studio gate omits {required}");
    }
}

#[test]
fn hestia_artifact_gate_proves_human_review_owner_admission_and_repository_hygiene() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/hestia_artifact_acquisition.sh"))
        .expect("Hestia artifact gate");
    for required in [
        "artifact_catalog",
        "HESTIA · ACQUISITION",
        "source revision    revision-1",
        "license            Apache-2.0",
        "expected bytes     23",
        "human gate",
        "lifecycle          Admitted",
        "headless --workspace-profile",
        "does not instantiate, qualify, invoke, or schedule",
        "-name '*.gguf'",
        "private to the owner store",
    ] {
        assert!(
            gate.contains(required),
            "Hestia artifact gate omits {required}"
        );
    }
    let profile = fs::read_to_string(root.join("cli/src/profile.rs")).expect("profile source");
    assert!(profile.contains("artifact_catalog"));
    let headless = fs::read_to_string(root.join("cli/src/headless.rs")).expect("headless source");
    assert!(headless.contains("artifacts.catalog.v1"));
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    assert!(terminal.contains("hestia.artifact.acquire"));
    assert!(terminal.contains("apply_hestia_artifact_action"));
}

#[test]
fn hestia_real_model_gate_separates_host_readiness_from_provider_readiness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/hestia_real_model.sh"))
        .expect("Hestia real-model gate");
    for required in [
        "PANTHEON_LIVE_MODEL",
        "Qwen2.5-Coder-1.5B-Instruct",
        "2ab9f8f42af02fc212effaef7c4850c885e965f4",
        "cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046",
        "1117320768",
        "llama-server",
        "/v1/models",
        "/v1/chat/completions",
        "PANTHEON_REAL_MODEL_DAIMON",
        "PANTHEON_LOCAL_MODEL_HESTIA_SOCKET",
        "daimon_model_lmstudio.sh",
        "Artifact admitted",
        "drain",
        "shut down a real Qwen GGUF",
    ] {
        assert!(
            gate.contains(required),
            "Hestia real-model gate omits {required}"
        );
    }
}

#[test]
fn daimon_advance_reuses_the_owner_guard_in_standalone_and_workbench() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_advance.sh")).expect("Daimon advance gate");
    for required in [
        "agent_advance_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-advance",
        "agents.session.advance.v1",
        "ADVANCE DAIMON SESSION",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon advance gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::advance"));
    assert!(!cli.contains("AgentCommandV1::Advance"));
}

#[test]
fn daimon_cancel_reuses_the_owner_guard_in_standalone_and_workbench() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_cancel.sh")).expect("Daimon cancel gate");
    for required in [
        "agent_cancel_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-cancel",
        "agents.session.cancel.v1",
        "CANCEL DAIMON SESSION",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon cancel gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::cancel"));
    assert!(!cli.contains("AgentCommandV1::Cancel"));
}

#[test]
fn daimon_close_reuses_the_owner_guard_in_standalone_and_workbench() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_close.sh")).expect("Daimon close gate");
    for required in [
        "agent_close_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-close",
        "agents.session.close.v1",
        "CLOSE DAIMON SESSION",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon close gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::close"));
    assert!(!cli.contains("AgentCommandV1::Close"));
}

#[test]
fn daimon_reattach_reuses_the_owner_guard_and_document_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_reattach.sh"))
        .expect("Daimon reattach gate");
    for required in [
        "agent_reattach_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-reattach",
        "agents.session.reattach.v1",
        "request.json",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon reattach gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::reattach"));
    assert!(cli.contains("parse_reattach_document"));
    assert!(!cli.contains("AgentCommandV1::Reattach"));
}

#[test]
fn daimon_prompt_reuses_the_owner_guard_and_document_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_prompt.sh")).expect("Daimon prompt gate");
    for required in [
        "agent_prompt_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-prompt",
        "agents.session.prompt.v1",
        "prompt.txt",
        "request.json",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon prompt gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::prompt"));
    assert!(cli.contains("parse_prompt_document"));
    assert!(!cli.contains("AgentCommandV1::Prompt"));
}

#[test]
fn daimon_session_open_reuses_the_owner_guard_and_document_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/daimon_open.sh")).expect("Daimon open gate");
    for required in [
        "agent_open_fixture",
        "daimon-tui",
        "pantheon",
        "agent-open",
        "daimon-agent-open",
        "agents.session.open.v1",
        "request.json",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon session-open gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::open"));
    assert!(cli.contains("parse_open_document"));
    assert!(!cli.contains("AgentCommandV1::Open"));
}

#[test]
fn daimon_permission_reuses_the_owner_guard_and_exact_challenge_in_both_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/daimon_permission.sh"))
        .expect("Daimon permission gate");
    for required in [
        "agent_permission_fixture",
        "daimon-tui",
        "pantheon",
        "daimon-agent-permission",
        "agents.session.permission.v1",
        "request.json",
        "standalone.marker",
        "workbench.marker",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "Daimon permission gate omits {required}"
        );
    }
    let cli = fs::read_to_string(root.join("cli/src/main.rs")).expect("Pantheon CLI");
    assert!(cli.contains("daimon_agent_command::decide_permission"));
    assert!(cli.contains("parse_permission_document"));
    assert!(!cli.contains("AgentCommandV1::DecidePermission"));
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
        "tellus_fixture.exs",
        "sagas.inspect.v1",
        "fenced",
        "agents.presence.v1",
        "ledger.observe.v1",
        "authority.observe.v1",
        "tellus.observe.v1",
        "configured_stream",
        "configured_cursor",
        "05050505050505050505050505050505",
        "moira                 sagas",
        "daimon                sessions/events",
        "hermes                ledger/cursors",
        "aegis                 authority",
        "tellus                provenance",
        "events: 1 visible / 2 observed · filter usage",
        "observed windows 1/2 events visible · usage input=21 output=34",
        "0 in flight · 0 budget violations",
        "workbench.render",
        "evidence-backed Ratatui Overview",
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

#[test]
fn tui_breadth_gate_drives_every_first_class_surface_and_global_key() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate =
        fs::read_to_string(root.join("tests/live/tui_breadth.sh")).expect("TUI breadth gate");
    for required in [
        "tmux",
        "OVERVIEW",
        "CHRONOS",
        "VELLUM",
        "AGORA",
        "MOIRA",
        "DAIMON",
        "TELLUS",
        "AEGIS",
        "HERMES",
        "OPERATIONS",
        "CONFIGURATION",
        "M-c",
        "M-v",
        "M-,",
        "M-2",
        "[48;5u",
        "[57;5u",
        "\\x1b[<0;14;2M",
        "C-p",
        "F1",
        "C-g",
        "C-q",
        "Fresh observations queued across configured peers",
        "PASS: tmux drove every Pantheon surface",
        "--locked",
        "--offline",
    ] {
        assert!(gate.contains(required), "TUI breadth gate omits {required}");
    }
}

#[test]
fn tui_operational_demo_proves_make_run_and_every_peer_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let makefile = fs::read_to_string(root.join("Makefile")).expect("Makefile");
    for required in [
        "tui-demo:",
        "./tests/live/tui_breadth.sh",
        "./tests/live/emacs_keybindings.sh",
        "./tests/live/tui_operational_depth.sh",
        "--chronos-allow-writes",
        "--vellum-allow-add-file",
    ] {
        assert!(makefile.contains(required), "Makefile omits {required}");
    }

    let run_gate = fs::read_to_string(root.join("tests/live/make_run.sh")).expect("make run gate");
    for required in [
        "make run",
        "Task batch",
        "Open file",
        "source.txt",
        "Ctrl-O/Ctrl-F-capable hosted Vellum",
    ] {
        assert!(
            run_gate.contains(required),
            "make run gate omits {required}"
        );
    }

    let emacs = fs::read_to_string(root.join("tests/live/emacs_keybindings.sh"))
        .expect("Emacs keybinding gate");
    for required in [
        "C-x\", \"b",
        "C-x\", \"Right",
        "C-s",
        "C-o",
        "C-a",
        "C-x\", \"C-f",
        "C-x\", \"C-s",
        "C-x\", \"C-c",
        "Emacs profile uses real prefix chords",
    ] {
        assert!(emacs.contains(required), "Emacs gate omits {required}");
    }

    let depth = fs::read_to_string(root.join("tests/live/tui_operational_depth.sh"))
        .expect("TUI operational depth gate");
    for required in [
        "make_run",
        "chronos_mutation",
        "dual_service",
        "agora_mutation",
        "moira_reconciliation",
        "daimon_prompt",
        "work_item_agent_journey",
        "tellus_attach",
        "aegis_attach",
        "hermes_attach",
        "operational depth exercised every configured peer surface",
    ] {
        assert!(depth.contains(required), "TUI depth gate omits {required}");
    }

    let journey = fs::read_to_string(root.join("tests/live/work_item_agent_journey.sh"))
        .expect("work item agent journey");
    for required in [
        "create task with product owner agent lens",
        "Compose Daimon prompt",
        "Local-agent turn accepted",
        "product-owner collaboration lens",
        "not a human identity",
        "local-agent-task",
        "provider_catalog_open_owner.sh",
        "return to the kanban board",
        "agent_open_journey_fixture",
        "Open fresh Daimon session for Chronos task",
        "human-gated fresh Daimon session",
        "Create Agora room",
        "Bind Chronos task to Agora room",
        "durable exact relationship",
        "offer addressed agora message to agent cohort",
        "ordinary room chatter is not a prompt",
        "ambient chatter inert",
        "PASS: Chronos task -> exact owner binding -> human-gated fresh Daimon session -> guarded prompt -> human-gated direct Agora task conversation -> durable exact relationship -> explicitly addressed room message -> guarded second agent turn -> refreshed Kanban board",
    ] {
        assert!(
            journey.contains(required),
            "work-item journey omits {required}"
        );
    }
}

#[test]
fn interactive_reconfiguration_reuses_typed_preflight_and_replaces_the_process() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    for required in [
        "WorkspaceProfile::load",
        "reconfiguration::plan",
        "full local topology restart required",
        "Restart refused: guarded work or topology drain is pending or outcome-unknown",
        "Restart refused: save the dirty Vellum workset",
    ] {
        assert!(terminal.contains(required), "terminal omits {required}");
    }
    let application = fs::read_to_string(root.join("cli/src/main.rs")).expect("application");
    assert!(application.contains("restart_current_process_with_profile"));
    assert!(application.contains("Command::new(executable)"));
    assert!(application.contains("PANTHEON_EXPECT_PROFILE_SHA256"));
    let gate = fs::read_to_string(root.join("tests/live/reconfiguration_restart.sh"))
        .expect("reconfiguration live gate");
    for required in [
        "preflight and restart workspace profile",
        "workspace.name · changed · shell_restart_required",
        "workspace.surfaces · changed · shell_restart_required",
        "ARMED: Enter drains owned topology and restarts this process",
        "workbench.reconfigure.preflight",
        "0 in flight · 0 budget violations",
        "Restart refused: profile bytes changed after preflight",
        "surfaces            Overview · Operations",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "reconfiguration gate omits {required}"
        );
    }
}

#[test]
fn interactive_reconfiguration_drains_and_replaces_a_real_hosted_topology() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gate = fs::read_to_string(root.join("tests/live/reconfiguration_hosted_restart.sh"))
        .expect("hosted reconfiguration live gate");
    for required in [
        "full_local_topology_restart",
        "chronos · changed · full_local_topology_restart_required",
        "hosted_shutdown\"][\"status\"] == \"clean",
        "chronos/Borealis",
        "reconfiguration: allowed",
        "independently reopened old topology",
        "--locked",
        "--offline",
    ] {
        assert!(
            gate.contains(required),
            "hosted reconfiguration gate omits {required}"
        );
    }
}

#[test]
fn moira_participation_gate_proves_restart_and_honest_failure_projections() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("qualification/moira-participation/Cargo.toml"))
        .expect("Moira participation qualification manifest");
    for owner in [
        "moira-local-daemon",
        "moira-sqlite",
        "moira-service",
        "moira-client",
        "moira-tui",
    ] {
        assert!(manifest.contains(owner), "qualification omits {owner}");
    }
    let gate = fs::read_to_string(
        root.join("qualification/moira-participation/tests/live_participation.rs"),
    )
    .expect("Moira participation live gate");
    for required in [
        "current-before-restart",
        "current-after-restart",
        "not_found",
        "connect_refused",
        "read-refused",
        "RefusalV1::RevisionConflict",
        "Current governed revision 2",
        "accountable_owner",
        "principal_type",
        "previously_observed_revision",
        "revision_history",
        "missing_facts",
        "persona: Researcher",
        "grounding depth: Activated",
        "not published: role/work mode · scope · delegation · budgets",
        "owner_derived_open_prepares_once_and_survives_restart",
        "OpenControlInput::Arm",
        "OpenControlInput::Confirm",
        "PrepareAgentOpenDispositionV1::AlreadyPrepared",
        "pending operation: Some(Open",
    ] {
        assert!(
            gate.contains(required),
            "Moira participation qualification omits {required}"
        );
    }
    let makefile = fs::read_to_string(root.join("Makefile")).expect("Makefile");
    assert!(makefile.contains("qualify-moira-participation"));
}

#[test]
fn moira_agent_worker_is_opaque_participation_bound_and_separate_from_preparation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = fs::read_to_string(root.join("cli/src/profile.rs")).expect("profile source");
    for required in [
        "MoiraAgentWorkerProfile",
        "Moira agent worker requires an exact selected participation",
        "worker.executable = resolve",
        "worker.config = resolve",
    ] {
        assert!(
            profile.contains(required),
            "worker profile omits {required}"
        );
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    for required in [
        "moira.participation.open.prepare",
        "moira.participation.worker.run",
        "dispatch_ready",
        "owner worker receipt named a different participation",
        "owner worker output exceeded 4096 bytes",
        "Dispatch refused: review and durably prepare",
        "MoiraAgentWorkerSettlement::FencedUnknown",
        "release_for_dispatch",
    ] {
        assert!(terminal.contains(required), "worker route omits {required}");
    }
    let owner = fs::read_to_string(root.join("../moira/apps/moira-agent-local/src/lib.rs"))
        .expect("Moira agent worker source");
    for required in [
        r#""participation":"{}""#,
        "summary(outcome, config.participation)",
        "participation_revision",
    ] {
        assert!(owner.contains(required), "owner receipt omits {required}");
    }
    let live =
        fs::read_to_string(root.join("qualification/moira-participation/live_owner_worker.sh"))
            .expect("owner-worker live qualification");
    for required in [
        "headless-before",
        "dispatch prepared moira agent session",
        "Moira owner worker Observed participation revision 3",
        "headless-after",
        "owner_worker_fixture",
    ] {
        assert!(live.contains(required), "live worker gate omits {required}");
    }
}

#[test]
fn task_conversation_gate_proves_tui_headless_and_recovery_parity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let live = fs::read_to_string(root.join("tests/live/task_conversation.sh"))
        .expect("task-conversation live qualification");
    for required in [
        "Bind selected task to open Agora room",
        "room activity remains separate from task progress",
        "task-conversation.v1",
        "peer[\"revision\"] == 1 and peer[\"links\"] == 1",
        "drain_link",
        "owner restart recovery",
    ] {
        assert!(
            live.contains(required),
            "task-conversation gate omits {required}"
        );
    }
    let makefile = fs::read_to_string(root.join("Makefile")).expect("Makefile");
    assert!(makefile.contains("qualify-task-conversation"));
}

#[test]
fn shared_agent_context_gate_proves_exact_multi_stakeholder_routing() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let live = fs::read_to_string(root.join("tests/live/agora_shared_agent_context.sh"))
        .expect("Agora shared-agent-context qualification");
    for required in [
        "Do we have the right ensemble?",
        "does not authorize disposition",
        "group direct · operator, project-agent, sre-agent",
        "agent_routes",
        "Configured addressees: sre-agent, project-agent",
        "2 exact session controls",
        "session 1 prompted",
        "session 2 prompted",
        "session 3 prompted",
        "ambient session excluded",
        "exact_participant_sessions",
        "headless routing parity",
    ] {
        assert!(
            live.contains(required),
            "shared-context gate omits {required}"
        );
    }
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    for required in [
        "addressed_message_routes",
        "target_sessions",
        "agent.observation(*session)",
        "participant-to-session routes",
        "one reviewed Daimon turn per routed session",
    ] {
        assert!(
            terminal.contains(required),
            "shared-context host omits {required}"
        );
    }
    let headless = fs::read_to_string(root.join("cli/src/headless.rs")).expect("headless source");
    for required in [
        "task_conversation_routing_projection",
        "exact_participant_sessions",
        "grants neither Agora membership",
    ] {
        assert!(
            headless.contains(required),
            "shared-context headless projection omits {required}"
        );
    }
    let makefile = fs::read_to_string(root.join("Makefile")).expect("Makefile");
    assert!(makefile.contains("qualify-agora-shared-context"));
}

#[test]
fn moira_deliberation_gate_proves_exact_ensemble_and_human_gate_evidence_in_both_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let live = fs::read_to_string(root.join("tests/live/moira_deliberation.sh"))
        .expect("Moira deliberation qualification");
    for required in [
        "deliberation.inspect.v1",
        "right ensemble? Yes",
        "authorized_representative",
        "stakeholder standing: accountable=22222222222222222222222222222222",
        "evidence=1 dissent=1",
        "human gate: changes_requested",
        "material_revision_digest",
        "authority",
        "infers no ensemble fitness",
        "Open Moira deliberation",
    ] {
        assert!(
            live.contains(required),
            "Moira deliberation gate omits {required}"
        );
    }
    let profile = fs::read_to_string(root.join("cli/src/profile.rs")).expect("profile source");
    assert!(profile.contains("deliberation_socket"));
    assert!(profile.contains("at least one selected presentation endpoint"));
    let terminal = fs::read_to_string(root.join("cli/src/terminal.rs")).expect("terminal source");
    assert!(terminal.contains("moira.view.deliberation"));
    assert!(terminal.contains("MoiraPane::Deliberation"));
    let headless = fs::read_to_string(root.join("cli/src/headless.rs")).expect("headless source");
    assert!(headless.contains("moira_tui::deliberation_view_json"));
    assert!(headless.contains("Named-human deliberation gate"));
    assert!(headless.contains("no selected packet reader"));
    assert!(headless.contains("AgendaActionability::InspectOnly"));
    assert!(terminal.contains("operator_agenda_observation"));
    let makefile = fs::read_to_string(root.join("Makefile")).expect("Makefile");
    assert!(makefile.contains("qualify-moira-deliberation"));
}
