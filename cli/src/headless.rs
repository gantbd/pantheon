//! Headless peer observation shared by the standalone CLI and deployments.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
use chronos_tui::Workstream;
use hestia_local_client::LocalClient as HestiaLocalClient;
use moira_client::{AgentParticipationReadOnlyClient, SagaReadOnlyClient};
use moira_protocol::{
    SagaOutcomeV1, SagaReplyV1, agent_participation as participation,
    deliberation as deliberation_protocol,
};
use moira_vellum::{DecisionPacketAdapter, DocumentRouteV1, ExternalDocumentRefV1};
use sha2::{Digest, Sha256};
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};

use crate::capabilities::RequirementDisposition;
use crate::hestia::observe_provider_runtime;
use crate::management::PeerRuntime;
use crate::profile::{
    AgoraProfile, AgoraTransportProfile, AlignmentBinding, DaimonAgentOpenProfile,
    DaimonExecutionPolicyProfile, DaimonHostProfile, HestiaProfile,
    MoiraDecisionPacketReaderProfile, TaskConversationProfile,
};
use crate::provider::provider_catalog;
use pantheon::{
    AgendaActionability, AgendaEntry, AgendaKind, AgendaUrgency, ChronosAttentionFact,
    OperatorAgendaView, alignment_drift_entry, alignment_human_gate_entry, chronos_attention_entry,
};

pub struct HeadlessRequest {
    pub operation_id: String,
    pub workspace: String,
    pub agora: Option<AgoraProfile>,
    pub agora_hosted: bool,
    pub task_conversation: Option<TaskConversationProfile>,
    pub moira_socket: Option<PathBuf>,
    pub moira_participation_socket: Option<PathBuf>,
    pub moira_participation: Option<participation::ParticipationIdV1>,
    pub moira_deliberation_socket: Option<PathBuf>,
    pub moira_deliberation: Option<deliberation_protocol::DeliberationIdV1>,
    pub moira_decision_packet_reader: Option<MoiraDecisionPacketReaderProfile>,
    pub moira_hosted: bool,
    pub daimon_socket: Option<PathBuf>,
    pub daimon_model_socket: Option<PathBuf>,
    pub daimon_agent_socket: Option<PathBuf>,
    pub daimon_agent_session: Option<daimon_protocol::AgentSessionIdV1>,
    pub daimon_agent_open: Option<DaimonAgentOpenProfile>,
    pub daimon_execution_policy: Option<DaimonExecutionPolicyProfile>,
    pub daimon_provider_owner: Option<DaimonHostProfile>,
    pub daimon_hosted: bool,
    pub hestia: Option<HestiaProfile>,
    pub hermes_socket: Option<PathBuf>,
    pub hermes_stream: Option<hermes_protocol::StreamIdV1>,
    pub hermes_consumer: Option<hermes_protocol::ConsumerIdV1>,
    pub hermes_consumer_access: bool,
    pub hermes_hosted: bool,
    pub aegis_socket: Option<PathBuf>,
    pub aegis_hosted: bool,
    pub tellus_endpoint: Option<SocketAddr>,
    pub tellus_token: Option<PathBuf>,
    pub chronos: Vec<Workstream>,
    pub chronos_alignment: Vec<AlignmentBinding>,
    pub chronos_hosted: bool,
    pub vellum_socket: Option<PathBuf>,
    pub vellum_credential: Option<PathBuf>,
    pub vellum_hosted: bool,
    pub not_exercised_profile_requirements: Vec<RequirementDisposition>,
}

#[derive(Debug)]
pub struct HeadlessObservation {
    pub operation_id: String,
    pub workspace: String,
    pub complete: bool,
    pub peers: Vec<serde_json::Value>,
    pub provider_configuration: Option<serde_json::Value>,
    pub operator_agenda: OperatorAgendaView,
    pub management: Vec<PeerRuntime>,
    pub not_exercised_profile_requirements: Vec<RequirementDisposition>,
}

/// Observes every configured peer through its ordinary public client without
/// constructing presentation state. Healthy peer evidence remains present when
/// another peer refuses or has an uncertain transport outcome.
pub fn observe(request: HeadlessRequest) -> Result<HeadlessObservation, String> {
    let mut peers = Vec::new();
    let mut management = Vec::new();
    let mut complete = true;
    let provider_configuration = request.daimon_provider_owner.as_ref().map(|owner| {
        match provider_catalog(&owner.executable, &owner.config) {
            Ok(catalog) => serde_json::json!({
                "status": "current",
                "catalog": catalog,
                "boundary": "commands, arguments, paths, credential names, and secrets remain redacted by Daimon",
            }),
            Err(detail) => {
                complete = false;
                serde_json::json!({
                    "status": "refused",
                    "detail": detail,
                    "next_action": "preflight the selected Daimon owner configuration and retry the aggregate observation",
                    "boundary": "no fallback provider file was inspected and no provider state was inferred from a socket",
                })
            }
        }
    });

    if let Some(profile) = request.agora {
        let (observation, current) = observe_agora(profile);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.agora_hosted,
        );
    }
    if let Some(profile) = request.task_conversation {
        let routing = task_conversation_routing_projection(&profile);
        let (mut observation, current) = observe_task_conversation(profile.socket);
        if let Some(object) = observation.as_object_mut() {
            object.insert("agent_routing".into(), routing);
            object.insert(
                "routing_boundary".into(),
                serde_json::json!("participant-to-session routing is delivery configuration; it grants neither Agora membership, stakeholder standing, prompt authority, nor task disposition"),
            );
        }
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    if let Some(socket) = request.moira_socket {
        let (observation, current) = observe_moira(socket);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.moira_hosted,
        );
    }
    if let (Some(socket), Some(participation)) = (
        request.moira_participation_socket,
        request.moira_participation,
    ) {
        let (observation, current) = observe_moira_participation(socket, participation);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    if let (Some(socket), Some(deliberation)) = (
        request.moira_deliberation_socket,
        request.moira_deliberation,
    ) {
        let (mut observation, mut current, packet_reference) =
            observe_moira_deliberation(socket, deliberation);
        if let (Some(reader), Some(reference), Some(vellum_socket)) = (
            request.moira_decision_packet_reader.as_ref(),
            packet_reference,
            request.vellum_socket.as_ref(),
        ) && let Some(object) = observation.as_object_mut()
        {
            let (packet, packet_current) = observe_decision_packet(
                vellum_socket,
                request.vellum_credential.as_deref(),
                reader,
                reference,
            );
            current &= packet_current;
            object.insert("decision_packet_reader".into(), packet);
        }
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    if request.daimon_socket.is_some()
        || request.daimon_model_socket.is_some()
        || request.daimon_agent_socket.is_some()
    {
        let (observation, current) = observe_daimon(
            request.daimon_socket,
            request.daimon_model_socket,
            request.daimon_agent_socket,
            request.daimon_agent_session,
            request.daimon_agent_open.as_ref(),
            request.daimon_execution_policy.as_ref(),
        );
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.daimon_hosted,
        );
    }
    if let Some(hestia) = request.hestia {
        let (observation, current) = observe_hestia(hestia, request.daimon_provider_owner.as_ref());
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    if let Some(socket) = request.hermes_socket {
        let (observation, current) = observe_hermes(
            socket,
            request.hermes_stream,
            request.hermes_consumer,
            request.hermes_consumer_access,
        );
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.hermes_hosted,
        );
    }
    if let Some(socket) = request.aegis_socket {
        let (observation, current) = observe_aegis(socket);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.aegis_hosted,
        );
    }
    if let Some(endpoint) = request.tellus_endpoint {
        let (observation, current) = observe_tellus(endpoint, request.tellus_token.as_ref());
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    for workstream in request.chronos {
        let (observation, current) = observe_chronos(workstream);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.chronos_hosted,
        );
    }
    for binding in request.chronos_alignment {
        let (observation, current) = observe_chronos_alignment(binding);
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            false,
        );
    }
    if let Some(socket) = request.vellum_socket {
        let (observation, current) = observe_vellum(socket, request.vellum_credential.as_ref());
        record_observation(
            &mut peers,
            &mut management,
            &mut complete,
            observation,
            current,
            request.vellum_hosted,
        );
    }
    if peers.is_empty() {
        return Err("headless observation requires at least one configured peer".into());
    }
    let operator_agenda = project_operator_agenda(&peers);
    Ok(HeadlessObservation {
        operation_id: request.operation_id,
        workspace: request.workspace,
        complete,
        peers,
        provider_configuration,
        operator_agenda,
        management,
        not_exercised_profile_requirements: request.not_exercised_profile_requirements,
    })
}

fn observe_hestia(
    profile: HestiaProfile,
    daimon_owner: Option<&DaimonHostProfile>,
) -> (serde_json::Value, bool) {
    let mut runtimes = Vec::new();
    let mut current = true;
    if !profile.runtime_bindings.is_empty() {
        let catalog = match daimon_owner {
            Some(owner) => match provider_catalog(&owner.executable, &owner.config) {
                Ok(catalog) => Some(catalog),
                Err(detail) => {
                    current = false;
                    runtimes.push(serde_json::json!({
                        "status": "provider_catalog_refused",
                        "detail": detail,
                    }));
                    None
                }
            },
            None => {
                current = false;
                runtimes.push(serde_json::json!({
                    "status": "configuration_refused",
                    "next_action": "select the exact Daimon owner used by the Hestia provider bindings",
                }));
                None
            }
        };
        for binding in catalog.iter().flat_map(|_| profile.runtime_bindings.iter()) {
            let Some(provider) = catalog
                .as_ref()
                .expect("binding iteration requires provider catalog")
                .providers
                .iter()
                .find(|provider| provider.provider_id == binding.provider_id)
            else {
                current = false;
                runtimes.push(serde_json::json!({
                    "provider_id": binding.provider_id,
                    "runtime_id": binding.runtime_id,
                    "status": "provider_missing",
                }));
                continue;
            };
            let Some(model) = provider.model.as_deref() else {
                current = false;
                runtimes.push(serde_json::json!({
                    "provider_id": binding.provider_id,
                    "runtime_id": binding.runtime_id,
                    "status": "model_missing",
                }));
                continue;
            };
            match observe_provider_runtime(&profile, &binding.provider_id, model) {
                Ok(evidence) => {
                    current &= evidence.invocation_eligible;
                    runtimes.push(serde_json::json!({
                        "status": if evidence.invocation_eligible { "ready" } else { "not_ready" },
                        "evidence": evidence,
                    }));
                }
                Err(detail) => {
                    current = false;
                    runtimes.push(serde_json::json!({
                        "provider_id": binding.provider_id,
                        "runtime_id": binding.runtime_id,
                        "status": "observation_refused",
                        "detail": detail,
                    }));
                }
            }
        }
    }

    let artifacts = if profile.artifact_catalog {
        match HestiaLocalClient::new(profile.socket.clone()).catalog_artifacts() {
            Ok(catalog) => serde_json::json!({
                "status": "current",
                "catalog": catalog,
                "boundary": "artifact admission does not instantiate, qualify, invoke, or schedule a model",
            }),
            Err(error) => {
                current = false;
                serde_json::json!({
                    "status": "observation_refused",
                    "detail": format!("{error:?}"),
                })
            }
        }
    } else {
        serde_json::Value::Null
    };
    hestia_observation(&profile, runtimes, artifacts, current)
}

