//! Headless peer observation shared by the standalone CLI and deployments.

use std::net::SocketAddr;
use std::path::PathBuf;

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
use chronos_tui::Workstream;
use moira_client::SagaReadOnlyClient;
use moira_protocol::{SagaOutcomeV1, SagaReplyV1};
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};

use crate::capabilities::{self, Mode};
use crate::management::PeerRuntime;
use crate::profile::{AgoraProfile, AgoraTransportProfile};

pub struct HeadlessRequest {
    pub operation_id: String,
    pub workspace: String,
    pub agora: Option<AgoraProfile>,
    pub agora_hosted: bool,
    pub moira_socket: Option<PathBuf>,
    pub moira_requires: Vec<String>,
    pub moira_hosted: bool,
    pub daimon_socket: Option<PathBuf>,
    pub daimon_agent_socket: Option<PathBuf>,
    pub daimon_agent_session: Option<daimon_protocol::AgentSessionIdV1>,
    pub daimon_requires: Vec<String>,
    pub daimon_hosted: bool,
    pub hermes_socket: Option<PathBuf>,
    pub hermes_stream: Option<hermes_protocol::StreamIdV1>,
    pub hermes_consumer: Option<hermes_protocol::ConsumerIdV1>,
    pub hermes_requires: Vec<String>,
    pub hermes_hosted: bool,
    pub aegis_socket: Option<PathBuf>,
    pub aegis_requires: Vec<String>,
    pub aegis_hosted: bool,
    pub grounding_endpoint: Option<SocketAddr>,
    pub grounding_token: Option<PathBuf>,
    pub grounding_requires: Vec<String>,
    pub chronos: Vec<Workstream>,
    pub chronos_requires: Vec<String>,
    pub chronos_hosted: bool,
    pub vellum_socket: Option<PathBuf>,
    pub vellum_credential: Option<PathBuf>,
    pub vellum_requires: Vec<String>,
    pub vellum_hosted: bool,
}

#[derive(Debug)]
pub struct HeadlessObservation {
    pub operation_id: String,
    pub workspace: String,
    pub complete: bool,
    pub peers: Vec<serde_json::Value>,
    pub management: Vec<PeerRuntime>,
}

