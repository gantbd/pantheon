use std::{collections::BTreeMap, fs, path::Path, process::Command};

use pantheon_cli::capabilities::{
    self, AGENT_OWNER_TOOL_ROUTES, AGENT_ROLE_PROFILES, AGENT_TOOL_GAPS, AGENT_TOOL_ROUTES,
    Capability, ChronosFeatures, GUARDED_CONTROL_CAPABILITIES, GUARDED_CONTROL_ROUTES, Mode,
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
    let owner_tools = catalog["owner_tools"].as_array().unwrap();
    assert_eq!(owner_tools.len(), AGENT_OWNER_TOOL_ROUTES.len());
    for route in AGENT_OWNER_TOOL_ROUTES {
        let entry = owner_tools
            .iter()
            .find(|entry| entry["semantic_capability"] == route.capability)
            .unwrap_or_else(|| panic!("catalog omits owner tool {}", route.capability));
        assert_eq!(entry["component"], route.component);
        assert_eq!(entry["owner_surface"], route.owner_surface);
        assert_eq!(entry["model_arguments"], route.model_arguments);
        assert_eq!(entry["enforcement"], route.enforcement);
        assert_eq!(entry["aggregate_route"], false);
        assert_eq!(entry["default_admission"], "denied");
    }
    let unavailable = catalog["unavailable"].as_array().unwrap();
    assert_eq!(unavailable.len(), AGENT_TOOL_GAPS.len());
    for gap in AGENT_TOOL_GAPS {
        let entry = unavailable
            .iter()
            .find(|entry| entry["component"] == gap.component && entry["surface"] == gap.surface)
            .unwrap_or_else(|| panic!("catalog omits {} {} gap", gap.component, gap.surface));
        assert_eq!(entry["status"], "surface_not_admissible");
        assert_eq!(entry["missing"], gap.missing);
        assert_eq!(entry["next_boundary"], gap.next_boundary);
        assert_eq!(entry["aggregate_fallback"], false);
    }
}

#[test]
fn role_plans_fail_closed_on_missing_tools_and_owner_surfaces() {
    for profile in AGENT_ROLE_PROFILES {
        let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
            .args(["agent-role-plan", profile.role])
            .output()
            .expect("run bounded agent role plan");
        assert!(output.status.success());
        let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(plan["mode"], "bounded_agent_role_plan");
        assert_eq!(plan["role"], profile.role);
        assert_eq!(plan["complete"], true);
        assert_eq!(plan["operational"], false);
        assert_eq!(plan["candidate_admission_complete"], false);
        assert_eq!(plan["candidate_least_authority"], true);
        assert_eq!(plan["status"], "blocked_by_owner_surfaces");
        assert_eq!(
            plan["missing_candidate_tools"].as_array().unwrap().len(),
            profile.required_tools.len()
        );
        assert_eq!(
            plan["structural_blockers"].as_array().unwrap().len(),
            profile.gaps.len()
        );
        assert!(
            plan["boundary"]
                .as_str()
                .unwrap()
                .contains("grants no tool")
        );
    }
}

#[test]
fn complete_semantic_candidate_still_does_not_claim_an_operational_sre() {
    let profile = capabilities::agent_role_profile("sre").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_pantheon"));
    command.args(["agent-role-plan", profile.role]);
    command.args(profile.required_tools);
    let output = command.output().expect("run complete SRE candidate plan");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["candidate_admission_complete"], true);
    assert_eq!(plan["candidate_least_authority"], true);
    assert_eq!(plan["operational"], false);
    assert_eq!(plan["status"], "blocked_by_owner_surfaces");
    assert!(
        plan["structural_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|gap| gap["component"] == "oculus"
                && gap["surface"] == "owner-query-runtime"
                && gap["aggregate_fallback"] == false)
    );
}

#[test]
fn verification_role_reuses_the_daimon_native_governed_test_catalog() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agent-role-plan",
            "verification-worker",
            "verification.test.run.v1",
        ])
        .output()
        .expect("run verification-worker candidate plan");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tool = plan["required_tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_capability"] == "verification.test.run.v1")
        .expect("Daimon governed test tool");
    assert_eq!(tool["component"], "daimon");
    assert_eq!(tool["route_kind"], "component_owner_catalogue");
    assert_eq!(tool["owner_surface"], "acp_terminal.commands[]");
    assert_eq!(tool["candidate_disposition"], "included");
    let envelope = plan["required_tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_capability"] == "verification.result-envelope.validate.v1")
        .expect("closed verification result envelope");
    assert_eq!(envelope["component"], "verification");
    assert_eq!(envelope["route_kind"], "pantheon_gateway");
    assert_eq!(envelope["candidate_disposition"], "missing");
    assert!(
        plan["structural_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|gap| gap["surface"] != "tests.allowlisted.execute.v1")
    );
    assert!(
        plan["structural_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|gap| gap["component"] == "moira" && gap["surface"] == "tests.result.publish.v1")
    );
    assert_eq!(plan["operational"], false);
}

#[test]
fn role_plan_exposes_overbroad_candidates_and_rejects_aggregate_authority() {
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agent-role-plan",
            "verification-worker",
            "agent.web.fetch.v1",
        ])
        .output()
        .expect("run overbroad verification plan");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["candidate_least_authority"], false);
    assert_eq!(
        plan["extraneous_candidate_tools"],
        serde_json::json!(["agent.web.fetch.v1"])
    );

    for forbidden in ["pantheon", "pantheon.*", "tasks.*"] {
        let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
            .args(["agent-role-plan", "product-manager", forbidden])
            .output()
            .expect("run forbidden aggregate plan");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("forbidden"));
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