fn hestia_observation(
    profile: &HestiaProfile,
    runtimes: Vec<serde_json::Value>,
    artifacts: serde_json::Value,
    current: bool,
) -> (serde_json::Value, bool) {
    (
        serde_json::json!({
            "peer": "hestia",
            "capability": if profile.runtime_bindings.is_empty() {
                "artifacts.catalog.v1"
            } else {
                "runtimes.observe.v1"
            },
            "capabilities": {
                "runtimes.observe.v1": !profile.runtime_bindings.is_empty(),
                "artifacts.catalog.v1": profile.artifact_catalog,
            },
            "status": if current { "current" } else { "partial" },
            "runtimes": runtimes,
            "artifacts": artifacts,
            "boundary": "Hestia publishes runtime lifecycle and artifact admission; Daimon separately publishes provider qualification and execution",
        }),
        current,
    )
}

fn project_operator_agenda(peers: &[serde_json::Value]) -> OperatorAgendaView {
    let mut agenda = OperatorAgendaView::default();
    let direct_moira_gate = peers.iter().any(|peer| {
        peer.get("peer").and_then(serde_json::Value::as_str) == Some("moira-deliberation")
            && peer.get("status").and_then(serde_json::Value::as_str) == Some("current")
            && peer
                .get("deliberation")
                .and_then(|value| value.get("human_gate"))
                .and_then(|gate| gate.get("status"))
                .and_then(serde_json::Value::as_str)
                == Some("pending")
    });
    for peer in peers {
        let Some(name) = peer.get("peer").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let status = peer
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        if name == "chronos" {
            let instance = peer
                .get("instance")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let source = format!("chronos/{instance}");
            if status != "current" {
                agenda.unavailable_sources.push(source);
                continue;
            }
            let revision = peer
                .get("revision")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            for task in peer
                .get("agenda_items")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(task_id) = task.get("task").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                let incarnation = task
                    .get("incarnation")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let status = task
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let execution_phase = task
                    .get("execution_phase")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let dependency_count = task
                    .get("dependency_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let priority = task
                    .get("priority")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                if let Some(entry) = chronos_attention_entry(ChronosAttentionFact {
                    workstream: instance,
                    revision,
                    task: task_id,
                    incarnation,
                    status,
                    execution_phase,
                    priority,
                    dependency_count,
                }) {
                    agenda.push(entry);
                }
            }
        } else if name == "chronos-alignment" {
            let instance = peer
                .get("instance")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let source = format!("chronos-alignment/{instance}");
            let Some(observation) = (status == "current")
                .then(|| peer.get("observation"))
                .flatten()
            else {
                agenda.unavailable_sources.push(source);
                continue;
            };
            let Some(record_revision) = observation
                .get("record_revision")
                .and_then(serde_json::Value::as_str)
            else {
                agenda.unavailable_sources.push(source);
                continue;
            };
            let Some(currentness) = observation
                .get("currentness")
                .and_then(serde_json::Value::as_str)
            else {
                agenda.unavailable_sources.push(source);
                continue;
            };
            let Some(proposal) = observation.get("proposal") else {
                agenda.unavailable_sources.push(source);
                continue;
            };
            let (Some(namespace), Some(document)) = (
                proposal
                    .get("namespace")
                    .and_then(serde_json::Value::as_str),
                proposal.get("document").and_then(serde_json::Value::as_str),
            ) else {
                agenda.unavailable_sources.push(source);
                continue;
            };
            if let Some(entry) = alignment_drift_entry(
                instance,
                record_revision,
                &format!("{namespace}/{document}"),
                currentness,
            ) {
                agenda.push(entry);
            }
            if !direct_moira_gate {
                for gate in observation
                    .get("pending_human_gates")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let (Some(owner), Some(resource), Some(revision)) = (
                        gate.get("owner").and_then(serde_json::Value::as_str),
                        gate.get("resource").and_then(serde_json::Value::as_str),
                        gate.get("revision").and_then(serde_json::Value::as_str),
                    ) else {
                        if !agenda
                            .truncated_sources
                            .iter()
                            .any(|value| value == &source)
                        {
                            agenda.truncated_sources.push(source.clone());
                        }
                        continue;
                    };
                    agenda.push(alignment_human_gate_entry(
                        instance,
                        record_revision,
                        owner,
                        resource,
                        revision,
                    ));
                }
            }
        } else if name == "moira-participation" && status == "current" {
            let options = peer
                .get("permission_option_count")
                .and_then(serde_json::Value::as_u64);
            if let Some(options) = options {
                let revision = peer
                    .get("revision")
                    .map(serde_json::Value::to_string)
                    .unwrap_or_else(|| "unknown".into());
                let identity = peer
                    .get("participation")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                agenda.push(AgendaEntry {
                    source_peer: "moira-participation".into(),
                    source_instance: None,
                    source_revision: revision,
                    resource_identity: identity.into(),
                    kind: AgendaKind::GovernedPermission,
                    urgency: AgendaUrgency::Blocking,
                    actionability: AgendaActionability::Actionable,
                    blocker: None,
                    title: format!("Governed permission · {options} exact option(s)"),
                    why_operator: "the governed participation is awaiting an attributable choice"
                        .into(),
                    authority: "only the Moira/Aegis owner contract can settle this request".into(),
                    requested_action: "inspect the exact challenge and choose, refuse, or defer"
                        .into(),
                    consequence_of_waiting:
                        "the participation remains paused at the permission gate".into(),
                    next_owner_action: "open Moira; refresh before acting if the revision changed"
                        .into(),
                    route: "navigate.moira".into(),
                });
            }
        } else if name == "moira-participation" && status != "current" {
            agenda.unavailable_sources.push(name.into());
        } else if name == "moira-deliberation" && status == "current" {
            let Some(gate) = peer
                .get("deliberation")
                .and_then(|value| value.get("human_gate"))
                .filter(|value| !value.is_null())
            else {
                continue;
            };
            if gate.get("status").and_then(serde_json::Value::as_str) != Some("pending") {
                continue;
            }
            let Some(request) = gate.get("request") else {
                continue;
            };
            let identity = request
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let reviewer = request
                .get("reviewer")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let revision = peer
                .get("registry_revision")
                .map(serde_json::Value::to_string)
                .unwrap_or_else(|| "unknown".into());
            let packet_is_current = peer
                .get("decision_packet_reader")
                .and_then(|reader| reader.get("status"))
                .and_then(serde_json::Value::as_str)
                == Some("current");
            let blocker = if packet_is_current {
                "Pantheon verified the exact decision packet, but no Aegis-authorized response control is composed"
            } else {
                "Moira publishes an exact decision-packet reference, but Pantheon has no current selected packet reader or Aegis-authorized response control"
            };
            let requested_action = if packet_is_current {
                "inspect the verified decision packet; response remains unavailable until the command-scoped Aegis control is composed"
            } else {
                "inspect the exact packet reference; configure and qualify its selected reader before any human disposition is offered"
            };
            let next_owner_action = if packet_is_current {
                "compose and qualify the command-scoped response control without weakening the human gate"
            } else {
                "select and qualify the decision-packet reader, then compose the command-scoped response control"
            };
            agenda.push(AgendaEntry {
                source_peer: "moira-deliberation".into(),
                source_instance: None,
                source_revision: revision,
                resource_identity: identity.into(),
                kind: AgendaKind::HumanGate,
                urgency: AgendaUrgency::Blocking,
                actionability: AgendaActionability::InspectOnly,
                blocker: Some(blocker.into()),
                title: "Named-human deliberation gate".into(),
                why_operator: format!("Moira retained an unresolved gate for reviewer {reviewer}"),
                authority: "agenda presence grants none; only the command-scoped Moira/Aegis response can settle this exact gate".into(),
                requested_action: requested_action.into(),
                consequence_of_waiting: "the exact decision remains human-gated; no agent or timeout may infer a disposition".into(),
                next_owner_action: next_owner_action.into(),
                route: "navigate.moira.deliberation".into(),
            });
        } else if name == "moira-deliberation" && status != "current" {
            agenda.unavailable_sources.push(name.into());
        }
    }
    agenda.sort();
    agenda
}

fn record_observation(
    peers: &mut Vec<serde_json::Value>,
    management: &mut Vec<PeerRuntime>,
    complete: &mut bool,
    observation: serde_json::Value,
    current: bool,
    hosted: bool,
) {
    management.push(PeerRuntime::from_observation(&observation, hosted));
    peers.push(observation);
    *complete &= current;
}

fn task_conversation_routing_projection(profile: &TaskConversationProfile) -> serde_json::Value {
    if !profile.agent_routes.is_empty() {
        return serde_json::json!({
            "mode": "exact_participant_sessions",
            "routes": profile.agent_routes.iter().map(|route| serde_json::json!({
                "addressee": route.addressee,
                "session": route.session,
            })).collect::<Vec<_>>(),
        });
    }
    if let Some(addressee) = &profile.agent_addressee {
        return serde_json::json!({
            "mode": "legacy_selected_workset",
            "addressee": addressee,
            "session": "selected_in_tui",
        });
    }
    serde_json::json!({"mode": "not_configured"})
}