/// Observes every configured peer through its ordinary public client without
/// constructing presentation state. Healthy peer evidence remains present when
/// another peer refuses or has an uncertain transport outcome.
pub fn observe(request: HeadlessRequest) -> Result<HeadlessObservation, String> {
    if let Some(profile) = &request.agora {
        capabilities::negotiate(
            "agora",
            Mode::Headless,
            &profile.requires,
            &capabilities::agora(Mode::Headless, request.agora_hosted),
        )?;
    }
    capabilities::negotiate(
        "moira",
        Mode::Headless,
        &request.moira_requires,
        &capabilities::moira(Mode::Headless, request.moira_hosted),
    )?;
    capabilities::negotiate(
        "daimon",
        Mode::Headless,
        &request.daimon_requires,
        &capabilities::daimon(
            Mode::Headless,
            request.daimon_hosted,
            request.daimon_socket.is_some(),
            request.daimon_agent_socket.is_some() && request.daimon_agent_session.is_some(),
        ),
    )?;
    capabilities::negotiate(
        "hermes",
        Mode::Headless,
        &request.hermes_requires,
        &capabilities::hermes(
            Mode::Headless,
            request.hermes_hosted,
            request.hermes_consumer.is_some(),
        ),
    )?;
    capabilities::negotiate(
        "aegis",
        Mode::Headless,
        &request.aegis_requires,
        &capabilities::aegis(request.aegis_hosted),
    )?;
    capabilities::negotiate(
        "grounding",
        Mode::Headless,
        &request.grounding_requires,
        &capabilities::grounding(),
    )?;
    capabilities::negotiate(
        "chronos",
        Mode::Headless,
        &request.chronos_requires,
        &capabilities::chronos(
            Mode::Headless,
            request.chronos_hosted,
            capabilities::ChronosFeatures::default(),
        ),
    )?;
    capabilities::negotiate(
        "vellum",
        Mode::Headless,
        &request.vellum_requires,
        &capabilities::vellum(Mode::Headless, request.vellum_hosted),
    )?;
    let mut peers = Vec::new();
    let mut management = Vec::new();
    let mut complete = true;

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
    if request.daimon_socket.is_some() || request.daimon_agent_socket.is_some() {
        let (observation, current) = observe_daimon(
            request.daimon_socket,
            request.daimon_agent_socket,
            request.daimon_agent_session,
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
    if let Some(socket) = request.hermes_socket {
        let (observation, current) =
            observe_hermes(socket, request.hermes_stream, request.hermes_consumer);
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
    if let Some(endpoint) = request.grounding_endpoint {
        let (observation, current) = observe_grounding(endpoint, request.grounding_token.as_ref());
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
    Ok(HeadlessObservation {
        operation_id: request.operation_id,
        workspace: request.workspace,
        complete,
        peers,
        management,
    })
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

fn observe_grounding(
    endpoint: SocketAddr,
    token_path: Option<&PathBuf>,
) -> (serde_json::Value, bool) {
    use std::time::Duration;
    use tellus::GroundingPort;

    let capability = tellus::CAPABILITIES.headless[0];
    let token = match token_path.map(read_token).transpose() {
        Ok(token) => token,
        Err(error) => {
            return (
                peer_failure("grounding", capability, "configuration_refused", error),
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
                        "grounding",
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
                    "peer": "grounding", "capability": capability,
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
            peer_failure("grounding", capability, "unavailable", format!("{error:?}")),
            false,
        ),
    }
}

fn read_token(path: &PathBuf) -> Result<String, String> {
    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > 8 * 1024 {
        return Err("Grounding token exceeds 8 KiB".into());
    }
    let token = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let token = token.trim_end().to_owned();
    if token.is_empty() {
        return Err("Grounding token is empty".into());
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
    agent_socket: Option<PathBuf>,
    agent_session: Option<daimon_protocol::AgentSessionIdV1>,
) -> (serde_json::Value, bool) {
    use daimon_client::ReadOnlyClient;
    use daimon_client::unix::ClientErrorV1;
    use daimon_protocol::{OutcomeV1, ReplyV1};
    use std::time::Duration;

    let capability = daimon_tui::CAPABILITIES.headless[0];
    let Some(socket) = socket else {
        let (agent, current) = match (agent_socket, agent_session) {
            (Some(socket), Some(session)) => observe_daimon_agent(socket, session),
            _ => (
                serde_json::json!({"status": "configuration_refused"}),
                false,
            ),
        };
        return (
            serde_json::json!({
                "peer": "daimon",
                "capability": "agents.session.inspect.v1",
                "capabilities": ["agents.session.inspect.v1"],
                "status": if current { "current" } else { "partial" },
                "configured_agent_session": agent,
            }),
            current,
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
            let (agent, agent_current) = match (agent_socket, agent_session) {
                (Some(socket), Some(session)) => observe_daimon_agent(socket, session),
                (None, None) => (serde_json::Value::Null, true),
                _ => (
                    serde_json::json!({"status": "configuration_refused"}),
                    false,
                ),
            };
            (
                serde_json::json!({
                    "peer": "daimon",
                    "capability": capability,
                    "capabilities": if agent.is_null() {
                        vec![capability]
                    } else {
                        vec![capability, "agents.session.inspect.v1"]
                    },
                    "status": if agent_current { "current" } else { "partial" },
                    "agents": entries.len(),
                    "current": current,
                    "stale": entries.len().saturating_sub(current),
                    "truncated": truncated,
                    "configured_agent_session": agent,
                }),
                agent_current,
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

fn observe_daimon_agent(
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
            let events = observation
                .events
                .entries
                .iter()
                .map(|record| {
                    serde_json::json!({
                        "cursor": record.cursor,
                        "kind": daimon_agent_event_kind(&record.event),
                    })
                })
                .collect::<Vec<_>>();
            (
                serde_json::json!({
                    "status": "current",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                    "profile_document_digest": hex_bytes(&view.profile_document_digest.into_bytes()),
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
    use chronos_client::observer::{Observer, ObserverLimits, RefreshError};
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
            (
                serde_json::json!({
                    "peer": "chronos",
                    "instance": workstream.name,
                    "capability": capability,
                    "status": "current",
                    "revision": format!("{:?}", view.revision),
                    "tasks": view.entries.len(),
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
            (
                serde_json::json!({
                    "peer": "vellum",
                    "capability": capability,
                    "status": "current",
                    "documents": inventory.documents.len(),
                    "roots": inventory.roots.len(),
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
    fn requirements_are_refused_before_any_peer_connection() {
        let failure = observe(HeadlessRequest {
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
            moira_socket: None,
            moira_requires: Vec::new(),
            moira_hosted: false,
            daimon_socket: None,
            daimon_agent_socket: None,
            daimon_agent_session: None,
            daimon_requires: Vec::new(),
            daimon_hosted: false,
            hermes_socket: None,
            hermes_stream: None,
            hermes_consumer: None,
            hermes_requires: Vec::new(),
            hermes_hosted: false,
            aegis_socket: None,
            aegis_requires: Vec::new(),
            aegis_hosted: false,
            grounding_endpoint: None,
            grounding_token: None,
            grounding_requires: Vec::new(),
            chronos: Vec::new(),
            chronos_requires: Vec::new(),
            chronos_hosted: false,
            vellum_socket: None,
            vellum_credential: None,
            vellum_requires: Vec::new(),
            vellum_hosted: false,
        })
        .unwrap_err();
        assert_eq!(
            failure,
            "agora does not deliver required headless route capability messages.read.v2"
        );
    }
}
