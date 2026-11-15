//! Headless peer observation shared by the standalone CLI and deployments.

use std::net::SocketAddr;
use std::path::PathBuf;

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_tui_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
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

use crate::profile::AgoraProfile;

pub struct HeadlessRequest {
    pub workspace: String,
    pub agora: Option<AgoraProfile>,
    pub moira_socket: Option<PathBuf>,
    pub moira_requires: Vec<String>,
    pub daimon_socket: Option<PathBuf>,
    pub daimon_requires: Vec<String>,
    pub hermes_socket: Option<PathBuf>,
    pub hermes_stream: Option<hermes_protocol::StreamIdV1>,
    pub hermes_consumer: Option<hermes_protocol::ConsumerIdV1>,
    pub hermes_requires: Vec<String>,
    pub aegis_socket: Option<PathBuf>,
    pub aegis_requires: Vec<String>,
    pub grounding_endpoint: Option<SocketAddr>,
    pub grounding_token: Option<PathBuf>,
    pub grounding_requires: Vec<String>,
    pub chronos: Vec<Workstream>,
    pub chronos_requires: Vec<String>,
    pub vellum_socket: Option<PathBuf>,
    pub vellum_credential: Option<PathBuf>,
    pub vellum_requires: Vec<String>,
}

#[derive(Debug)]
pub struct HeadlessObservation {
    pub workspace: String,
    pub complete: bool,
    pub peers: Vec<serde_json::Value>,
}

/// Observes every configured peer through its ordinary public client without
/// constructing presentation state. Healthy peer evidence remains present when
/// another peer refuses or has an uncertain transport outcome.
pub fn observe(request: HeadlessRequest) -> Result<HeadlessObservation, String> {
    if let Some(profile) = &request.agora {
        negotiate("agora", &profile.requires, agora_tui::CAPABILITIES.headless)?;
    }
    negotiate(
        "moira",
        &request.moira_requires,
        moira_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "daimon",
        &request.daimon_requires,
        daimon_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "hermes",
        &request.hermes_requires,
        hermes_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "aegis",
        &request.aegis_requires,
        aegis_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "grounding",
        &request.grounding_requires,
        grounding_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "chronos",
        &request.chronos_requires,
        chronos_tui::CAPABILITIES.headless,
    )?;
    negotiate(
        "vellum",
        &request.vellum_requires,
        vellum_tui::CAPABILITIES.headless,
    )?;
    let mut peers = Vec::new();
    let mut complete = true;

    if let Some(profile) = request.agora {
        let (observation, current) = observe_agora(profile);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = request.moira_socket {
        let (observation, current) = observe_moira(socket);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = request.daimon_socket {
        let (observation, current) = observe_daimon(socket);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = request.hermes_socket {
        let (observation, current) =
            observe_hermes(socket, request.hermes_stream, request.hermes_consumer);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = request.aegis_socket {
        let (observation, current) = observe_aegis(socket);
        peers.push(observation);
        complete &= current;
    }
    if let Some(endpoint) = request.grounding_endpoint {
        let (observation, current) = observe_grounding(endpoint, request.grounding_token.as_ref());
        peers.push(observation);
        complete &= current;
    }
    for workstream in request.chronos {
        let (observation, current) = observe_chronos(workstream);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = request.vellum_socket {
        let (observation, current) = observe_vellum(socket, request.vellum_credential.as_ref());
        peers.push(observation);
        complete &= current;
    }
    if peers.is_empty() {
        return Err("headless observation requires at least one configured peer".into());
    }
    Ok(HeadlessObservation {
        workspace: request.workspace,
        complete,
        peers,
    })
}

fn observe_grounding(
    endpoint: SocketAddr,
    token_path: Option<&PathBuf>,
) -> (serde_json::Value, bool) {
    use grounding_tui::GroundingPort;
    use std::time::Duration;

    let capability = grounding_tui::CAPABILITIES.headless[0];
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
        match grounding_tui_local_client::LocalPort::new(endpoint, token, Duration::from_secs(5)) {
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
    match port.observe(grounding_tui::OBSERVATION_LIMIT) {
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
    let (session, port) = match aegis_tui_local_client::connect(
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
    let port = match hermes_tui_local_client::LocalPort::connect(
        socket,
        hermes_tui_local_client::ClientLimits::default(),
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
        hermes_client::ReadOnlyClient::new(hermes_tui_local_client::CONNECTION_SESSION, port);
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

fn observe_daimon(socket: PathBuf) -> (serde_json::Value, bool) {
    use daimon_client::ReadOnlyClient;
    use daimon_client::unix::ClientErrorV1;
    use daimon_protocol::{OutcomeV1, ReplyV1};
    use std::time::Duration;

    let capability = daimon_tui::CAPABILITIES.headless[0];
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
            (
                serde_json::json!({
                    "peer": "daimon",
                    "capability": capability,
                    "status": "current",
                    "agents": entries.len(),
                    "current": current,
                    "stale": entries.len().saturating_sub(current),
                    "truncated": truncated,
                }),
                true,
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

fn observe_moira(socket: PathBuf) -> (serde_json::Value, bool) {
    let capability = moira_tui::CAPABILITIES.headless[0];
    let port = match moira_tui_local_client::LocalSagaPort::connect(
        socket,
        moira_tui_local_client::ClientLimits::default(),
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
    error: moira_tui_local_client::ClientError,
) -> (serde_json::Value, bool) {
    let status = match error {
        moira_tui_local_client::ClientError::NotSent(_) => "not_sent",
        moira_tui_local_client::ClientError::OutcomeUnknown(_) => "outcome_unknown",
        moira_tui_local_client::ClientError::RequestLimit => "request_limit",
        _ => "invalid_reply",
    };
    (
        peer_failure("moira", capability, status, "client access failure"),
        false,
    )
}

fn negotiate(peer: &str, required: &[String], supported: &[&str]) -> Result<(), String> {
    for capability in required {
        if !supported.contains(&capability.as_str()) {
            return Err(format!(
                "{peer} does not support required headless capability {capability}"
            ));
        }
    }
    Ok(())
}

fn observe_agora(profile: AgoraProfile) -> (serde_json::Value, bool) {
    use agora_client::v2::ClientErrorV2;
    use agora_protocol::v2::{CommandV2, OutcomeV2, ReplyV2};
    let capability = agora_tui::CAPABILITIES.headless[0];

    let (client_id, port) = match AgoraLocalServicePort::connect(
        &profile.socket,
        &profile.participant,
        LocalClientLimits::default(),
    ) {
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
            workspace: "gate".into(),
            agora: Some(AgoraProfile {
                socket: "/definitely/not/a/socket".into(),
                participant: "operator".into(),
                requires: vec!["messages.read.v2".into()],
            }),
            moira_socket: None,
            moira_requires: Vec::new(),
            daimon_socket: None,
            daimon_requires: Vec::new(),
            hermes_socket: None,
            hermes_stream: None,
            hermes_consumer: None,
            hermes_requires: Vec::new(),
            aegis_socket: None,
            aegis_requires: Vec::new(),
            grounding_endpoint: None,
            grounding_token: None,
            grounding_requires: Vec::new(),
            chronos: Vec::new(),
            chronos_requires: Vec::new(),
            vellum_socket: None,
            vellum_credential: None,
            vellum_requires: Vec::new(),
        })
        .unwrap_err();
        assert_eq!(
            failure,
            "agora does not support required headless capability messages.read.v2"
        );
    }
}