fn observe_task_conversation(socket: PathBuf) -> (serde_json::Value, bool) {
    use agora_chronos_adapter::task_conversation_local::{
        LocalRequestV1, TASK_CONVERSATION_CAPABILITY_V1, TaskConversationLocalClientV1,
    };

    let client = TaskConversationLocalClientV1::new(socket);
    match client.request(&LocalRequestV1::Status) {
        Ok(reply)
            if reply.version == 1
                && reply.capability == TASK_CONVERSATION_CAPABILITY_V1
                && reply.complete
                && reply.status == "current" =>
        {
            (
                serde_json::json!({
                    "peer": "task-conversation",
                    "capability": TASK_CONVERSATION_CAPABILITY_V1,
                    "status": "current",
                    "revision": reply.revision,
                    "links": reply.link_count,
                    "boundary": "typed relationship truth only; room activity is not task progress",
                }),
                true,
            )
        }
        Ok(reply) => (
            serde_json::json!({
                "peer": "task-conversation",
                "capability": TASK_CONVERSATION_CAPABILITY_V1,
                "status": "refused",
                "owner_status": reply.status,
                "refusal": reply.refusal,
                "next_action": "inspect the relationship owner and recover any fenced or uncertain commit before binding",
            }),
            false,
        ),
        Err(error) => (
            peer_failure(
                "task-conversation",
                TASK_CONVERSATION_CAPABILITY_V1,
                "unavailable",
                format!("{error:?}"),
            ),
            false,
        ),
    }
}

fn observe_tellus(endpoint: SocketAddr, token_path: Option<&PathBuf>) -> (serde_json::Value, bool) {
    use std::time::Duration;
    use tellus::TellusPort;

    let capability = tellus::CAPABILITIES.headless[0];
    let token = match token_path.map(read_token).transpose() {
        Ok(token) => token,
        Err(error) => {
            return (
                peer_failure("tellus", capability, "configuration_refused", error),
                false,
            );
        }
    };
    let mut port =
        match tellus_local_client::LocalPort::new(endpoint, token, Duration::from_secs(5)) {
            Ok(port) => port,
            Err(error) => {
                return (
                    peer_failure(
                        "tellus",
                        capability,
                        "configuration_refused",
                        format!("{error:?}"),
                    ),
                    false,
                );
            }
        };
    match port.observe(tellus::OBSERVATION_LIMIT) {
        Ok(value) => {
            let current = value.complete && value.status == "current";
            (
                serde_json::json!({
                    "peer": "tellus", "capability": capability,
                    "status": value.status, "complete": value.complete,
                    "entries": value.entries.iter().map(|entry| serde_json::json!({
                        "load_id": entry.load_id, "state": entry.state,
                        "freshness": entry.freshness, "sha256": entry.sha256
                    })).collect::<Vec<_>>(),
                    "truncated": value.truncated,
                }),
                current,
            )
        }
        Err(error) => (
            peer_failure("tellus", capability, "unavailable", format!("{error:?}")),
            false,
        ),
    }
}

fn read_token(path: &PathBuf) -> Result<String, String> {
    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > 8 * 1024 {
        return Err("Tellus token exceeds 8 KiB".into());
    }
    let token = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let token = token.trim_end().to_owned();
    if token.is_empty() {
        return Err("Tellus token is empty".into());
    }
    Ok(token)
}

fn observe_aegis(socket: PathBuf) -> (serde_json::Value, bool) {
    use std::time::Duration;

    let capability = aegis_tui::CAPABILITIES.headless[0];
    let (session, port) = match aegis_local_client::connect(
        socket,
        aegis_wire::LimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return (
                peer_failure("aegis", capability, "connect_refused", format!("{error:?}")),
                false,
            );
        }
    };
    let mut client = match aegis_client::ReadOnlyClient::new(session, port, aegis_tui::PAGE_LIMIT) {
        Ok(client) => client,
        Err(_) => {
            return (
                peer_failure("aegis", capability, "configuration_refused", "page limit"),
                false,
            );
        }
    };
    match aegis_tui::observe(&mut client) {
        Ok(value) => (
            serde_json::json!({
                "peer": "aegis", "capability": capability, "status": "current",
                "epoch": value.status.epoch, "active_grants": value.status.active_grants,
                "retained_decisions": value.status.retained_decisions,
                "page_grants": value.grants.len(), "more_grants": value.more_grants,
                "page_decisions": value.decisions.len(), "more_decisions": value.more_decisions,
            }),
            true,
        ),
        Err(error) => (
            peer_failure("aegis", capability, "unavailable", error),
            false,
        ),
    }
}

fn observe_hermes(
    socket: PathBuf,
    configured_stream: Option<hermes_protocol::StreamIdV1>,
    configured_consumer: Option<hermes_protocol::ConsumerIdV1>,
    consumer_access: bool,
) -> (serde_json::Value, bool) {
    let capability = hermes_tui::CAPABILITIES.headless[0];
    let port = match hermes_local_client::LocalPort::connect(
        socket,
        hermes_local_client::ClientLimits::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return (
                peer_failure(
                    "hermes",
                    capability,
                    "connect_refused",
                    format!("{error:?}"),
                ),
                false,
            );
        }
    };
    if consumer_access {
        let stream = configured_stream.expect("validated consumer Hermes stream");
        let consumer = configured_consumer.expect("validated consumer Hermes identity");
        let mut client = hermes_client::ConsumerClient::new(
            hermes_local_client::CONNECTION_SESSION,
            port,
            hermes_tui::DISCLOSURE_BATCH_LIMIT,
        )
        .expect("positive Hermes consumer read bound");
        return match hermes_tui::observe_consumer(
            &mut client,
            stream,
            consumer,
            hermes_tui::DISCLOSURE_BATCH_LIMIT,
        ) {
            Ok(value) => (
                serde_json::json!({
                    "peer": "hermes", "capability": "ledger.consumer.observe.v1",
                    "status": "current", "access": "consumer",
                    "service_aggregates": "not_requested",
                    "configured_stream": {
                        "status": "current", "retained_from": value.retained_from.0,
                        "published_edge": value.published_edge.map(|edge| edge.0),
                    },
                    "configured_cursor": value.cursor.map(|cursor| serde_json::json!({
                        "status": "current", "next_sequence": cursor.next_sequence.0,
                    })).unwrap_or_else(|| serde_json::json!({"status": "absent"})),
                    "bounded_read": {
                        "start": value.read_start.0,
                        "events": value.disclosed_events,
                        "through": value.disclosed_through.map(|sequence| sequence.0),
                        "gap": format!("{:?}", value.gap),
                    },
                }),
                true,
            ),
            Err(error) => (
                peer_failure("hermes", "ledger.consumer.observe.v1", "unavailable", error),
                false,
            ),
        };
    }
    let mut client =
        hermes_client::ReadOnlyClient::new(hermes_local_client::CONNECTION_SESSION, port);
    match hermes_tui::observe_selected(&mut client, configured_stream, configured_consumer) {
        Ok(value) => {
            let stream = value.configured_stream.map(|selection| match selection {
                hermes_tui::ConfiguredStreamObservation::Missing => {
                    serde_json::json!({"status": "absent"})
                }
                hermes_tui::ConfiguredStreamObservation::Current(stream) => serde_json::json!({
                    "status": "current", "retained_from": stream.retained_from.0,
                    "published_edge": stream.published_edge.map(|edge| edge.0),
                    "retained_events": stream.retained_events,
                    "consumer_count": stream.consumer_count,
                    "stalled_consumers": stream.stalled_consumers,
                    "max_consumer_backlog": stream.max_consumer_backlog,
                }),
            });
            let cursor = value.configured_cursor.map(|selection| match selection {
                hermes_tui::ConfiguredCursorObservation::Missing => {
                    serde_json::json!({"status": "absent"})
                }
                hermes_tui::ConfiguredCursorObservation::Current(cursor) => {
                    let stream = match value.configured_stream {
                        Some(hermes_tui::ConfiguredStreamObservation::Current(stream)) => stream,
                        _ => unreachable!("current cursor requires current stream"),
                    };
                    serde_json::json!({
                        "status": "current",
                        "next_sequence": cursor.next_sequence.0,
                        "backlog": hermes_tui::cursor_backlog(stream, cursor),
                        "behind_retention": cursor.next_sequence < stream.retained_from,
                    })
                }
            });
            (
                serde_json::json!({
                "peer": "hermes", "capability": capability, "status": "current",
                "storage_revision": value.status.storage_revision,
                "fenced": value.status.fenced, "streams": value.usage.streams,
                "events": value.usage.events, "cursors": value.usage.cursors,
                "commits": value.signals.commits, "conflicts": value.signals.conflicts,
                "saturated": value.signals.saturated,
                "configured_stream": stream,
                "configured_cursor": cursor,
                }),
                true,
            )
        }
        Err(error) => (
            peer_failure("hermes", capability, "unavailable", error),
            false,
        ),
    }
}

