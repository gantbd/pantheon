//! Pantheon-owned capability resolution for the exact composed route.
//!
//! Peer manifests describe everything a peer presentation package can expose.
//! A Pantheon profile is narrower: it selects one mode, topology and component
//! shape. Requirements are therefore negotiated against this route catalog,
//! never against the peer package's aggregate manifest.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Interactive,
    Headless,
}

impl Mode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Headless => "headless",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Plane {
    Observation,
    Operator,
    Control,
    Management,
}

impl Plane {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Operator => "operator",
            Self::Control => "control",
            Self::Management => "management",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capability {
    pub id: &'static str,
    pub plane: Plane,
}

const fn capability(id: &'static str, plane: Plane) -> Capability {
    Capability { id, plane }
}

fn topology(hosted: bool) -> Capability {
    capability(
        if hosted {
            "topology.host.v1"
        } else {
            "topology.attach.v1"
        },
        Plane::Management,
    )
}

pub fn agora(mode: Mode, hosted: bool) -> Vec<Capability> {
    let mut capabilities = vec![capability("rooms.read.v2", Plane::Observation)];
    if mode == Mode::Interactive {
        capabilities.push(capability("messages.read.v2", Plane::Operator));
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn moira(mode: Mode, hosted: bool) -> Vec<Capability> {
    let mut capabilities = vec![capability("sagas.inspect.v1", Plane::Observation)];
    if mode == Mode::Interactive {
        capabilities.extend([
            capability("sagas.reconciliation.v1", Plane::Operator),
            capability("sagas.evidence.v1", Plane::Operator),
            capability("sagas.operator-receipts.v1", Plane::Operator),
        ]);
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn daimon(mode: Mode, hosted: bool, presence: bool, selected_agent: bool) -> Vec<Capability> {
    let mut capabilities = Vec::new();
    if mode == Mode::Headless {
        if presence {
            capabilities.push(capability("agents.presence.v1", Plane::Observation));
        }
        if selected_agent {
            capabilities.push(capability("agents.session.inspect.v1", Plane::Observation));
        }
    } else if selected_agent {
        if presence {
            capabilities.push(capability("agents.presence.v1", Plane::Observation));
        }
        capabilities.extend([
            capability("agents.session.inspect.v1", Plane::Operator),
            capability("agents.session-set.inspect.v1", Plane::Operator),
            capability("agents.session-set.control.v1", Plane::Operator),
            capability("agents.session-set.prompt.v1", Plane::Operator),
            capability("agents.session-set.permission.v1", Plane::Operator),
            capability("agents.session-set.reattach.v1", Plane::Operator),
        ]);
    } else if presence {
        capabilities.push(capability("agents.presence.v1", Plane::Operator));
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn hermes(mode: Mode, hosted: bool, selected_cursor: bool) -> Vec<Capability> {
    let mut capabilities = vec![capability("ledger.observe.v1", Plane::Observation)];
    if selected_cursor {
        capabilities.push(capability(
            "ledger.cursor.observe.v1",
            if mode == Mode::Headless {
                Plane::Observation
            } else {
                Plane::Operator
            },
        ));
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn aegis(hosted: bool) -> Vec<Capability> {
    vec![
        capability("authority.observe.v1", Plane::Observation),
        topology(hosted),
    ]
}

pub fn grounding() -> Vec<Capability> {
    vec![
        capability("grounding.observe.v1", Plane::Observation),
        topology(false),
    ]
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChronosFeatures {
    pub writes: bool,
    pub scheduling: bool,
    pub watch: bool,
    pub planning: bool,
    pub cancel_receipts: bool,
    pub moira_attempts: bool,
    pub daimon_attempts: bool,
}

pub fn chronos(mode: Mode, hosted: bool, features: ChronosFeatures) -> Vec<Capability> {
    let mut capabilities = vec![capability("tasks.observe.v1", Plane::Observation)];
    if mode == Mode::Interactive {
        if features.writes {
            capabilities.push(capability("tasks.mutate.v1", Plane::Control));
        }
        if features.scheduling {
            capabilities.push(capability("scheduling.observe.v5", Plane::Operator));
            if features.writes {
                capabilities.push(capability("scheduling.mutate.v5", Plane::Control));
            }
        }
        if features.watch {
            capabilities.push(capability("tasks.feed.observe.v3", Plane::Operator));
        }
        if features.planning {
            capabilities.push(capability("planning.observe.v1", Plane::Operator));
        }
        if features.cancel_receipts && features.writes {
            capabilities.push(capability("tasks.cancel-receipt.v3", Plane::Control));
        }
        if features.moira_attempts {
            capabilities.push(capability("attempts.moira.inspect.v1", Plane::Operator));
        }
        if features.daimon_attempts {
            capabilities.push(capability("attempts.daimon.inspect.v1", Plane::Operator));
        }
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn vellum(mode: Mode, hosted: bool, durable_identity: bool) -> Vec<Capability> {
    let mut capabilities = vec![capability("workspace.inventory.v1", Plane::Observation)];
    if mode == Mode::Interactive {
        capabilities.extend([
            capability("workspace.edit.v1", Plane::Operator),
            capability("workspace.save.v1", Plane::Control),
        ]);
        if durable_identity {
            capabilities.extend([
                capability("workspace.durable-save.execute.v1", Plane::Control),
                capability("workspace.durable-save.lookup.v1", Plane::Control),
            ]);
        }
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn negotiate(
    peer: &str,
    mode: Mode,
    required: &[String],
    supported: &[Capability],
) -> Result<(), String> {
    for required in required {
        if !supported.iter().any(|capability| capability.id == required) {
            return Err(format!(
                "{peer} does not deliver required {} route capability {required}",
                mode.label()
            ));
        }
    }
    Ok(())
}

/// Guarded control capabilities. Most remain distinct CLI routes; capabilities
/// composed into an interactive workbench still retain their own confirmation,
/// connection, and authority contract.
pub const GUARDED_CONTROL_CAPABILITIES: &[(&str, &[&str])] = &[
    (
        "aegis",
        &[
            "authority.revoke-epoch.v1",
            "authority.attenuate-authorize-once.v1",
            "aegis.admin-session.v1",
            "authority.issue.v1",
            "authority.attenuate.v1",
        ],
    ),
    ("agora", &["rooms.create.v2", "messages.speak.v2"]),
    (
        "daimon",
        &[
            "agents.session.advance.v1",
            "agents.session.cancel.v1",
            "agents.session.close.v1",
            "agents.session.reattach.v1",
            "agents.session.prompt.v1",
            "agents.session.permission.v1",
        ],
    ),
    (
        "hermes",
        &[
            "ledger.events.disclose.v1",
            "ledger.acknowledge.v1",
            "ledger.compact.v1",
        ],
    ),
    (
        "moira",
        &[
            "sagas.reconciliation.confirmation.v1",
            "sagas.reconciliation.confirm.v1",
            "sagas.reconciliation.recovery.v1",
        ],
    ),
    (
        "vellum",
        &[
            "workspace.durable-save.execute.v1",
            "workspace.durable-save.lookup.v1",
        ],
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardedControlRoute {
    pub peer: &'static str,
    pub capability: &'static str,
    pub command: &'static str,
}

/// Concrete local CLI routes for separately guarded controls. Shared commands
/// retain their peer-owned subcommand or administrative-session semantics.
pub const GUARDED_CONTROL_ROUTES: &[GuardedControlRoute] = &[
    GuardedControlRoute {
        peer: "aegis",
        capability: "authority.revoke-epoch.v1",
        command: "aegis-revoke-epoch",
    },
    GuardedControlRoute {
        peer: "aegis",
        capability: "authority.attenuate-authorize-once.v1",
        command: "aegis-attenuate-authorize-once",
    },
    GuardedControlRoute {
        peer: "aegis",
        capability: "aegis.admin-session.v1",
        command: "aegis-manage-session",
    },
    GuardedControlRoute {
        peer: "aegis",
        capability: "authority.issue.v1",
        command: "aegis-manage-session",
    },
    GuardedControlRoute {
        peer: "aegis",
        capability: "authority.attenuate.v1",
        command: "aegis-manage-session",
    },
    GuardedControlRoute {
        peer: "agora",
        capability: "rooms.create.v2",
        command: "agora-create-room",
    },
    GuardedControlRoute {
        peer: "agora",
        capability: "messages.speak.v2",
        command: "workbench",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.advance.v1",
        command: "daimon-agent-advance",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.cancel.v1",
        command: "daimon-agent-cancel",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.close.v1",
        command: "daimon-agent-close",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.reattach.v1",
        command: "daimon-agent-reattach",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.prompt.v1",
        command: "daimon-agent-prompt",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "agents.session.permission.v1",
        command: "daimon-agent-permission",
    },
    GuardedControlRoute {
        peer: "hermes",
        capability: "ledger.events.disclose.v1",
        command: "hermes-disclose-events",
    },
    GuardedControlRoute {
        peer: "hermes",
        capability: "ledger.acknowledge.v1",
        command: "hermes-acknowledge",
    },
    GuardedControlRoute {
        peer: "hermes",
        capability: "ledger.compact.v1",
        command: "hermes-compact",
    },
    GuardedControlRoute {
        peer: "moira",
        capability: "sagas.reconciliation.confirmation.v1",
        command: "moira-reconciliation-view",
    },
    GuardedControlRoute {
        peer: "moira",
        capability: "sagas.reconciliation.confirm.v1",
        command: "moira-reconciliation-confirm",
    },
    GuardedControlRoute {
        peer: "moira",
        capability: "sagas.reconciliation.recovery.v1",
        command: "moira-reconciliation-recover",
    },
    GuardedControlRoute {
        peer: "vellum",
        capability: "workspace.durable-save.execute.v1",
        command: "vellum-durable-save execute",
    },
    GuardedControlRoute {
        peer: "vellum",
        capability: "workspace.durable-save.lookup.v1",
        command: "vellum-durable-save lookup",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moira_aggregate_does_not_claim_separate_agent_or_control_routes() {
        let interactive = moira(Mode::Interactive, false);
        assert!(
            interactive
                .iter()
                .any(|capability| capability.id == "sagas.evidence.v1")
        );
        for excluded in [
            "agent.session.observe.v1",
            "agent.prompt.submit.v1",
            "sagas.reconciliation.confirm.v1",
        ] {
            assert!(
                !interactive
                    .iter()
                    .any(|capability| capability.id == excluded)
            );
        }
        assert_eq!(
            moira(Mode::Headless, true)
                .iter()
                .map(|capability| capability.id)
                .collect::<Vec<_>>(),
            ["sagas.inspect.v1", "topology.host.v1"]
        );
    }

    #[test]
    fn vellum_durable_save_is_interactive_but_not_headless() {
        let interactive = vellum(Mode::Interactive, false, true);
        for required in [
            "workspace.durable-save.execute.v1",
            "workspace.durable-save.lookup.v1",
        ] {
            assert!(
                interactive
                    .iter()
                    .any(|capability| capability.id == required)
            );
            assert!(
                !vellum(Mode::Interactive, false, false)
                    .iter()
                    .any(|capability| capability.id == required)
            );
        }
        let headless = vellum(Mode::Headless, false, true);
        assert_eq!(
            headless
                .iter()
                .map(|capability| capability.id)
                .collect::<Vec<_>>(),
            ["workspace.inventory.v1", "topology.attach.v1"]
        );
    }

    #[test]
    fn selected_topology_and_features_control_chronos_capabilities() {
        let read = chronos(Mode::Interactive, false, ChronosFeatures::default());
        assert!(
            !read
                .iter()
                .any(|capability| capability.plane == Plane::Control)
        );
        let selected = chronos(
            Mode::Interactive,
            true,
            ChronosFeatures {
                writes: true,
                scheduling: true,
                cancel_receipts: true,
                ..ChronosFeatures::default()
            },
        );
        for expected in [
            "tasks.mutate.v1",
            "scheduling.observe.v5",
            "scheduling.mutate.v5",
            "tasks.cancel-receipt.v3",
            "topology.host.v1",
        ] {
            assert!(selected.iter().any(|capability| capability.id == expected));
        }
    }

    #[test]
    fn daimon_interactive_reports_both_visible_components() {
        let selected = daimon(Mode::Interactive, false, true, true);
        assert!(
            selected
                .iter()
                .any(|capability| capability.id == "agents.session.inspect.v1")
        );
        assert!(
            selected
                .iter()
                .any(|capability| capability.id == "agents.presence.v1")
        );
    }

    #[test]
    fn every_guarded_capability_has_a_concrete_separate_command() {
        for (peer, capabilities) in GUARDED_CONTROL_CAPABILITIES {
            for capability in *capabilities {
                assert!(
                    GUARDED_CONTROL_ROUTES
                        .iter()
                        .any(|route| { route.peer == *peer && route.capability == *capability }),
                    "missing guarded command for {peer} {capability}"
                );
            }
        }
    }
}
