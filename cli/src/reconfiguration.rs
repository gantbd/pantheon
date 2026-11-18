//! Bounded local-machine reconfiguration planning.

use serde::Serialize;

use crate::profile::WorkspaceProfile;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReconfigurationChange {
    pub target: &'static str,
    pub change: &'static str,
    pub required_action: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReconfigurationPlan {
    pub version: u8,
    pub operation_id: String,
    pub scope: &'static str,
    pub live_apply_supported: bool,
    pub minimum_action: &'static str,
    pub changes: Vec<ReconfigurationChange>,
}

pub fn plan(
    operation_id: String,
    current: &WorkspaceProfile,
    next: &WorkspaceProfile,
) -> ReconfigurationPlan {
    let mut changes = Vec::new();
    if current.name != next.name {
        changes.push(change(
            "workspace.name",
            "changed",
            "shell_restart_required",
        ));
    }
    if current.surfaces != next.surfaces {
        changes.push(change(
            "workspace.surfaces",
            "changed",
            "shell_restart_required",
        ));
    }
    if current.keybindings != next.keybindings {
        changes.push(change(
            "workspace.keybindings",
            "changed",
            "shell_restart_required",
        ));
    }
    classify_peer("agora", &current.agora, &next.agora, &mut changes);
    classify_peer("moira", &current.moira, &next.moira, &mut changes);
    classify_peer("daimon", &current.daimon, &next.daimon, &mut changes);
    classify_peer("hermes", &current.hermes, &next.hermes, &mut changes);
    classify_peer("aegis", &current.aegis, &next.aegis, &mut changes);
    classify_peer("tellus", &current.tellus, &next.tellus, &mut changes);
    classify_peer("chronos", &current.chronos, &next.chronos, &mut changes);
    classify_peer("vellum", &current.vellum, &next.vellum, &mut changes);
    ReconfigurationPlan {
        version: 1,
        operation_id,
        scope: "single_user_local_machine",
        live_apply_supported: false,
        minimum_action: if changes.is_empty() {
            "none"
        } else if changes
            .iter()
            .all(|change| change.required_action == "shell_restart_required")
        {
            "shell_restart"
        } else {
            "full_local_topology_restart"
        },
        changes,
    }
}

fn classify_peer<T: PartialEq>(
    target: &'static str,
    current: &Option<T>,
    next: &Option<T>,
    changes: &mut Vec<ReconfigurationChange>,
) {
    let kind = match (current, next) {
        (None, None) => return,
        (None, Some(_)) => "added",
        (Some(_), None) => "removed",
        (Some(current), Some(next)) if current == next => return,
        (Some(_), Some(_)) => "changed",
    };
    changes.push(change(target, kind, "full_local_topology_restart_required"));
}

const fn change(
    target: &'static str,
    change: &'static str,
    required_action: &'static str,
) -> ReconfigurationChange {
    ReconfigurationChange {
        target,
        change,
        required_action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str) -> WorkspaceProfile {
        WorkspaceProfile {
            version: 1,
            name: name.into(),
            surfaces: vec![pantheon::Surface::Overview],
            keybindings: crate::profile::KeybindingProfile::Pantheon,
            agora: None,
            task_conversation: None,
            moira: None,
            daimon: None,
            hestia: None,
            hermes: None,
            aegis: None,
            tellus: None,
            chronos: None,
            vellum: None,
        }
    }

    #[test]
    fn identical_profiles_require_no_action() {
        let current = profile("local");
        let plan = plan("compare-1".into(), &current, &current);
        assert_eq!(plan.minimum_action, "none");
        assert!(!plan.live_apply_supported);
        assert!(plan.changes.is_empty());
    }

    #[test]
    fn peer_change_requires_full_restart_while_shell_only_change_does_not() {
        let current = profile("old");
        let mut shell = profile("new");
        let shell_plan = plan("compare-2".into(), &current, &shell);
        assert_eq!(shell_plan.minimum_action, "shell_restart");

        shell.tellus = Some(crate::profile::TellusProfile {
            endpoint: "127.0.0.1:4499".parse().unwrap(),
            token: None,
            requires: vec!["tellus.observe.v1".into()],
        });
        let peer_plan = plan("compare-3".into(), &current, &shell);
        assert_eq!(peer_plan.minimum_action, "full_local_topology_restart");
        assert_eq!(peer_plan.changes[1].target, "tellus");
        assert_eq!(peer_plan.changes[1].change, "added");
    }
}