fn observe_daimon(
    socket: Option<PathBuf>,
    model_socket: Option<PathBuf>,
    agent_socket: Option<PathBuf>,
    agent_session: Option<daimon_protocol::AgentSessionIdV1>,
    agent_open: Option<&DaimonAgentOpenProfile>,
    execution_policy: Option<&DaimonExecutionPolicyProfile>,
) -> (serde_json::Value, bool) {
    use daimon_client::ReadOnlyClient;
    use daimon_client::unix::ClientErrorV1;
    use daimon_protocol::{OutcomeV1, ReplyV1};
    use std::time::Duration;

    let capability = daimon_tui::CAPABILITIES.headless[0];
    let Some(socket) = socket else {
        let (model, model_current) = match model_socket {
            Some(socket) => observe_daimon_model(socket),
            None => (serde_json::Value::Null, true),
        };
        let (agent, current) = match (agent_socket, agent_session, agent_open) {
            (Some(socket), Some(session), _) => observe_daimon_agent(socket, session),
            (Some(socket), None, Some(open)) => observe_daimon_open_endpoint(socket, open),
            (None, None, None) => (serde_json::Value::Null, true),
            _ => (
                serde_json::json!({"status": "configuration_refused"}),
                false,
            ),
        };
        return (
            serde_json::json!({
                "peer": "daimon",
                "capability": if model.is_null() {
                    if agent_open.is_some() { "agents.session.open.v1" } else { "agents.session.inspect.v1" }
                } else { "models.endpoint.attach.v1" },
                "capabilities": match (model.is_null(), agent.is_null()) {
                    (false, false) => vec!["models.endpoint.attach.v1", if agent_open.is_some() { "agents.session.open.v1" } else { "agents.session.inspect.v1" }],
                    (false, true) => vec!["models.endpoint.attach.v1"],
                    (true, false) => vec![if agent_open.is_some() { "agents.session.open.v1" } else { "agents.session.inspect.v1" }],
                    (true, true) => Vec::<&str>::new(),
                },
                "status": if current && model_current { "current" } else { "partial" },
                "model_endpoint": model,
                "configured_agent_session": agent,
                "configured_execution_profile": daimon_execution_projection(execution_policy),
            }),
            current && model_current,
        );
    };
    let (session, port) = match daimon_client::unix::connect(
        socket,
        daimon_wire::LimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return (
                peer_failure(
                    "daimon",
                    capability,
                    "connect_refused",
                    format!("{error:?}"),
                ),
                false,
            );
        }
    };
    let mut client = ReadOnlyClient::new(session, port);
    match client.read_presence(64) {
        Ok(ReplyV1::Accepted(OutcomeV1::Presence { entries, truncated })) => {
            let current = entries.iter().filter(|entry| !entry.stale).count();
            let (agent, agent_current) = match (agent_socket, agent_session, agent_open) {
                (Some(socket), Some(session), _) => observe_daimon_agent(socket, session),
                (Some(socket), None, Some(open)) => observe_daimon_open_endpoint(socket, open),
                (None, None, None) => (serde_json::Value::Null, true),
                _ => (
                    serde_json::json!({"status": "configuration_refused"}),
                    false,
                ),
            };
            let (model, model_current) = match model_socket {
                Some(socket) => observe_daimon_model(socket),
                None => (serde_json::Value::Null, true),
            };
            (
                serde_json::json!({
                    "peer": "daimon",
                    "capability": capability,
                    "capabilities": if agent.is_null() {
                        vec![capability]
                    } else if agent_open.is_some() {
                        vec![capability, "agents.session.open.v1"]
                    } else {
                        vec![capability, "agents.session.inspect.v1"]
                    },
                    "status": if agent_current && model_current { "current" } else { "partial" },
                    "agents": entries.len(),
                    "current": current,
                    "stale": entries.len().saturating_sub(current),
                    "truncated": truncated,
                    "model_endpoint": model,
                    "configured_agent_session": agent,
                    "configured_execution_profile": daimon_execution_projection(execution_policy),
                }),
                agent_current && model_current,
            )
        }
        Ok(ReplyV1::Refused(reason)) => (
            peer_failure("daimon", capability, "refused", format!("{reason:?}")),
            false,
        ),
        Ok(ReplyV1::Accepted(_)) => (
            peer_failure("daimon", capability, "invalid_reply", "unexpected outcome"),
            false,
        ),
        Err(error) => {
            let status = match error {
                ClientErrorV1::NotSent(_) => "not_sent",
                ClientErrorV1::OutcomeUnknown(_) => "outcome_unknown",
                ClientErrorV1::Wire(_) | ClientErrorV1::SessionMismatch => "invalid_reply",
            };
            (
                peer_failure("daimon", capability, status, "client access failure"),
                false,
            )
        }
    }
}

fn observe_daimon_open_endpoint(
    socket: PathBuf,
    open: &DaimonAgentOpenProfile,
) -> (serde_json::Value, bool) {
    use std::time::Duration;

    match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => {
            drop(connection);
            (
                serde_json::json!({
                    "status": "current",
                    "capability": "agents.session.open.v1",
                    "workspace": open.workspace,
                    "session_budget": {
                        "max_elapsed_millis": open.budget.max_elapsed_millis,
                        "max_turns": open.budget.max_turns,
                        "max_events": open.budget.max_events,
                        "max_event_bytes": open.budget.max_event_bytes,
                        "max_prompt_bytes": open.budget.max_prompt_bytes,
                        "max_permission_requests": open.budget.max_permission_requests,
                    },
                    "selected_session": serde_json::Value::Null,
                    "boundary": "endpoint attachment and configured open policy only; no agent session was invented or opened",
                }),
                true,
            )
        }
        Err(error) => (
            serde_json::json!({
                "status": "connect_refused",
                "capability": "agents.session.open.v1",
                "detail": format!("{error:?}"),
            }),
            false,
        ),
    }
}

fn observe_daimon_model(socket: PathBuf) -> (serde_json::Value, bool) {
    use daimon_client::unix::{ClientErrorV1, connect_model};
    use std::time::Duration;

    match connect_model(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::ModelLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok((_session, port)) => {
            drop(port);
            (
                serde_json::json!({
                    "status": "attached",
                    "capability": "models.endpoint.attach.v1",
                    "boundary": "authenticated model-protocol attachment only; no execution, provider health, qualification, or authority is implied",
                }),
                true,
            )
        }
        Err(error) => {
            let status = match error {
                ClientErrorV1::NotSent(_) => "not_sent",
                ClientErrorV1::OutcomeUnknown(_) => "outcome_unknown",
                ClientErrorV1::Wire(_) | ClientErrorV1::SessionMismatch => "invalid_reply",
            };
            (
                serde_json::json!({
                    "status": status,
                    "capability": "models.endpoint.attach.v1",
                    "next_action": "inspect the selected Daimon model owner and retry attachment; do not invoke the provider",
                }),
                false,
            )
        }
    }
}

fn daimon_execution_projection(policy: Option<&DaimonExecutionPolicyProfile>) -> serde_json::Value {
    let Some(policy) = policy else {
        return serde_json::Value::Null;
    };
    serde_json::json!({
        "status": "selected",
        "principal": policy.principal,
        "principal_type": "not_published",
        "harness_profile": policy.profile,
        "task_attempt": {
            "task_key": policy.task_attempt.task_key,
            "task_incarnation": policy.task_attempt.task_incarnation,
            "attempt_number": policy.task_attempt.attempt_number,
        },
        "context_count": policy.context.len(),
        "context_references": policy.context,
        "tool_count": policy.tools.len(),
        "tool_capabilities": policy.tools,
        "budget": {
            "max_elapsed_millis": policy.budget.max_elapsed_millis,
            "max_input_tokens": policy.budget.max_input_tokens,
            "max_output_tokens": policy.budget.max_output_tokens,
            "max_tool_calls": policy.budget.max_tool_calls,
            "max_context_bytes": policy.budget.max_context_bytes,
            "max_output_bytes": policy.budget.max_output_bytes,
        },
        "role_work_mode": "not_published",
        "delegation": "not_published",
        "accountable_owner": "not_published",
        "authority_reference": "not_published",
        "policy_epoch": "not_published",
        "expiry": "not_published",
        "boundary": "configuration selection is not role activation, delegation, authority, or owner receipt",
    })
}

pub(crate) fn observe_daimon_agent(
    socket: PathBuf,
    agent_session: daimon_protocol::AgentSessionIdV1,
) -> (serde_json::Value, bool) {
    use daimon_client::AgentReadOnlyClient;
    use std::time::Duration;

    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return (
                serde_json::json!({
                    "status": "connect_refused",
                    "error": format!("{error:?}"),
                }),
                false,
            );
        }
    };
    let mut client = AgentReadOnlyClient::new(service_session, port);
    match daimon_tui::observe_agent(&mut client, agent_session) {
        Ok(observation) => {
            let view = observation.view;
            let capabilities = view.capabilities;
            let last_context = daimon_context_projection(view.last_context.as_ref());
            let events = observation
                .events
                .entries
                .iter()
                .map(daimon_agent_event_projection)
                .collect::<Vec<_>>();
            (
                serde_json::json!({
                    "status": "current",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                    "profile_document_digest": hex_bytes(&view.profile_document_digest.into_bytes()),
                    "last_context": last_context,
                    "capabilities": {
                        "load_session": capabilities.load_session,
                        "resume_session": capabilities.resume_session,
                        "permissions": capabilities.permissions,
                        "filesystem_read": capabilities.filesystem_read,
                        "filesystem_write": capabilities.filesystem_write,
                        "terminal": capabilities.terminal,
                        "session_modes": capabilities.session_modes,
                        "session_models": capabilities.session_models,
                    },
                    "events": events,
                    "truncated": observation.events.next_after.is_some(),
                }),
                true,
            )
        }
        Err(error) => {
            let status = if error.contains("refused") {
                "refused"
            } else {
                "unavailable"
            };
            (serde_json::json!({"status": status, "error": error}), false)
        }
    }
}

fn daimon_context_projection(
    value: Option<&daimon_protocol::agent::AgentContextReceiptV1>,
) -> Option<serde_json::Value> {
    value.map(|receipt| {
        serde_json::json!({
            "execution": hex_bytes(&receipt.execution.into_bytes()),
            "manifest_digest": format!(
                "sha256:{}",
                hex_bytes(&receipt.manifest_digest.into_bytes())
            ),
            "entries": receipt.entries.iter().map(|entry| serde_json::json!({
                "reference": hex_bytes(&entry.reference.into_bytes()),
                "kind": match entry.kind {
                    daimon_protocol::ContextKindV1::AgoraMessage => "agora_message",
                    daimon_protocol::ContextKindV1::ChronosWork => "chronos_work",
                    daimon_protocol::ContextKindV1::VellumDocument => "vellum_document",
                    daimon_protocol::ContextKindV1::DendriteGrounding => "dendrite_grounding",
                },
                "source_digest": format!(
                    "sha256:{}",
                    hex_bytes(&entry.source_digest.into_bytes())
                ),
                "content_bytes": entry.content_bytes,
                "grounding": entry.grounding.map(|grounding| serde_json::json!({
                    "depth": format!("{:?}", grounding.depth).to_ascii_lowercase(),
                    "grounding_digest": format!(
                        "sha256:{}",
                        hex_bytes(&grounding.grounding_digest.into_bytes())
                    ),
                    "surface_digest": format!(
                        "sha256:{}",
                        hex_bytes(&grounding.surface_digest.into_bytes())
                    ),
                })),
            })).collect::<Vec<_>>(),
            "content_disclosure": "not_published",
        })
    })
}

