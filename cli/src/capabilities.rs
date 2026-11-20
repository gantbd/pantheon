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

#[derive(Clone, Debug, serde::Serialize)]
pub struct RequirementDisposition {
    pub peer: &'static str,
    pub capability: String,
    pub disposition: &'static str,
    pub reason: &'static str,
}

impl RequirementDisposition {
    pub fn not_exercised(peer: &'static str, capability: String) -> Self {
        Self {
            peer,
            capability,
            disposition: "not_exercised",
            reason: "aggregate_observation_only",
        }
    }
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

pub fn task_conversation(mode: Mode) -> Vec<Capability> {
    let mut capabilities = vec![capability("task-conversation.v1", Plane::Observation)];
    if mode == Mode::Interactive {
        capabilities.push(capability("task-conversation.bind.v1", Plane::Control));
    }
    capabilities.push(topology(false));
    capabilities
}

pub fn moira(
    mode: Mode,
    hosted: bool,
    selected_sagas: bool,
    selected_participation: bool,
    selected_deliberation: bool,
    selected_human_response: bool,
) -> Vec<Capability> {
    let mut capabilities = Vec::new();
    if selected_sagas {
        capabilities.push(capability("sagas.inspect.v1", Plane::Observation));
    }
    if selected_participation {
        capabilities.push(capability(
            "agent.participation.inspect.v1",
            Plane::Observation,
        ));
    }
    if selected_deliberation {
        capabilities.push(capability("deliberation.inspect.v1", Plane::Observation));
    }
    if mode == Mode::Interactive && selected_human_response {
        capabilities.push(capability(
            "deliberation.human-gate.respond.v1",
            Plane::Control,
        ));
    }
    if mode == Mode::Interactive && selected_sagas {
        capabilities.extend([
            capability("sagas.reconciliation.v1", Plane::Operator),
            capability("sagas.evidence.v1", Plane::Operator),
            capability("sagas.operator-receipts.v1", Plane::Operator),
        ]);
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn daimon(
    mode: Mode,
    hosted: bool,
    presence: bool,
    model_endpoint: bool,
    selected_agent: bool,
    session_open: bool,
) -> Vec<Capability> {
    let mut capabilities = Vec::new();
    if mode == Mode::Headless {
        if presence {
            capabilities.push(capability("agents.presence.v1", Plane::Observation));
        }
        if selected_agent {
            capabilities.push(capability("agents.session.inspect.v1", Plane::Observation));
        }
        if model_endpoint {
            capabilities.push(capability("models.endpoint.attach.v1", Plane::Observation));
        }
    } else if selected_agent || session_open {
        if presence {
            capabilities.push(capability("agents.presence.v1", Plane::Observation));
        }
        if selected_agent {
            capabilities.extend([
                capability("agents.session.inspect.v1", Plane::Operator),
                capability("agents.session-set.inspect.v1", Plane::Operator),
                capability("agents.session-set.control.v1", Plane::Operator),
                capability("agents.session-set.prompt.v1", Plane::Operator),
                capability("agents.session-set.permission.v1", Plane::Operator),
                capability("agents.session-set.reattach.v1", Plane::Operator),
            ]);
        }
        if session_open {
            capabilities.push(capability("agents.session.open.v1", Plane::Operator));
        }
    } else {
        if presence {
            capabilities.push(capability("agents.presence.v1", Plane::Operator));
        }
    }
    if mode == Mode::Interactive && model_endpoint {
        capabilities.extend([
            capability("models.endpoint.attach.v1", Plane::Operator),
            capability("models.provider.configure.v1", Plane::Control),
            capability("models.provider.qualify.v1", Plane::Control),
            capability("models.task.invoke.v1", Plane::Control),
        ]);
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
        capabilities.push(capability("ledger.consumer.observe.v1", Plane::Observation));
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

pub fn tellus() -> Vec<Capability> {
    vec![
        capability("tellus.observe.v1", Plane::Observation),
        capability("tellus.sme-plan.v1", Plane::Observation),
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
    pub alignment: bool,
}

pub fn chronos(mode: Mode, hosted: bool, features: ChronosFeatures) -> Vec<Capability> {
    let mut capabilities = vec![capability("tasks.workflow.observe.v9", Plane::Observation)];
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
    if features.alignment {
        capabilities.push(capability(
            "product-alignment.observe.v1",
            Plane::Observation,
        ));
    }
    capabilities.push(topology(hosted));
    capabilities
}

pub fn vellum(mode: Mode, hosted: bool, durable_identity: bool) -> Vec<Capability> {
    let mut capabilities = vec![
        capability("workspace.inventory.v1", Plane::Observation),
        capability("workspace.snapshot.v1", Plane::Observation),
    ];
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

/// Validates a profile against the complete configured workspace route while
/// returning requirements that the finite aggregate observation does not
/// exercise. This keeps an observation adapter from redefining workspace
/// capability truth without pretending that unexercised controls ran.
pub fn negotiate_observation(
    peer: &str,
    required: &[String],
    observed: &[Capability],
    configured: &[Capability],
) -> Result<Vec<String>, String> {
    let mut not_exercised = Vec::new();
    for required in required {
        if observed.iter().any(|capability| capability.id == required) {
            continue;
        }
        if configured
            .iter()
            .any(|capability| capability.id == required)
        {
            not_exercised.push(required.clone());
            continue;
        }
        return Err(format!(
            "{peer} does not deliver required configured workspace capability {required}"
        ));
    }
    Ok(not_exercised)
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
            "models.provider.configure.v1",
            "models.provider.qualify.v1",
            "models.task.invoke.v1",
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
            "agent.open.prepare.v1",
            "agent.worker.step.v1",
            "deliberation.human-gate.respond.v1",
            "task-agent.instantiate.v1",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentToolRoute {
    pub component: &'static str,
    pub capability: &'static str,
    pub command: &'static str,
    pub model_arguments: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentOwnerToolRoute {
    pub component: &'static str,
    pub capability: &'static str,
    pub owner_surface: &'static str,
    pub model_arguments: &'static str,
    pub enforcement: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentToolGap {
    pub component: &'static str,
    pub surface: &'static str,
    pub missing: &'static str,
    pub next_boundary: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleGap {
    pub component: &'static str,
    pub surface: &'static str,
    pub missing: &'static str,
    pub next_boundary: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleProfile {
    pub role: &'static str,
    pub objective: &'static str,
    pub required_tools: &'static [&'static str],
    pub gaps: &'static [AgentRoleGap],
    pub human_gates: &'static [&'static str],
    pub forbidden_authorities: &'static [&'static str],
}

/// Closed gateways qualified for bounded agent profiles. This is intentionally
/// smaller than the human/operator command surface: absence is non-admission.
pub const AGENT_TOOL_ROUTES: &[AgentToolRoute] = &[
    AgentToolRoute {
        component: "agora",
        capability: "rooms.read.v2",
        command: "agora-room-observe",
        model_arguments: "exactly_one_room_id",
    },
    AgentToolRoute {
        component: "agora",
        capability: "messages.read.v2",
        command: "agora-message-observe",
        model_arguments: "one_exact_room_id_cursor_and_limit_up_to_64",
    },
    AgentToolRoute {
        component: "agora",
        capability: "messages.actor-publication.observe.v1",
        command: "agora-message-publication-observe",
        model_arguments: "none_when_exact_private_request_document_is_pinned",
    },
    AgentToolRoute {
        component: "agora",
        capability: "messages.actor-publication.execute.v1",
        command: "agora-message-publish",
        model_arguments: "none_when_exact_private_request_document_and_confirmation_are_pinned",
    },
    AgentToolRoute {
        component: "chronos",
        capability: "tasks.workflow.observe.v9",
        command: "chronos-task-observe",
        model_arguments: "exactly_one_task_id",
    },
    AgentToolRoute {
        component: "chronos",
        capability: "product-alignment.observe.v1",
        command: "chronos-alignment-observe",
        model_arguments: "none_when_owner_directory_and_scope_are_pinned",
    },
    AgentToolRoute {
        component: "tellus",
        capability: "tellus.sme-plan.v1",
        command: "tellus-sme-plan",
        model_arguments: "none_when_session_pinned",
    },
    AgentToolRoute {
        component: "vellum",
        capability: "workspace.snapshot.v1",
        command: "vellum-document-read",
        model_arguments: "one_admitted_workspace_locator_and_optional_utf8_byte_range",
    },
    AgentToolRoute {
        component: "moira",
        capability: "moira.task-agent.catalogue.v1",
        command: "moira-task-agent-catalogue",
        model_arguments: "none_when_session_pinned",
    },
    AgentToolRoute {
        component: "moira",
        capability: "moira.task-agent.observe.v1",
        command: "moira-task-agent-observe",
        model_arguments: "one_owner_id_and_one_journey_sequence",
    },
    AgentToolRoute {
        component: "moira",
        capability: "moira.deliberation.observe.v1",
        command: "moira-deliberation-observe",
        model_arguments: "exactly_one_deliberation_id",
    },
    AgentToolRoute {
        component: "task-conversation",
        capability: "task-conversation.v1",
        command: "task-conversation-inspect",
        model_arguments: "one_exact_task_or_conversation_reference",
    },
    AgentToolRoute {
        component: "daimon",
        capability: "agents.session.inspect.v1",
        command: "daimon-agent-observe",
        model_arguments: "exactly_one_agent_session_id",
    },
    AgentToolRoute {
        component: "hestia",
        capability: "runtimes.observe.v1",
        command: "hestia-runtime-observe",
        model_arguments: "exactly_one_runtime_id",
    },
    AgentToolRoute {
        component: "hestia",
        capability: "artifacts.observe.v1",
        command: "hestia-artifact-observe",
        model_arguments: "exactly_one_artifact_id",
    },
    AgentToolRoute {
        component: "hestia",
        capability: "artifacts.catalog.v1",
        command: "hestia-artifact-catalog",
        model_arguments: "none_when_owner_socket_pinned",
    },
    AgentToolRoute {
        component: "hermes",
        capability: "ledger.events.disclose.v1",
        command: "hermes-disclose-events",
        model_arguments: "one_stream_start_limit_and_exact_disclosure_confirmation",
    },
    AgentToolRoute {
        component: "external-web",
        capability: "agent.web.fetch.v1",
        command: "agent-web-fetch",
        model_arguments: "exactly_one_https_url",
    },
    AgentToolRoute {
        component: "verification",
        capability: "verification.result-envelope.validate.v1",
        command: "verification-result-validate",
        model_arguments: "none_when_exact_private_request_document_is_pinned",
    },
];

/// Component-native tool routes already enforced by their sovereign owner.
/// These are not Pantheon CLI commands and must never be restyled as such.
pub const AGENT_OWNER_TOOL_ROUTES: &[AgentOwnerToolRoute] = &[AgentOwnerToolRoute {
    component: "daimon",
    capability: "verification.test.run.v1",
    owner_surface: "acp_terminal.commands[]",
    model_arguments: "select_one_catalog_entry_with_no_appended_arguments",
    enforcement: "Daimon canonicalizes the workspace, boundary and executable; Aegis reauthorizes the command-specific capability; the retained argument prefix is the complete argv; the environment and UTF-8 output are bounded",
}];

/// Named component surfaces that must not be inferred from aggregate Pantheon
/// observation. Keeping these machine-visible makes absence an explicit
/// non-admission rather than a discoverability failure.
pub const AGENT_TOOL_GAPS: &[AgentToolGap] = &[AgentToolGap {
    component: "oculus",
    surface: "owner-query-runtime",
    missing: "owner service, public client, source catalogue runtime, and bounded query settlement transport",
    next_boundary: "finish the sovereign Oculus catalogue/query owner before admitting source, window, frontier, sampling, gap, disclosure, and partial-result observations",
}];

const ROLE_ACTIVATION_GAP: AgentRoleGap = AgentRoleGap {
    component: "moira-aegis-progression",
    surface: "task-agent.activation.v1",
    missing: "trusted workflow-relationship and deliberation-member observation, durable prepare-before-dispatch, composition of Moira's qualified fresh command-scoped Aegis participation authority, participation-owner admission and lost-reply reconciliation, subsequent worker/session lifecycle and completed-work receipts; the qualified same-client room path also does not yet retain standalone-process non-bearer Aegis decision attribution",
    next_boundary: "compose the workflow-relationship and deliberation observers, persist the qualified participation blueprint before dispatch, invoke the existing exact-identity Aegis authority freshly, dispatch and reconcile the participation-owner result, retain only non-bearer decision attribution, then prepare the Daimon open effect",
};

const WORKER_LIFECYCLE_GAP: AgentRoleGap = AgentRoleGap {
    component: "daimon",
    surface: "persistent-role-worker-lifecycle",
    missing: "an owner-bound start, stop, reattach, schedule, and completed-work receipt for the activated role",
    next_boundary: "compose a frozen provider/profile and exact Running Chronos attempt through Daimon and retain owner receipts",
};

const PRODUCT_MANAGER_GAPS: &[AgentRoleGap] = &[
    ROLE_ACTIVATION_GAP,
    AgentRoleGap {
        component: "vellum",
        surface: "workspace.proposal.publish.v1",
        missing: "changed PM-authored immutable proposal publication with author, source revision, digest, and recovery receipt through the existing owner composition",
        next_boundary: "extend the qualified one-shot Chronos/Vellum proposal process with bounded authoring and unchanged-frontier coalescing rather than arbitrary filesystem write",
    },
    WORKER_LIFECYCLE_GAP,
];

const SRE_GAPS: &[AgentRoleGap] = &[
    ROLE_ACTIVATION_GAP,
    AgentRoleGap {
        component: "oculus",
        surface: "owner-query-runtime",
        missing: "owner service, public client, populated source catalogue, bounded query settlement transport, and source/cursor/gap/sampling projections",
        next_boundary: "finish and qualify the sovereign Oculus catalogue/query owner; Hermes events or shell output are not a substitute",
    },
    AgentRoleGap {
        component: "hermes",
        surface: "event-trigger-subscription.v1",
        missing: "a durable bounded trigger cursor with coalescing, schedule, replay, and delivery evidence",
        next_boundary: "add owner-bound event/schedule triggering; bounded disclosure alone is not a persistent worker subscription",
    },
    AgentRoleGap {
        component: "chronos",
        surface: "incident-admission.propose.v1",
        missing: "a typed proposal that links recurring evidence to existing work or prepares a new admission without setting priority or declaring an incident",
        next_boundary: "publish a Chronos-owned proposal contract and keep task admission plus priority human/owner gated",
    },
    WORKER_LIFECYCLE_GAP,
];

const FRONTIER_GAPS: &[AgentRoleGap] = &[
    ROLE_ACTIVATION_GAP,
    AgentRoleGap {
        component: "daimon",
        surface: "provider-host-tool-continuation.v1",
        missing: "a real provider-specific host-tool continuation with callback authorization, denial matrix, restart, and content-safe telemetry evidence",
        next_boundary: "qualify one Codex or Claude provider journey end to end; ACP evidence is not portable by analogy",
    },
    AgentRoleGap {
        component: "aegis",
        surface: "command-binding-evidence.v1",
        missing: "current opaque per-command grants bound to the same semantic capabilities, task attempt, principal, policy epoch, and expiry",
        next_boundary: "revalidate every tool callback through Aegis and retain the owner decision without disclosing bearer material",
    },
    WORKER_LIFECYCLE_GAP,
];

const VERIFICATION_GAPS: &[AgentRoleGap] = &[
    ROLE_ACTIVATION_GAP,
    AgentRoleGap {
        component: "moira",
        surface: "tests.result.publish.v1",
        missing: "a qualified owner composition that either governs test dispatch through Moira or admits a versioned external Daimon verification receipt without rewriting execution history",
        next_boundary: "select one honest owner path: run the test under Moira's attempt/gate/profile/output lifecycle, or add a distinct Moira receipt bound to the exact validated Daimon evidence; never relabel a post-hoc continuation as provenance",
    },
    AgentRoleGap {
        component: "task-to-session-runtime",
        surface: "verification-worker.compose.v1",
        missing: "a replay-safe runtime that supplies owner-derived task identity and revision to Pantheon's retained Moira/Chronos worker, then binds the exact verification execution and result before settlement",
        next_boundary: "reuse the existing private worker claim/bind/settle lifecycle; add operation identity, lost-reply lookup, cancellation and restart receipts instead of creating another scheduler",
    },
    WORKER_LIFECYCLE_GAP,
];

pub const AGENT_ROLE_PROFILES: &[AgentRoleProfile] = &[
    AgentRoleProfile {
        role: "product-manager",
        objective: "sample current owner facts, coalesce unchanged alignment, and publish attributed recommendations without applying material decisions",
        required_tools: &[
            "tasks.workflow.observe.v9",
            "product-alignment.observe.v1",
            "tellus.sme-plan.v1",
            "workspace.snapshot.v1",
            "moira.task-agent.observe.v1",
            "moira.deliberation.observe.v1",
            "rooms.read.v2",
            "messages.read.v2",
            "messages.actor-publication.observe.v1",
            "messages.actor-publication.execute.v1",
            "task-conversation.v1",
            "ledger.events.disclose.v1",
            "agents.session.inspect.v1",
        ],
        gaps: PRODUCT_MANAGER_GAPS,
        human_gates: &[
            "goal_or_scope_change",
            "priority_or_acceptance_change",
            "rollout_or_stage_decision",
            "ensemble_adequacy_dispute",
        ],
        forbidden_authorities: &[
            "apply_own_recommendation",
            "mutate_chronos",
            "approve_or_sign_off",
            "transfer_task_lease",
            "grant_tools_or_role_authority",
        ],
    },
    AgentRoleProfile {
        role: "sre",
        objective: "observe bounded ELMT evidence, coalesce repeats, and propose an existing-work link or new admission packet without declaring or closing an incident",
        required_tools: &[
            "tasks.workflow.observe.v9",
            "tellus.sme-plan.v1",
            "workspace.snapshot.v1",
            "moira.task-agent.observe.v1",
            "rooms.read.v2",
            "messages.read.v2",
            "messages.actor-publication.observe.v1",
            "messages.actor-publication.execute.v1",
            "task-conversation.v1",
            "ledger.events.disclose.v1",
            "agents.session.inspect.v1",
        ],
        gaps: SRE_GAPS,
        human_gates: &[
            "incident_declaration_or_page",
            "priority_or_scope_change",
            "risk_acceptance",
            "work_closure",
        ],
        forbidden_authorities: &[
            "infer_oculus_from_logs_or_shell",
            "declare_incident",
            "page_human",
            "set_priority",
            "close_work",
        ],
    },
    AgentRoleProfile {
        role: "frontier-collaborator",
        objective: "perform one complex owner-bound task attempt using only independently admitted Pantheon component gateways",
        required_tools: &[
            "tasks.workflow.observe.v9",
            "tellus.sme-plan.v1",
            "workspace.snapshot.v1",
            "moira.task-agent.observe.v1",
            "rooms.read.v2",
            "messages.read.v2",
            "messages.actor-publication.observe.v1",
            "messages.actor-publication.execute.v1",
            "task-conversation.v1",
            "agents.session.inspect.v1",
            "runtimes.observe.v1",
            "artifacts.observe.v1",
            "artifacts.catalog.v1",
            "agent.web.fetch.v1",
        ],
        gaps: FRONTIER_GAPS,
        human_gates: &[
            "authority_escalation",
            "material_scope_or_acceptance_change",
            "rollout_or_irreversible_effect",
        ],
        forbidden_authorities: &[
            "shell_or_interpreter",
            "ambient_filesystem",
            "ambient_network",
            "inherited_secrets",
            "undeclared_mcp_or_pantheon_aggregate",
        ],
    },
    AgentRoleProfile {
        role: "verification-worker",
        objective: "asynchronously execute an exact allowlisted verification profile and publish evidence against one source revision and task attempt",
        required_tools: &[
            "tasks.workflow.observe.v9",
            "workspace.snapshot.v1",
            "moira.task-agent.observe.v1",
            "messages.actor-publication.observe.v1",
            "messages.actor-publication.execute.v1",
            "task-conversation.v1",
            "agents.session.inspect.v1",
            "runtimes.observe.v1",
            "artifacts.observe.v1",
            "verification.test.run.v1",
            "verification.result-envelope.validate.v1",
        ],
        gaps: VERIFICATION_GAPS,
        human_gates: &[
            "acceptance_disposition",
            "test_scope_expansion",
            "release_or_rollout_decision",
        ],
        forbidden_authorities: &[
            "arbitrary_test_command",
            "shell_or_package_manager",
            "accept_own_result",
            "mutate_source",
            "close_work",
        ],
    },
];

pub fn agent_role_profile(role: &str) -> Option<&'static AgentRoleProfile> {
    AGENT_ROLE_PROFILES
        .iter()
        .find(|profile| profile.role == role)
}

/// Concrete local CLI routes for separately guarded controls. Shared commands
/// retain their peer-owned subcommand or administrative-session semantics.
pub const GUARDED_CONTROL_ROUTES: &[GuardedControlRoute] = &[
    GuardedControlRoute {
        peer: "moira",
        capability: "deliberation.human-gate.respond.v1",
        command: "workbench",
    },
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
        capability: "agents.session.open.v1",
        command: "daimon-agent-open",
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
        peer: "daimon",
        capability: "models.provider.configure.v1",
        command: "workbench",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "models.provider.qualify.v1",
        command: "workbench",
    },
    GuardedControlRoute {
        peer: "daimon",
        capability: "models.task.invoke.v1",
        command: "workbench",
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
        capability: "agent.open.prepare.v1",
        command: "workbench",
    },
    GuardedControlRoute {
        peer: "moira",
        capability: "agent.worker.step.v1",
        command: "moira-participation-worker-run",
    },
    GuardedControlRoute {
        peer: "moira",
        capability: "task-agent.instantiate.v1",
        command: "moira-task-agent-instantiate",
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
    fn agent_tools_are_operation_specific_and_non_aggregate() {
        let mut capabilities = std::collections::BTreeSet::new();
        let mut commands = std::collections::BTreeSet::new();
        for route in AGENT_TOOL_ROUTES {
            assert_ne!(route.component, "pantheon");
            assert!(!route.capability.starts_with("pantheon."));
            assert_ne!(route.command, "headless");
            assert!(!route.model_arguments.is_empty());
            assert!(capabilities.insert(route.capability));
            assert!(commands.insert(route.command));
        }
        for route in AGENT_OWNER_TOOL_ROUTES {
            assert_ne!(route.component, "pantheon");
            assert!(!route.capability.starts_with("pantheon."));
            assert!(!route.owner_surface.is_empty());
            assert!(!route.model_arguments.is_empty());
            assert!(!route.enforcement.is_empty());
            assert!(capabilities.insert(route.capability));
        }
        for gap in AGENT_TOOL_GAPS {
            assert_ne!(gap.component, "pantheon");
            assert!(!gap.surface.is_empty());
            assert!(!gap.missing.is_empty());
            assert!(!gap.next_boundary.is_empty());
            assert!(!AGENT_TOOL_ROUTES.iter().any(|route| {
                route.component == gap.component && route.capability == gap.surface
            }));
        }
    }

    #[test]
    fn agent_role_profiles_are_least_authority_and_reference_real_tools() {
        let mut catalogued = AGENT_TOOL_ROUTES
            .iter()
            .map(|route| route.capability)
            .collect::<std::collections::BTreeSet<_>>();
        catalogued.extend(AGENT_OWNER_TOOL_ROUTES.iter().map(|route| route.capability));
        let mut roles = std::collections::BTreeSet::new();
        for profile in AGENT_ROLE_PROFILES {
            assert!(roles.insert(profile.role));
            assert!(!profile.objective.is_empty());
            assert!(!profile.required_tools.is_empty());
            assert!(!profile.gaps.is_empty());
            assert!(!profile.human_gates.is_empty());
            assert!(!profile.forbidden_authorities.is_empty());
            let mut requirements = std::collections::BTreeSet::new();
            for requirement in profile.required_tools {
                assert!(catalogued.contains(requirement));
                assert!(requirements.insert(requirement));
                assert!(!requirement.starts_with("pantheon."));
            }
            let mut gaps = std::collections::BTreeSet::new();
            for gap in profile.gaps {
                assert!(gaps.insert((gap.component, gap.surface)));
                assert!(!gap.missing.is_empty());
                assert!(!gap.next_boundary.is_empty());
            }
        }
        assert_eq!(
            roles,
            [
                "frontier-collaborator",
                "product-manager",
                "sre",
                "verification-worker",
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn role_activation_never_models_volatile_aegis_grants_as_durable_state() {
        assert!(
            ROLE_ACTIVATION_GAP
                .missing
                .contains("composition of Moira's qualified fresh command-scoped Aegis")
        );
        assert!(
            ROLE_ACTIVATION_GAP
                .next_boundary
                .contains("invoke the existing exact-identity Aegis authority freshly")
        );
        assert!(
            ROLE_ACTIVATION_GAP
                .next_boundary
                .contains("retain only non-bearer decision attribution")
        );
        assert!(
            !ROLE_ACTIVATION_GAP
                .missing
                .contains("current Aegis delegation")
        );
        assert!(
            !ROLE_ACTIVATION_GAP
                .next_boundary
                .contains("current Aegis delegation")
        );
    }

    #[test]
    fn observation_negotiation_preserves_but_does_not_exercise_workspace_controls() {
        let observed = daimon(Mode::Headless, false, true, false, true, false);
        let configured = daimon(Mode::Interactive, false, true, false, true, false);
        let required = vec![
            "agents.session.inspect.v1".to_owned(),
            "agents.session-set.prompt.v1".to_owned(),
        ];
        assert_eq!(
            negotiate_observation("daimon", &required, &observed, &configured).unwrap(),
            ["agents.session-set.prompt.v1"]
        );
        assert_eq!(
            negotiate_observation(
                "daimon",
                &["agents.session-set.unknown.v1".to_owned()],
                &observed,
                &configured,
            )
            .unwrap_err(),
            "daimon does not deliver required configured workspace capability agents.session-set.unknown.v1"
        );
    }

    #[test]
    fn moira_aggregate_does_not_claim_separate_agent_or_control_routes() {
        let interactive = moira(Mode::Interactive, false, true, false, false, false);
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
            moira(Mode::Headless, true, true, false, false, false)
                .iter()
                .map(|capability| capability.id)
                .collect::<Vec<_>>(),
            ["sagas.inspect.v1", "topology.host.v1"]
        );
    }

    #[test]
    fn moira_participation_is_advertised_only_when_selected() {
        assert!(
            moira(Mode::Interactive, false, false, true, false, false)
                .iter()
                .any(|capability| capability.id == "agent.participation.inspect.v1")
        );
        assert!(
            !moira(Mode::Headless, false, false, false, false, false)
                .iter()
                .any(|capability| capability.id == "agent.participation.inspect.v1")
        );
    }

    #[test]
    fn moira_deliberation_is_advertised_only_when_selected() {
        assert!(
            moira(Mode::Interactive, false, false, false, true, false)
                .iter()
                .any(|capability| capability.id == "deliberation.inspect.v1")
        );
        assert!(
            !moira(Mode::Headless, false, false, false, false, false)
                .iter()
                .any(|capability| capability.id == "deliberation.inspect.v1")
        );
    }

    #[test]
    fn moira_human_response_is_interactive_and_explicitly_selected() {
        assert!(
            moira(Mode::Interactive, false, false, false, true, true)
                .iter()
                .any(|capability| capability.id == "deliberation.human-gate.respond.v1")
        );
        assert!(
            !moira(Mode::Headless, false, false, false, true, true)
                .iter()
                .any(|capability| capability.id == "deliberation.human-gate.respond.v1")
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
            [
                "workspace.inventory.v1",
                "workspace.snapshot.v1",
                "topology.attach.v1"
            ]
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
        let aligned = chronos(
            Mode::Headless,
            false,
            ChronosFeatures {
                alignment: true,
                ..ChronosFeatures::default()
            },
        );
        assert!(
            aligned
                .iter()
                .any(|capability| capability.id == "product-alignment.observe.v1")
        );
    }

    #[test]
    fn daimon_interactive_reports_both_visible_components() {
        let selected = daimon(Mode::Interactive, false, true, false, true, false);
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
    fn daimon_session_open_does_not_invent_a_preexisting_session() {
        let selected = daimon(Mode::Interactive, false, false, false, false, true);
        assert!(
            selected
                .iter()
                .any(|capability| capability.id == "agents.session.open.v1")
        );
        assert!(
            !selected
                .iter()
                .any(|capability| capability.id == "agents.session.inspect.v1")
        );
        assert!(GUARDED_CONTROL_ROUTES.iter().any(|route| {
            route.peer == "daimon"
                && route.capability == "agents.session.open.v1"
                && route.command == "daimon-agent-open"
        }));
    }

    #[test]
    fn local_model_interactive_reports_the_governed_task_invocation_control() {
        let selected = daimon(Mode::Interactive, true, false, true, false, false);
        assert!(
            selected
                .iter()
                .any(|capability| capability.id == "models.task.invoke.v1")
        );
        assert!(GUARDED_CONTROL_ROUTES.iter().any(|route| {
            route.peer == "daimon"
                && route.capability == "models.task.invoke.v1"
                && route.command == "workbench"
        }));
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
