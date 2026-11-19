use std::{collections::BTreeMap, fs, path::Path, process::Command};

use pantheon_cli::capabilities::{
    self, AGENT_TOOL_GAPS, AGENT_TOOL_ROUTES, Capability, ChronosFeatures,
    GUARDED_CONTROL_CAPABILITIES, GUARDED_CONTROL_ROUTES, Mode,
};

fn status() -> serde_json::Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CLI package is nested under Pantheon");
    serde_json::from_str(
        &fs::read_to_string(root.join("composition-status.json"))
            .expect("composition capability status"),
    )
    .expect("valid composition capability status")
}

fn assert_route(status: &serde_json::Value, peer: &str, mode: Mode, capabilities: Vec<Capability>) {
    let mode = mode.label();
    let declared = status["peers"][peer]["pantheonRoutes"][mode]
        .as_array()
        .expect("declared Pantheon route")
        .iter()
        .map(|entry| {
            (
                entry["id"].as_str().expect("capability id"),
                entry["plane"].as_str().expect("capability plane"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for capability in capabilities {
        assert_eq!(
            declared.get(capability.id).copied(),
            Some(capability.plane.label()),
            "{peer} {mode} runtime capability is absent or classified differently in composition-status.json: {}",
            capability.id
        );
    }
}

#[test]
fn every_runtime_route_capability_is_in_the_machine_matrix() {
    let status = status();
    for hosted in [false, true] {
        for mode in [Mode::Interactive, Mode::Headless] {
            assert_route(&status, "agora", mode, capabilities::agora(mode, hosted));
            assert_route(
                &status,
                "moira",
                mode,
                capabilities::moira(mode, hosted, true, false, false, false),
            );
            assert_route(
                &status,
                "daimon",
                mode,
                capabilities::daimon(mode, hosted, true, false, true, false),
            );
            assert_route(
                &status,
                "hermes",
                mode,
                capabilities::hermes(mode, hosted, true),
            );
            assert_route(&status, "aegis", mode, capabilities::aegis(hosted));
            assert_route(
                &status,
                "chronos",
                mode,
                capabilities::chronos(
                    mode,
                    hosted,
                    ChronosFeatures {
                        writes: true,
                        scheduling: true,
                        watch: true,
                        planning: true,
                        cancel_receipts: true,
                        moira_attempts: true,
                        daimon_attempts: true,
                        alignment: true,
                    },
                ),
            );
            assert_route(
                &status,
                "vellum",
                mode,
                capabilities::vellum(mode, hosted, true),
            );
        }
    }
    for mode in [Mode::Interactive, Mode::Headless] {
        assert_route(&status, "tellus", mode, capabilities::tellus());
    }
}

#[test]
fn every_guarded_control_is_classified_and_only_composed_deliberately() {
    let status = status();
    for (peer, capabilities) in GUARDED_CONTROL_CAPABILITIES {
        let peer_status = &status["peers"][peer];
        for capability in *capabilities {
            let state = peer_status["mutationCapabilities"][capability]
                .as_str()
                .or_else(|| peer_status["disclosureCapabilities"][capability].as_str());
            assert_eq!(
                state,
                Some("qualified"),
                "unclassified {peer} control {capability}"
            );
            for mode in ["interactive", "headless"] {
                let integrated_vellum = *peer == "vellum" && mode == "interactive";
                let deliberately_composed = mode == "interactive"
                    && peer_status["interactiveControlComposition"][capability]["state"]
                        == "composed";
                assert!(
                    integrated_vellum
                        || deliberately_composed
                        || !peer_status["pantheonRoutes"][mode]
                            .as_array()
                            .expect("aggregate route")
                            .iter()
                            .any(|entry| entry["id"].as_str() == Some(capability)),
                    "separate guarded control leaked into {peer} {mode} aggregate negotiation: {capability}"
                );
            }
        }
    }
}

#[test]
fn guarded_controls_are_machine_discoverable_with_concrete_routes() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("controls")
        .output()
        .expect("run guarded control catalog");
    assert!(output.status.success());
    let catalog: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(catalog["mode"], "guarded_control_catalog");
    assert_eq!(catalog["scope"], "single_user_local_machine");
    let controls = catalog["controls"].as_array().unwrap();
    assert_eq!(controls.len(), GUARDED_CONTROL_ROUTES.len());
    for route in GUARDED_CONTROL_ROUTES {
        let entry = controls
            .iter()
            .find(|entry| entry["capability"] == route.capability)
            .unwrap_or_else(|| panic!("catalog omits {}", route.capability));
        assert_eq!(entry["peer"], route.peer);
        assert_eq!(entry["command"], route.command);
        assert_eq!(entry["plane"], "control");
        assert_eq!(entry["aggregate_route"], false);
        assert_eq!(entry["authority"], "peer_owned");
    }
}

#[test]
fn bounded_agent_tools_are_machine_discoverable_and_individually_admissible() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .arg("agent-tools")
        .output()
        .expect("run bounded agent tool catalog");
    assert!(output.status.success());
    let catalog: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(catalog["mode"], "bounded_agent_tool_catalog");
    let tools = catalog["tools"].as_array().unwrap();
    assert_eq!(tools.len(), AGENT_TOOL_ROUTES.len());
    for route in AGENT_TOOL_ROUTES {
        let entry = tools
            .iter()
            .find(|entry| entry["semantic_capability"] == route.capability)
            .unwrap_or_else(|| panic!("catalog omits {}", route.capability));
        assert_eq!(entry["component"], route.component);
        assert_eq!(entry["command"], route.command);
        assert_eq!(entry["model_arguments"], route.model_arguments);
        assert_eq!(entry["aggregate_route"], false);
        assert_eq!(entry["default_admission"], "denied");
    }
    let unavailable = catalog["unavailable"].as_array().unwrap();
    assert_eq!(unavailable.len(), AGENT_TOOL_GAPS.len());
    for gap in AGENT_TOOL_GAPS {
        let entry = unavailable
            .iter()
            .find(|entry| entry["component"] == gap.component)
            .unwrap_or_else(|| panic!("catalog omits {} gap", gap.component));
        assert_eq!(entry["status"], "not_admissible");
        assert_eq!(entry["missing"], gap.missing);
        assert_eq!(entry["next_boundary"], gap.next_boundary);
        assert_eq!(entry["aggregate_fallback"], false);
    }
}

#[test]
fn every_peer_presentation_capability_has_an_explicit_pantheon_disposition() {
    let status = status();
    let manifests: [(&str, &[&str], &[&str]); 8] = [
        (
            "aegis",
            aegis_tui::CAPABILITIES.interactive,
            aegis_tui::CAPABILITIES.headless,
        ),
        (
            "agora",
            agora_tui::CAPABILITIES.interactive,
            agora_tui::CAPABILITIES.headless,
        ),
        (
            "chronos",
            chronos_tui::CAPABILITIES.interactive,
            chronos_tui::CAPABILITIES.headless,
        ),
        (
            "daimon",
            daimon_tui::CAPABILITIES.interactive,
            daimon_tui::CAPABILITIES.headless,
        ),
        (
            "tellus",
            tellus::CAPABILITIES.interactive,
            tellus::CAPABILITIES.headless,
        ),
        (
            "hermes",
            hermes_tui::CAPABILITIES.interactive,
            hermes_tui::CAPABILITIES.headless,
        ),
        (
            "moira",
            moira_tui::CAPABILITIES.interactive,
            moira_tui::CAPABILITIES.headless,
        ),
        (
            "vellum",
            vellum_tui::CAPABILITIES.interactive,
            vellum_tui::CAPABILITIES.headless,
        ),
    ];
    for (peer, interactive, headless) in manifests {
        let peer_status = &status["peers"][peer];
        let aggregate = ["interactive", "headless"]
            .into_iter()
            .flat_map(|mode| {
                peer_status["pantheonRoutes"][mode]
                    .as_array()
                    .unwrap_or_else(|| panic!("{peer} aggregate route"))
            })
            .filter_map(|entry| entry["id"].as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let guarded = GUARDED_CONTROL_CAPABILITIES
            .iter()
            .find(|(candidate, _)| *candidate == peer)
            .map_or(&[][..], |(_, capabilities)| *capabilities)
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let uncomposed = peer_status["uncomposedPeerCapabilities"].as_object();
        for capability in interactive.iter().chain(headless) {
            assert!(
                aggregate.contains(capability)
                    || guarded.contains(capability)
                    || uncomposed.is_some_and(|entries| entries.contains_key(*capability)),
                "{peer} presentation capability has no Pantheon disposition: {capability}"
            );
        }
        if let Some(uncomposed) = uncomposed {
            for capability in uncomposed.keys() {
                assert!(
                    interactive.contains(&capability.as_str())
                        || headless.contains(&capability.as_str()),
                    "{peer} defers capability not published by its presentation: {capability}"
                );
            }
        }
    }
}