fn daimon_agent_event_kind(event: &daimon_protocol::agent::AgentEventV1) -> &'static str {
    use daimon_protocol::agent::AgentEventV1;
    match event {
        AgentEventV1::AssistantMessage { .. } => "assistant_message",
        AgentEventV1::Thought { .. } => "thought",
        AgentEventV1::Plan { .. } => "plan",
        AgentEventV1::ToolCall { .. } => "tool_call",
        AgentEventV1::Terminal { .. } => "terminal",
        AgentEventV1::Permission(_) => "permission",
        AgentEventV1::FileChange { .. } => "file_change",
        AgentEventV1::FileRead { .. } => "file_read",
        AgentEventV1::Usage { .. } => "usage",
        AgentEventV1::TurnFinished { .. } => "turn_finished",
    }
}

fn daimon_agent_event_projection(
    record: &daimon_protocol::agent::AgentEventRecordV1,
) -> serde_json::Value {
    use daimon_protocol::agent::AgentEventV1;
    let evidence = match &record.event {
        AgentEventV1::AssistantMessage { bytes }
        | AgentEventV1::Thought { bytes }
        | AgentEventV1::Plan { bytes } => content_shape(bytes),
        AgentEventV1::ToolCall { title, detail } => serde_json::json!({
            "title": content_shape(title),
            "detail": content_shape(detail),
        }),
        AgentEventV1::Terminal { stream, bytes } => serde_json::json!({
            "stream": content_shape(stream),
            "output": content_shape(bytes),
        }),
        AgentEventV1::Permission(request) => serde_json::json!({
            "request": hex_bytes(&request.request.into_bytes()),
            "turn": hex_bytes(&request.turn.into_bytes()),
            "capability": content_shape(&request.capability),
            "options": request.options.iter().map(|option| serde_json::json!({
                "id": hex_bytes(&option.id.into_bytes()),
                "kind": format!("{:?}", option.kind).to_ascii_lowercase(),
                "label": content_shape(&option.label),
            })).collect::<Vec<_>>(),
        }),
        AgentEventV1::FileChange {
            path,
            content_digest,
        } => serde_json::json!({
            "path": content_shape(path),
            "content_digest": format!("sha256:{}", hex_bytes(&content_digest.into_bytes())),
        }),
        AgentEventV1::FileRead {
            path,
            content_digest,
            content_bytes,
        } => serde_json::json!({
            "path": content_shape(path),
            "content_digest": format!("sha256:{}", hex_bytes(&content_digest.into_bytes())),
            "content_bytes": content_bytes,
        }),
        AgentEventV1::Usage {
            input_tokens,
            output_tokens,
        } => serde_json::json!({
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
        }),
        AgentEventV1::TurnFinished {
            turn,
            disposition,
            stop_reason,
            output_digest,
        } => serde_json::json!({
            "turn": hex_bytes(&turn.into_bytes()),
            "disposition": format!("{disposition:?}").to_ascii_lowercase(),
            "stop_reason": format!("{stop_reason:?}").to_ascii_lowercase(),
            "output_digest": output_digest.map(|digest| format!("sha256:{}", hex_bytes(&digest.into_bytes()))),
        }),
    };
    serde_json::json!({
        "cursor": record.cursor,
        "kind": daimon_agent_event_kind(&record.event),
        "evidence": evidence,
    })
}

fn content_shape(bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "bytes": bytes.len(),
        "content": "not_disclosed",
    })
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn observe_moira(socket: PathBuf) -> (serde_json::Value, bool) {
    let capability = moira_tui::CAPABILITIES.headless[0];
    let port = match moira_client::unix::SagaUnixPort::connect(
        socket,
        moira_client::unix::SagaClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return (
                peer_failure("moira", capability, "connect_refused", format!("{error:?}")),
                false,
            );
        }
    };
    let mut client = SagaReadOnlyClient::new(port);
    match client.inspect(None, 64) {
        Ok(SagaReplyV1::Accepted(SagaOutcomeV1::Page(page))) => {
            let census = match client.census() {
                Ok(SagaReplyV1::Accepted(SagaOutcomeV1::Census(census))) => census,
                Ok(SagaReplyV1::Refused(reason)) => {
                    return (
                        peer_failure("moira", capability, "refused", format!("{reason:?}")),
                        false,
                    );
                }
                Ok(SagaReplyV1::Accepted(_)) => {
                    return (
                        peer_failure(
                            "moira",
                            capability,
                            "invalid_reply",
                            "unexpected census outcome",
                        ),
                        false,
                    );
                }
                Err(error) => return moira_client_failure(capability, error),
            };
            (
                serde_json::json!({
                "peer": "moira",
                "capability": capability,
                "status": "current",
                "sagas": page.entries.len(),
                "truncated": page.next_after.is_some(),
                "census": {
                    "total": census.total,
                    "runnable_effects": census.runnable_effects,
                    "awaiting_decision": census.awaiting_decision,
                    "fenced": census.fenced,
                    "terminal": census.terminal,
                    "retries_consumed": census.retries_consumed,
                    "saturated": census.saturated,
                },
                }),
                true,
            )
        }
        Ok(SagaReplyV1::Refused(reason)) => (
            peer_failure("moira", capability, "refused", format!("{reason:?}")),
            false,
        ),
        Ok(SagaReplyV1::Accepted(_)) => (
            peer_failure("moira", capability, "invalid_reply", "unexpected outcome"),
            false,
        ),
        Err(error) => moira_client_failure(capability, error),
    }
}

fn observe_moira_participation(
    socket: PathBuf,
    id: participation::ParticipationIdV1,
) -> (serde_json::Value, bool) {
    let capability = "agent.participation.inspect.v1";
    let port = match moira_client::unix::AgentParticipationUnixPort::connect(
        socket,
        moira_client::unix::ParticipationClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return (
                peer_failure(
                    "moira-participation",
                    capability,
                    "connect_refused",
                    format!("{error:?}"),
                ),
                false,
            );
        }
    };
    let mut client = AgentParticipationReadOnlyClient::new(port);
    match client.participation(id) {
        Ok(participation::ReplyV1::Accepted(outcome)) => match *outcome {
            participation::OutcomeV1::Participation(Some(view)) => (
                serde_json::json!({
                    "peer": "moira-participation",
                    "capability": capability,
                    "status": "current",
                    "accountable_owner": "moira",
                    "principal_type": "agent",
                    "participation": hex_bytes(&view.binding.participation.0),
                    "revision": view.revision,
                    "previously_observed_revision": null,
                    "revision_history": "not_published",
                    "phase": format!("{:?}", view.phase),
                    "session_revision": view.session_revision,
                    "event_cursor": view.last_event_cursor,
                    "binding": {
                        "workflow": hex_bytes(&view.binding.workflow.0),
                        "deliberation": hex_bytes(&view.binding.deliberation.0),
                        "member": hex_bytes(&view.binding.member.0),
                        "session": hex_bytes(&view.binding.session.0),
                        "profile": hex_bytes(&view.binding.profile.0),
                        "task_key": hex_bytes(&view.binding.attempt.task_key),
                        "incarnation": view.binding.attempt.incarnation,
                        "attempt": view.binding.attempt.attempt,
                        "persona": format!("{:?}", view.binding.persona),
                        "method": {
                            "machine_digest": hex_bytes(&view.binding.method.machine_digest),
                            "state_digest": hex_bytes(&view.binding.method.state_digest),
                        },
                        "grounding": {
                            "depth": format!("{:?}", view.binding.grounding.depth),
                            "authority_digest": hex_bytes(&view.binding.grounding.authority_digest),
                            "surface_digest": hex_bytes(&view.binding.grounding.surface_digest),
                        },
                    },
                    "profile_document_digest": view.profile_document_digest.map(|digest| hex_bytes(&digest)),
                    "active_turn": view.active_turn.is_some(),
                    "pending_operation": view.pending.map(participation_operation_kind),
                    "permission_option_count": view.permission.map(|value| value.option_count),
                    "capabilities": {
                        "load_session": view.capabilities.load_session,
                        "resume_session": view.capabilities.resume_session,
                        "permissions": view.capabilities.permissions,
                        "filesystem_read": view.capabilities.filesystem_read,
                        "filesystem_write": view.capabilities.filesystem_write,
                        "terminal": view.capabilities.terminal,
                        "session_modes": view.capabilities.session_modes,
                        "session_models": view.capabilities.session_models,
                    },
                    "missing_facts": [
                        "role_or_work_mode",
                        "scope",
                        "delegation",
                        "execution_budget",
                        "session_budget",
                        "authority_reference",
                        "policy_epoch",
                        "expiry",
                        "accountable_human"
                    ]
                }),
                true,
            ),
            participation::OutcomeV1::Participation(None) => (
                peer_failure(
                    "moira-participation",
                    capability,
                    "not_found",
                    "participation not found",
                ),
                false,
            ),
            _ => (
                peer_failure(
                    "moira-participation",
                    capability,
                    "invalid_reply",
                    "unexpected outcome",
                ),
                false,
            ),
        },
        Ok(participation::ReplyV1::Refused(reason)) => (
            peer_failure(
                "moira-participation",
                capability,
                "refused",
                format!("{reason:?}"),
            ),
            false,
        ),
        Err(error) => (
            peer_failure(
                "moira-participation",
                capability,
                "transport",
                format!("{error:?}"),
            ),
            false,
        ),
    }
}

fn observe_moira_deliberation(
    socket: PathBuf,
    id: deliberation_protocol::DeliberationIdV1,
) -> (
    serde_json::Value,
    bool,
    Option<deliberation_protocol::HumanGateMaterialRefV1>,
) {
    let capability = "deliberation.inspect.v1";
    match crate::moira_deliberation::observe(socket, id) {
        crate::moira_deliberation::Observation::Current {
            registry_revision,
            deliberation,
            review,
            pending_packet,
        } => (
            serde_json::json!({
            "peer": "moira-deliberation",
            "capability": capability,
            "status": "current",
            "accountable_owner": "moira",
            "registry_revision": registry_revision,
            "deliberation": deliberation,
            "review": review,
            "boundary": "read-only owner evidence and stakeholder standing; distinct correlation values do not establish independence, and Pantheon infers no ensemble fitness, frontier, authority, or human disposition",
            }),
            true,
            pending_packet,
        ),
        crate::moira_deliberation::Observation::NotFound { registry_revision } => (
            serde_json::json!({
                "peer": "moira-deliberation",
                "capability": capability,
                "status": "not_found",
                "registry_revision": registry_revision,
            }),
            false,
            None,
        ),
        crate::moira_deliberation::Observation::Failure { status, detail } => (
            peer_failure("moira-deliberation", capability, status, detail),
            false,
            None,
        ),
    }
}

fn observe_decision_packet(
    socket: &std::path::Path,
    credential: Option<&std::path::Path>,
    profile: &MoiraDecisionPacketReaderProfile,
    reference: deliberation_protocol::HumanGateMaterialRefV1,
) -> (serde_json::Value, bool) {
    let failure = |stage: &'static str, detail: String| {
        (
            serde_json::json!({
                "status": "refused",
                "stage": stage,
                "detail": detail,
                "next_action": "repair the selected packet route or Vellum evidence, then refresh before deciding",
                "boundary": "packet-read failure grants no response authority and no disposition is inferred",
            }),
            false,
        )
    };
    let owner_route = match decode_fixed_hex::<16>(&profile.owner_route) {
        Some(value) => value,
        None => return failure("profile", "invalid owner route".into()),
    };
    let artifact_route = match decode_fixed_hex::<16>(&profile.artifact_route) {
        Some(value) => value,
        None => return failure("profile", "invalid artifact route".into()),
    };
    let Some(workspace) = vellum_protocol::WorkspaceIdV1::new(profile.vellum_workspace) else {
        return failure("profile", "invalid Vellum workspace identity".into());
    };
    let Some(document) =
        vellum_protocol::WorkspaceDocumentIdV1::new(workspace, profile.vellum_document)
    else {
        return failure("profile", "invalid Vellum document identity".into());
    };
    let credential = credential.map(PathBuf::from);
    let connector = match vellum_connector(socket.to_path_buf(), credential.as_ref()) {
        Ok(value) => value,
        Err(error) => return failure("connect", error),
    };
    let port = match RemoteTransport::new(connector, TransportLimits::reference()) {
        Ok(value) => value,
        Err(error) => return failure("transport", format!("{error:?}")),
    };
    let target = ExternalDocumentRefV1 {
        workspace: owner_route,
        document: artifact_route,
    };
    let mut routes = BTreeMap::new();
    routes.insert(
        target,
        DocumentRouteV1 {
            workspace,
            document,
        },
    );
    let mut reader = match DecisionPacketAdapter::connect(port, routes, profile.max_packet_bytes) {
        Ok(value) => value,
        Err(error) => return failure("handshake", format!("{error:?}")),
    };
    match reader.read(reference) {
        Ok(packet) => (
            serde_json::json!({
                "status": "current",
                "owner_route": hex_bytes(&packet.reference.owner_route),
                "artifact_route": hex_bytes(&packet.reference.artifact_route),
                "revision": packet.reference.expected_revision,
                "sha256": hex_bytes(&packet.reference.expected_sha256),
                "text": packet.text,
                "boundary": "exact Vellum bytes verified against Moira's revision and digest; response authority remains unavailable",
            }),
            true,
        ),
        Err(error) => failure("read", format!("{error:?}")),
    }
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 || !value.is_ascii() {
        return None;
    }
    let mut decoded = [0; N];
    for (index, byte) in decoded.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(decoded)
}

fn participation_operation_kind(operation: participation::OperationV1) -> &'static str {
    match operation {
        participation::OperationV1::Open { .. } => "open",
        participation::OperationV1::Prompt { .. } => "prompt",
        participation::OperationV1::Advance { .. } => "advance",
        participation::OperationV1::DecidePermission { .. } => "decide_permission",
        participation::OperationV1::Cancel { .. } => "cancel",
        participation::OperationV1::Close { .. } => "close",
        participation::OperationV1::Reattach { .. } => "reattach",
    }
}

fn moira_client_failure(
    capability: &str,
    error: moira_client::unix::SagaClientErrorV1,
) -> (serde_json::Value, bool) {
    let status = match error {
        moira_client::unix::SagaClientErrorV1::NotSent(_) => "not_sent",
        moira_client::unix::SagaClientErrorV1::OutcomeUnknown(_) => "outcome_unknown",
        moira_client::unix::SagaClientErrorV1::RequestLimit => "request_limit",
        _ => "invalid_reply",
    };
    (
        peer_failure("moira", capability, status, "client access failure"),
        false,
    )
}

fn observe_agora(profile: AgoraProfile) -> (serde_json::Value, bool) {
    use agora_client::v2::ClientErrorV2;
    use agora_protocol::v2::{CommandV2, OutcomeV2, ReplyV2};
    let capability = agora_tui::CAPABILITIES.headless[0];

    let connection = match profile.transport {
        AgoraTransportProfile::Conversation => AgoraLocalServicePort::connect(
            &profile.socket,
            &profile.participant,
            LocalClientLimits::default(),
        ),
        AgoraTransportProfile::Composition => AgoraLocalServicePort::connect_composition(
            &profile.socket,
            &profile.participant,
            LocalClientLimits::default(),
        ),
    };
    let (client_id, port) = match connection {
        Ok(connection) => connection,
        Err(error) => {
            return (
                peer_failure("agora", capability, "connect_refused", format!("{error:?}")),
                false,
            );
        }
    };
    let mut client = AgoraClientV2::new(client_id, port);
    match client.execute(CommandV2::Rooms {
        cursor: None,
        limit: 32,
    }) {
        Ok(ReplyV2::Accepted(OutcomeV2::Rooms(page))) => (
            serde_json::json!({
                "peer": "agora",
                "capability": capability,
                "status": "current",
                "revision": page.revision,
                "rooms": page.rooms.len(),
                "truncated": page.next.is_some(),
            }),
            true,
        ),
        Ok(ReplyV2::Refused(reason)) => (
            peer_failure("agora", capability, "refused", format!("{reason:?}")),
            false,
        ),
        Ok(ReplyV2::Accepted(_)) => (
            peer_failure("agora", capability, "invalid_reply", "unexpected outcome"),
            false,
        ),
        Err(error) => {
            let status = match error {
                ClientErrorV2::NotSent(_) => "not_sent",
                ClientErrorV2::OutcomeUnknown(_) => "outcome_unknown",
                ClientErrorV2::UnresolvedOutcome => "unresolved_outcome",
                ClientErrorV2::RequestIdExhausted => "request_id_exhausted",
            };
            (
                peer_failure("agora", capability, status, "client access failure"),
                false,
            )
        }
    }
}

fn observe_chronos(workstream: Workstream) -> (serde_json::Value, bool) {
    use chronos_client::workflow::{Observer, ObserverLimits, RefreshError};
    use chronos_protocol::workflow_v9::{ExecutionPhase, Status};
    let capability = chronos_tui::CAPABILITIES.headless[0];

    let mut observer = Observer::new(ObserverLimits {
        page_size: 32,
        max_pages: 8,
        max_entries: 256,
        max_id_bytes: 256,
    })
    .expect("static Chronos observer limits are valid");
    let mut client = chronos_local::LocalClient::new(workstream.directory);
    match observer.refresh(&mut client) {
        Ok(()) => {
            let view = observer
                .view()
                .expect("successful refresh publishes a view");
            let agenda_items = view
                .entries
                .iter()
                .filter(|entry| {
                    entry.snapshot.status == Status::Blocked
                        || (entry.snapshot.status == Status::InProgress
                            && entry.snapshot.execution_phase == ExecutionPhase::Uncertain)
                })
                .map(|entry| {
                    serde_json::json!({
                        "task": entry.reference.task,
                        "incarnation": entry.reference.incarnation,
                        "status": entry.snapshot.status,
                        "execution_phase": entry.snapshot.execution_phase,
                        "priority": entry.snapshot.priority,
                        "dependency_count": entry.snapshot.dependency_count,
                        "blocker": entry.snapshot.blocker,
                    })
                })
                .collect::<Vec<_>>();
            let tasks = view
                .entries
                .iter()
                .map(|entry| {
                    serde_json::json!({
                        "task": entry.reference.task,
                        "incarnation": entry.reference.incarnation,
                        "status": entry.snapshot.status,
                        "execution_phase": entry.snapshot.execution_phase,
                        "attempt_number": entry.snapshot.attempt_number,
                        "priority": entry.snapshot.priority,
                        "dependency_count": entry.snapshot.dependency_count,
                        "required_capability_count": entry.snapshot.required_capability_count,
                        "blocker": entry.snapshot.blocker,
                    })
                })
                .collect::<Vec<_>>();
            (
                serde_json::json!({
                    "peer": "chronos",
                    "instance": workstream.name,
                    "capability": capability,
                    "status": "current",
                    "revision": format!("{:?}", view.revision),
                    "task_count": view.entries.len(),
                    "tasks": tasks,
                    "agenda_items": agenda_items,
                    "truncated": false,
                }),
                true,
            )
        }
        Err(error) => {
            let status = match error {
                RefreshError::Transport(chronos_local::CallError::NotSent) => "not_sent",
                RefreshError::Transport(chronos_local::CallError::OutcomeUnknown) => {
                    "outcome_unknown"
                }
                RefreshError::Transport(chronos_local::CallError::InvalidResponse)
                | RefreshError::InvalidResponse => "invalid_reply",
                RefreshError::Refused => "refused",
                RefreshError::ResyncRequired => "resync_required",
                RefreshError::Limit => "limit",
                RefreshError::Capacity => "capacity",
                RefreshError::InvalidLimits => "invalid_limits",
            };
            (
                serde_json::json!({
                    "peer": "chronos",
                    "instance": workstream.name,
                    "capability": capability,
                    "status": status,
                }),
                false,
            )
        }
    }
}

fn observe_chronos_alignment(binding: AlignmentBinding) -> (serde_json::Value, bool) {
    let capability = "product-alignment.observe.v1";
    match crate::chronos_alignment::observe(binding.directory, binding.scope.clone()) {
        crate::chronos_alignment::Observation::Current(observation) => (
            serde_json::json!({
                "peer": "chronos-alignment",
                "instance": binding.workstream,
                "capability": capability,
                "status": "current",
                "scope": binding.scope,
                "observation": observation,
                "boundary": "authorized content-free observation only; no apply, accept, claim, reprioritize, or sign-off authority",
            }),
            true,
        ),
        crate::chronos_alignment::Observation::Failure { status, detail } => (
            peer_failure(
                "chronos-alignment",
                capability,
                status,
                format!("{detail} for {}", binding.workstream),
            ),
            false,
        ),
    }
}

enum VellumConnector {
    Plain(UnixConnector),
    Authenticated(AuthenticatedUnixConnector),
}

impl FrameExchange for VellumConnector {
    fn exchange(
        &mut self,
        request: &[u8],
        max_reply_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::Plain(connector) => connector.exchange(request, max_reply_bytes),
            Self::Authenticated(connector) => connector.exchange(request, max_reply_bytes),
        }
    }
}

fn vellum_connector(
    socket: PathBuf,
    credential: Option<&PathBuf>,
) -> Result<VellumConnector, String> {
    if let Some(path) = credential {
        let mut store = vellum_adapter_local_authentication::LocalCredentialStoreV1::default();
        let record = store
            .load(path)
            .map_err(|error| format!("Vellum credential refused: {error:?}"))?;
        let provider =
            vellum_adapter_local_authentication::LocalCredentialClaimProviderV1::new(record);
        Ok(VellumConnector::Authenticated(
            AuthenticatedUnixConnector::with_provider(socket, provider),
        ))
    } else {
        Ok(VellumConnector::Plain(UnixConnector::new(socket)))
    }
}

fn observe_vellum(socket: PathBuf, credential: Option<&PathBuf>) -> (serde_json::Value, bool) {
    use vellum_sdk_client::{protocol::ServiceAccessErrorV1, workspace::WorkspaceClientErrorV1};
    let capability = vellum_tui::CAPABILITIES.headless[0];

    let connector = match vellum_connector(socket, credential) {
        Ok(connector) => connector,
        Err(error) => {
            return (
                peer_failure("vellum", capability, "configuration_refused", error),
                false,
            );
        }
    };
    let transport = match RemoteTransport::new(connector, TransportLimits::reference()) {
        Ok(transport) => transport,
        Err(error) => {
            return (
                peer_failure(
                    "vellum",
                    capability,
                    "transport_refused",
                    format!("{error:?}"),
                ),
                false,
            );
        }
    };
    let mut workspace = match WorkspaceClientV1::connect(transport) {
        Ok(workspace) => workspace,
        Err(error) => {
            return (
                peer_failure(
                    "vellum",
                    capability,
                    "connect_refused",
                    format!("{error:?}"),
                ),
                false,
            );
        }
    };
    let result = match workspace.refresh_inventory() {
        Ok(()) => {
            let inventory = workspace
                .projection()
                .inventory()
                .expect("successful inventory refresh publishes inventory");
            let documents = inventory.documents.clone();
            let roots = inventory.roots.len();
            let mut contexts = Vec::with_capacity(documents.len());
            for document in documents.iter().copied() {
                if let Err(error) = workspace.select_document(document) {
                    return (
                        peer_failure(
                            "vellum",
                            capability,
                            "context_selection_refused",
                            format!("{error:?}"),
                        ),
                        false,
                    );
                }
                if let Err(error) = workspace.refresh_snapshot() {
                    return (
                        peer_failure(
                            "vellum",
                            capability,
                            "context_snapshot_refused",
                            format!("{error:?}"),
                        ),
                        false,
                    );
                }
                let snapshot = workspace
                    .projection()
                    .snapshot()
                    .expect("successful snapshot refresh publishes snapshot");
                let digest: [u8; 32] = Sha256::digest(snapshot.text.as_bytes()).into();
                contexts.push(serde_json::json!({
                    "workspace": document.workspace().get(),
                    "document": document.get(),
                    "revision": snapshot.revision,
                    "witness": snapshot.witness.get(),
                    "bytes": snapshot.text.len(),
                    "sha256": format!("sha256:{}", hex_bytes(&digest)),
                }));
            }
            (
                serde_json::json!({
                    "peer": "vellum",
                    "capability": capability,
                    "status": "current",
                    "documents": documents.len(),
                    "roots": roots,
                    "context_manifest": contexts,
                    "content_disclosure": "not_published",
                }),
                true,
            )
        }
        Err(error) => {
            let status = match error {
                WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::NotSent) => "not_sent",
                WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::OutcomeUnknown) => {
                    "outcome_unknown"
                }
                WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::Unavailable) => "unavailable",
                WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::Malformed) => "invalid_reply",
                WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::Limit) => "limit",
                WorkspaceClientErrorV1::Refused(_) => "refused",
                WorkspaceClientErrorV1::Projection(_) => "invalid_projection",
            };
            (
                peer_failure("vellum", capability, status, "client access failure"),
                false,
            )
        }
    };
    match workspace.disconnect() {
        Ok(_) => result,
        Err(failure) => (
            peer_failure(
                "vellum",
                capability,
                "disconnect_refused",
                format!("{:?}", failure.error()),
            ),
            false,
        ),
    }
}

fn peer_failure(
    peer: &str,
    capability: &str,
    status: &str,
    detail: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "peer": peer,
        "capability": capability,
        "status": status,
        "detail": detail.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daimon_agent_event_projection_is_useful_without_disclosing_content() {
        let record = daimon_protocol::agent::AgentEventRecordV1 {
            cursor: 7,
            event: daimon_protocol::agent::AgentEventV1::Terminal {
                stream: b"stderr-secret".to_vec(),
                bytes: b"provider-secret-output".to_vec(),
            },
        };
        let projection = daimon_agent_event_projection(&record);
        let encoded = serde_json::to_string(&projection).unwrap();
        assert_eq!(projection["cursor"], 7);
        assert_eq!(projection["kind"], "terminal");
        assert_eq!(projection["evidence"]["stream"]["bytes"], 13);
        assert_eq!(projection["evidence"]["output"]["bytes"], 22);
        assert!(!encoded.contains("stderr-secret"));
        assert!(!encoded.contains("provider-secret-output"));
        assert!(!encoded.contains("sha256:"));
    }

    #[test]
    fn daimon_execution_projection_preserves_task_and_bounds_without_inventing_participation() {
        let policy = DaimonExecutionPolicyProfile {
            principal: "10".repeat(16),
            profile: "11".repeat(16),
            task_attempt: crate::profile::DaimonTaskAttemptProfile {
                task_key: "14".repeat(16),
                task_incarnation: 4,
                attempt_number: 2,
            },
            context: vec!["12".repeat(16)],
            tools: vec!["13".repeat(16)],
            budget: crate::profile::DaimonExecutionBudgetProfile {
                max_elapsed_millis: 60_000,
                max_input_tokens: 4_096,
                max_output_tokens: 8_192,
                max_tool_calls: 32,
                max_context_bytes: 1_048_576,
                max_output_bytes: 1_048_576,
            },
        };
        let projection = daimon_execution_projection(Some(&policy));
        assert_eq!(projection["task_attempt"]["task_key"], "14".repeat(16));
        assert_eq!(projection["task_attempt"]["task_incarnation"], 4);
        assert_eq!(projection["task_attempt"]["attempt_number"], 2);
        assert_eq!(projection["context_references"][0], "12".repeat(16));
        assert_eq!(projection["tool_capabilities"][0], "13".repeat(16));
        assert_eq!(projection["budget"]["max_tool_calls"], 32);
        assert_eq!(projection["role_work_mode"], "not_published");
        assert_eq!(projection["delegation"], "not_published");
        assert_eq!(projection["accountable_owner"], "not_published");
        assert!(
            projection["boundary"]
                .as_str()
                .unwrap()
                .contains("not role activation")
        );
        assert!(daimon_execution_projection(None).is_null());
    }

    #[test]
    fn daimon_context_projection_is_exact_and_content_free() {
        use daimon_protocol::{
            ContentDigestV1, ContextKindV1, ContextReferenceIdV1, ExecutionIdV1,
            agent::{AgentContextManifestEntryV1, AgentContextReceiptV1},
        };

        let receipt = AgentContextReceiptV1 {
            execution: ExecutionIdV1::new([1; 16]),
            manifest_digest: ContentDigestV1::new([2; 32]),
            entries: vec![AgentContextManifestEntryV1 {
                reference: ContextReferenceIdV1::new([3; 16]),
                kind: ContextKindV1::VellumDocument,
                source_digest: ContentDigestV1::new([4; 32]),
                content_bytes: 21,
                grounding: None,
            }],
        };
        let projection = daimon_context_projection(Some(&receipt)).unwrap();
        assert_eq!(projection["execution"], "01".repeat(16));
        assert_eq!(
            projection["manifest_digest"],
            format!("sha256:{}", "02".repeat(32))
        );
        assert_eq!(projection["entries"][0]["kind"], "vellum_document");
        assert_eq!(
            projection["entries"][0]["source_digest"],
            format!("sha256:{}", "04".repeat(32))
        );
        assert_eq!(projection["entries"][0]["content_bytes"], 21);
        assert_eq!(projection["content_disclosure"], "not_published");
        assert!(projection.get("content").is_none());
        assert!(daimon_context_projection(None).is_none());
    }

    #[test]
    fn observation_preserves_unexercised_requirement_dispositions() {
        let observation = observe(HeadlessRequest {
            operation_id: "test-gate".into(),
            workspace: "gate".into(),
            agora: Some(AgoraProfile {
                deployment: crate::profile::DeploymentMode::Attach,
                transport: crate::profile::AgoraTransportProfile::Conversation,
                socket: "/definitely/not/a/socket".into(),
                participant: "operator".into(),
                host: None,
                requires: vec!["messages.read.v2".into()],
            }),
            agora_hosted: false,
            task_conversation: None,
            moira_socket: None,
            moira_participation_socket: None,
            moira_participation: None,
            moira_deliberation_socket: None,
            moira_deliberation: None,
            moira_decision_packet_reader: None,
            moira_hosted: false,
            daimon_socket: None,
            daimon_model_socket: None,
            daimon_agent_socket: None,
            daimon_agent_session: None,
            daimon_agent_open: None,
            daimon_execution_policy: None,
            daimon_provider_owner: None,
            daimon_hosted: false,
            hestia: None,
            hermes_socket: None,
            hermes_stream: None,
            hermes_consumer: None,
            hermes_consumer_access: false,
            hermes_hosted: false,
            aegis_socket: None,
            aegis_hosted: false,
            tellus_endpoint: None,
            tellus_token: None,
            chronos: Vec::new(),
            chronos_alignment: Vec::new(),
            chronos_hosted: false,
            vellum_socket: None,
            vellum_credential: None,
            vellum_hosted: false,
            not_exercised_profile_requirements: vec![RequirementDisposition::not_exercised(
                "agora",
                "messages.read.v2".to_owned(),
            )],
        })
        .unwrap();
        assert!(!observation.complete);
        assert_eq!(
            observation.not_exercised_profile_requirements[0].capability,
            "messages.read.v2"
        );
    }

    #[cfg(unix)]
    #[test]
    fn headless_alignment_uses_the_same_verified_owner_endpoint() {
        use chronos_protocol::alignment_v1::{Outcome, RefusalCode, Response};
        use std::{
            io::{BufRead, BufReader, Write},
            os::unix::{fs::PermissionsExt, net::UnixListener},
        };

        let directory = std::path::PathBuf::from("/tmp").join(format!(
            "pantheon-alignment-headless-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = chronos_local::alignment_socket_path(&directory);
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut frame = Vec::new();
            BufReader::new(&stream)
                .read_until(b'\n', &mut frame)
                .unwrap();
            assert!(frame.len() <= chronos_local::MAX_REQUEST_BYTES + 1);
            assert!(String::from_utf8_lossy(&frame).contains("product-delivery"));
            serde_json::to_writer(
                &mut stream,
                &Response {
                    version: 1,
                    outcome: Outcome::Refused {
                        code: RefusalCode::Unavailable,
                    },
                },
            )
            .unwrap();
            stream.write_all(b"\n").unwrap();
        });
        let (observation, current) = observe_chronos_alignment(AlignmentBinding {
            workstream: "Atlas".into(),
            directory: directory.clone(),
            scope: "product-delivery".into(),
        });
        assert!(!current);
        assert_eq!(observation["peer"], "chronos-alignment");
        assert_eq!(observation["status"], "unavailable");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn task_conversation_routing_projection_preserves_exact_identity() {
        let profile = TaskConversationProfile {
            socket: "links.sock".into(),
            principal: "10".repeat(16),
            agent_addressee: None,
            agent_routes: vec![
                crate::profile::TaskConversationAgentRouteProfile {
                    addressee: "sre-agent".into(),
                    session: "07".repeat(16),
                },
                crate::profile::TaskConversationAgentRouteProfile {
                    addressee: "project-agent".into(),
                    session: "08".repeat(16),
                },
            ],
            requires: vec![],
        };
        assert_eq!(
            task_conversation_routing_projection(&profile),
            serde_json::json!({
                "mode": "exact_participant_sessions",
                "routes": [
                    {"addressee": "sre-agent", "session": "07".repeat(16)},
                    {"addressee": "project-agent", "session": "08".repeat(16)},
                ],
            })
        );
    }

    #[test]
    fn operator_agenda_preserves_owner_identity_and_does_not_invent_review_owner() {
        let peers = vec![serde_json::json!({
            "peer": "chronos",
            "instance": "delivery",
            "status": "current",
            "revision": "Finite(7)",
            "agenda_items": [{
                "task": "recover-effect",
                "incarnation": "3",
                "status": "in_progress",
                "execution_phase": "uncertain",
                "priority": 8,
                "dependency_count": 0
            }, {
                "task": "align-product",
                "incarnation": "4",
                "status": "blocked",
                "execution_phase": "dependency_wait",
                "priority": 10,
                "dependency_count": 6
            }]
        })];
        let agenda = project_operator_agenda(&peers);
        assert_eq!(agenda.entries.len(), 2);
        let uncertain = agenda
            .entries
            .iter()
            .find(|entry| entry.resource_identity == "recover-effect#3")
            .unwrap();
        assert_eq!(uncertain.source_peer, "chronos");
        assert_eq!(uncertain.source_instance.as_deref(), Some("delivery"));
        assert_eq!(uncertain.source_revision, "Finite(7)");
        assert_eq!(uncertain.kind, AgendaKind::OutcomeUnknown);
        assert_eq!(uncertain.actionability, AgendaActionability::Actionable);
        let blocked = agenda
            .entries
            .iter()
            .find(|entry| entry.resource_identity == "align-product#4")
            .unwrap();
        assert_eq!(blocked.kind, AgendaKind::DependencyBlocked);
        assert_eq!(blocked.actionability, AgendaActionability::InspectOnly);
        assert!(
            blocked
                .blocker
                .as_deref()
                .is_some_and(|value| value.contains('6'))
        );
        assert!(blocked.next_owner_action.contains("Dependencies"));
        assert!(!agenda.operating_reviews.owner_configured);
    }

    #[test]
    fn operator_agenda_surfaces_alignment_drift_and_gate_without_inventing_control() {
        let alignment = serde_json::json!({
            "peer": "chronos-alignment",
            "instance": "Product",
            "status": "current",
            "observation": {
                "record_revision": "0",
                "currentness": "evidence_changed",
                "proposal": {"namespace":"product-delivery", "document":"pm-proposal"},
                "pending_human_gates": [{
                    "owner":"moira",
                    "resource":"human-gate/1e",
                    "revision":"gate-revision"
                }]
            }
        });
        let agenda = project_operator_agenda(&[alignment]);
        assert_eq!(agenda.entries.len(), 2);
        let drift = agenda
            .entries
            .iter()
            .find(|entry| entry.kind == AgendaKind::ProductAlignment)
            .unwrap();
        assert_eq!(drift.source_instance.as_deref(), Some("Product"));
        assert_eq!(drift.source_revision, "0");
        assert_eq!(drift.actionability, AgendaActionability::InspectOnly);
        let gate = agenda
            .entries
            .iter()
            .find(|entry| entry.kind == AgendaKind::HumanGate)
            .unwrap();
        assert_eq!(gate.resource_identity, "moira/human-gate/1e@gate-revision");
        assert_eq!(gate.actionability, AgendaActionability::InspectOnly);
        assert!(gate.blocker.as_deref().unwrap().contains("cannot resolve"));
    }

    #[test]
    fn direct_moira_gate_supersedes_the_alignment_projection() {
        let alignment = serde_json::json!({
            "peer": "chronos-alignment",
            "instance": "Product",
            "status": "current",
            "observation": {
                "record_revision": "0",
                "currentness": "current",
                "proposal": {"namespace":"product-delivery", "document":"pm-proposal"},
                "pending_human_gates": [{
                    "owner":"moira", "resource":"human-gate/1e", "revision":"gate-revision"
                }]
            }
        });
        let moira = serde_json::json!({
            "peer": "moira-deliberation",
            "status": "current",
            "registry_revision": 12,
            "deliberation": {"human_gate": {
                "status": "pending",
                "request": {"id": "17".repeat(16), "reviewer": "18".repeat(16)}
            }}
        });
        let agenda = project_operator_agenda(&[alignment, moira]);
        assert_eq!(agenda.entries.len(), 1);
        assert_eq!(agenda.entries[0].source_peer, "moira-deliberation");
    }

    #[test]
    fn operator_agenda_routes_only_pending_exact_moira_human_gates() {
        let pending = serde_json::json!({
            "peer": "moira-deliberation",
            "status": "current",
            "registry_revision": 12,
            "deliberation": {
                "human_gate": {
                    "status": "pending",
                    "request": {
                        "id": "17".repeat(16),
                        "reviewer": "18".repeat(16),
                        "finding_digest": "19".repeat(32),
                        "decision_digest": "1a".repeat(32),
                        "material_revision_digest": "1b".repeat(32),
                        "policy_digest": "1c".repeat(32),
                        "decision_packet": {
                            "owner_route": "1d".repeat(16),
                            "artifact_route": "1e".repeat(16),
                            "expected_revision": 28,
                            "expected_sha256": "1f".repeat(32)
                        },
                        "requested_at": 29
                    },
                    "response": null
                }
            }
        });
        let settled = serde_json::json!({
            "peer": "moira-deliberation",
            "status": "current",
            "registry_revision": 13,
            "deliberation": {
                "human_gate": {
                    "status": "settled",
                    "request": {"id": "20".repeat(16), "reviewer": "21".repeat(16)},
                    "response": {"disposition": "rejected"}
                }
            }
        });
        let agenda = project_operator_agenda(&[pending.clone(), settled]);
        assert_eq!(agenda.entries.len(), 1);
        let entry = &agenda.entries[0];
        assert_eq!(entry.kind, AgendaKind::HumanGate);
        assert_eq!(entry.source_peer, "moira-deliberation");
        assert_eq!(entry.source_revision, "12");
        assert_eq!(entry.resource_identity, "17".repeat(16));
        assert!(entry.why_operator.contains(&"18".repeat(16)));
        assert_eq!(entry.actionability, AgendaActionability::InspectOnly);
        assert!(
            entry
                .blocker
                .as_deref()
                .is_some_and(|value| value.contains("no current selected packet reader"))
        );
        assert!(entry.authority.contains("grants none"));
        assert_eq!(entry.route, "navigate.moira.deliberation");

        let mut verified = pending;
        verified["decision_packet_reader"] = serde_json::json!({
            "status": "current",
            "text": "Approve the bounded rollout?"
        });
        let verified_agenda = project_operator_agenda(&[verified]);
        let verified_entry = &verified_agenda.entries[0];
        assert_eq!(
            verified_entry.actionability,
            AgendaActionability::InspectOnly
        );
        assert!(
            verified_entry
                .blocker
                .as_deref()
                .is_some_and(|value| value.contains("verified the exact decision packet"))
        );
        assert!(
            verified_entry
                .requested_action
                .contains("response remains unavailable")
        );
    }
}
