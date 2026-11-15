#![forbid(unsafe_code)]

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_tui_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
use chronos_tui::{Hub, Workstream};
use std::path::PathBuf;
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};
use vellum_tui::ServiceEditor;
use workbench_tui::Shell;
use workbench_tui_cli::{
    profile::{AgoraProfile, WorkspaceProfile},
    run_multi_peer_terminal,
};

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

struct Options {
    workspace_name: String,
    shell: Shell,
    agora: Option<AgoraProfile>,
    chronos_streams: Vec<Workstream>,
    vellum_socket: Option<PathBuf>,
    vellum_credential: Option<PathBuf>,
    chronos_writes: bool,
    headless: bool,
    chronos_watch: bool,
    chronos_scheduling: bool,
    chronos_planning: Option<PathBuf>,
    chronos_cancel_receipts: bool,
    moira_bindings: Vec<(String, PathBuf, String)>,
    daimon_bindings: Vec<(String, PathBuf)>,
    check_profile: bool,
}

fn required_arg(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    usage: &'static str,
) -> Result<std::ffi::OsString, &'static str> {
    args.next()
        .filter(|value| !value.is_empty() && !value.as_encoded_bytes().starts_with(b"--"))
        .ok_or(usage)
}

fn options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return Err(usage().into());
    };
    if first == "--help" {
        println!("{}", usage());
        std::process::exit(0);
    }
    let workspace_profile = first == "--workspace-profile";
    let chronos_profile = first == "--chronos-profile";
    let mut shell = Shell::dual_peer();
    let mut workspace_name = "Workbench".to_owned();
    let mut agora = None;
    let mut vellum_socket = None;
    let mut vellum_credential = None;
    let mut moira_bindings = Vec::new();
    let mut daimon_bindings = Vec::new();
    let mut chronos_planning = None;
    let mut chronos_streams = if workspace_profile {
        let path = required_arg(&mut args, "--workspace-profile requires PATH")?;
        let profile = WorkspaceProfile::load(std::path::Path::new(&path))?;
        workspace_name = profile.name.clone();
        shell = profile.shell()?;
        agora = profile.agora;
        if let Some(vellum) = profile.vellum {
            vellum_socket = Some(vellum.socket);
            vellum_credential = vellum.credential;
        }
        if let Some(chronos) = profile.chronos {
            chronos_planning = chronos.planning;
            moira_bindings = chronos
                .moira
                .into_iter()
                .map(|binding| (binding.workstream, binding.socket, binding.scope))
                .collect();
            daimon_bindings = chronos
                .daimon
                .into_iter()
                .map(|binding| (binding.workstream, binding.socket))
                .collect();
            chronos
                .workstreams
                .into_iter()
                .map(|workstream| Workstream {
                    name: workstream.name,
                    directory: workstream.directory,
                })
                .collect()
        } else {
            Vec::new()
        }
    } else if chronos_profile {
        let path = required_arg(&mut args, "--chronos-profile requires PATH")?;
        chronos_tui::profile::load(std::path::Path::new(&path))?
    } else {
        vec![Workstream {
            name: "Chronos".into(),
            directory: PathBuf::from(first),
        }]
    };
    let mut chronos_writes = false;
    let mut headless = false;
    let mut chronos_watch = false;
    let mut chronos_scheduling = false;
    let mut chronos_cancel_receipts = false;
    let mut check_profile = false;
    while let Some(argument) = args.next() {
        if argument == "--vellum-service" && vellum_socket.is_none() {
            vellum_socket = Some(PathBuf::from(
                args.next().ok_or("--vellum-service requires SOCKET")?,
            ));
        } else if argument == "--vellum-credential" && vellum_credential.is_none() {
            vellum_credential = Some(PathBuf::from(
                args.next().ok_or("--vellum-credential requires PATH")?,
            ));
        } else if argument == "--chronos-allow-writes" && !chronos_writes {
            chronos_writes = true;
        } else if argument == "--chronos-watch" && !chronos_watch {
            chronos_watch = true;
        } else if argument == "--chronos-scheduling" && !chronos_scheduling {
            chronos_scheduling = true;
        } else if argument == "--chronos-cancel-receipts" && !chronos_cancel_receipts {
            chronos_cancel_receipts = true;
        } else if argument == "--chronos-planning" && chronos_planning.is_none() {
            chronos_planning =
                Some(required_arg(&mut args, "--chronos-planning requires PATH")?.into());
        } else if argument == "--chronos-workstream" {
            if workspace_profile || chronos_profile {
                return Err("--chronos-workstream cannot extend a saved profile".into());
            }
            let name = args
                .next()
                .and_then(|name| name.into_string().ok())
                .ok_or("--chronos-workstream requires NAME DIRECTORY")?;
            let directory = args
                .next()
                .filter(|path| !path.as_encoded_bytes().starts_with(b"--"))
                .ok_or("--chronos-workstream requires NAME DIRECTORY")?;
            chronos_streams.push(Workstream {
                name,
                directory: directory.into(),
            });
        } else if argument == "--chronos-moira" {
            let usage = "--chronos-moira requires WORKSTREAM SOCKET SCOPE";
            let name = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| usage)?;
            let socket = required_arg(&mut args, usage)?.into();
            let scope = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| usage)?;
            moira_bindings.push((name, socket, scope));
        } else if argument == "--chronos-daimon" {
            let usage = "--chronos-daimon requires WORKSTREAM SOCKET";
            let name = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| usage)?;
            let socket = required_arg(&mut args, usage)?.into();
            daimon_bindings.push((name, socket));
        } else if (argument == "--headless" || argument == "--snapshot") && !headless {
            headless = true;
        } else if argument == "--check-profile" && workspace_profile && !check_profile {
            check_profile = true;
        } else {
            return Err("unknown or duplicate option".into());
        }
    }
    if vellum_credential.is_some() && vellum_socket.is_none() {
        return Err("--vellum-credential requires --vellum-service".into());
    }
    Ok(Options {
        workspace_name,
        shell,
        agora,
        chronos_streams,
        vellum_socket,
        vellum_credential,
        chronos_writes,
        headless,
        chronos_watch,
        chronos_scheduling,
        chronos_planning,
        chronos_cancel_receipts,
        moira_bindings,
        daimon_bindings,
        check_profile,
    })
}

fn usage() -> &'static str {
    "workbench-tui (--workspace-profile PATH | CHRONOS_PRIVATE_SERVICE_DIRECTORY | --chronos-profile PATH) [--vellum-service SOCKET] [--vellum-credential PATH] [--chronos-allow-writes] [--chronos-watch] [--chronos-scheduling] [--chronos-planning PATH] [--chronos-cancel-receipts] [--chronos-workstream NAME DIRECTORY]… [--chronos-moira WORKSTREAM SOCKET SCOPE] [--chronos-daimon WORKSTREAM SOCKET] [--headless] [--check-profile]\n--headless emits one bounded, versioned JSON observation for every configured peer without constructing TUI state. --snapshot remains a compatibility alias. [/] switches configured surfaces; Tab switches focus inside Workspace; Ctrl-Q exits after persistence checks. Authority-bearing write access remains an explicit launch option."
}

fn vellum_connector(
    socket: PathBuf,
    credential: Option<&PathBuf>,
) -> Result<VellumConnector, Box<dyn std::error::Error>> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = options()?;
    if options.check_profile {
        println!(
            "workspace profile valid: {} surfaces",
            options.shell.surfaces().len()
        );
        return Ok(());
    }
    if options.headless {
        return headless(options);
    }
    let agora = if let Some(profile) = options.agora {
        let (client_id, port) = AgoraLocalServicePort::connect(
            &profile.socket,
            &profile.participant,
            LocalClientLimits::default(),
        )
        .map_err(|error| format!("Agora service refused: {error:?}"))?;
        Some(agora_tui::Component::new(AgoraClientV2::new(
            client_id, port,
        )))
    } else {
        None
    };
    let mut chronos = if options.chronos_streams.is_empty() {
        None
    } else {
        Some(Hub::with_cancel_receipts(
            options.chronos_streams,
            options.chronos_writes,
            options.chronos_scheduling,
            options.chronos_watch,
            options.chronos_cancel_receipts,
        )?)
    };
    if let Some(hub) = &mut chronos {
        if let Some(path) = &options.chronos_planning {
            hub.load_planning(path)?;
        }
        for (name, socket, scope) in options.moira_bindings {
            hub.bind_moira(&name, socket, scope)?;
        }
        for (name, socket) in options.daimon_bindings {
            hub.bind_daimon(&name, socket)?;
        }
    }
    let vellum = if let Some(socket) = options.vellum_socket {
        let connector = vellum_connector(socket, options.vellum_credential.as_ref())?;
        let transport = RemoteTransport::new(connector, TransportLimits::reference())
            .map_err(|error| format!("Vellum transport refused: {error:?}"))?;
        let workspace = WorkspaceClientV1::connect(transport)
            .map_err(|error| format!("Vellum service refused: {error:?}"))?;
        Some(ServiceEditor::open(workspace)?)
    } else {
        None
    };
    run_multi_peer_terminal(options.shell, chronos, vellum, agora)
}

fn headless(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let mut peers = Vec::new();
    let mut complete = true;

    if let Some(profile) = options.agora {
        let (observation, current) = observe_agora(profile);
        peers.push(observation);
        complete &= current;
    }
    for workstream in options.chronos_streams {
        let (observation, current) = observe_chronos(workstream);
        peers.push(observation);
        complete &= current;
    }
    if let Some(socket) = options.vellum_socket {
        let (observation, current) = observe_vellum(socket, options.vellum_credential.as_ref());
        peers.push(observation);
        complete &= current;
    }
    if peers.is_empty() {
        return Err("--headless requires at least one configured peer".into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "workspace": options.workspace_name,
            "mode": "headless",
            "complete": complete,
            "peers": peers,
        }))?
    );
    if complete {
        Ok(())
    } else {
        Err("one or more headless peer observations were not current".into())
    }
}

fn observe_agora(profile: AgoraProfile) -> (serde_json::Value, bool) {
    use agora_client::v2::ClientErrorV2;
    use agora_protocol::v2::{CommandV2, OutcomeV2, ReplyV2};

    let (client_id, port) = match AgoraLocalServicePort::connect(
        &profile.socket,
        &profile.participant,
        LocalClientLimits::default(),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return (
                peer_failure(
                    "agora",
                    "rooms.read.v2",
                    "connect_refused",
                    format!("{error:?}"),
                ),
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
                "capability": "rooms.read.v2",
                "status": "current",
                "revision": page.revision,
                "rooms": page.rooms.len(),
                "truncated": page.next.is_some(),
            }),
            true,
        ),
        Ok(ReplyV2::Refused(reason)) => (
            peer_failure("agora", "rooms.read.v2", "refused", format!("{reason:?}")),
            false,
        ),
        Ok(ReplyV2::Accepted(_)) => (
            peer_failure(
                "agora",
                "rooms.read.v2",
                "invalid_reply",
                "unexpected outcome",
            ),
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
                peer_failure("agora", "rooms.read.v2", status, "client access failure"),
                false,
            )
        }
    }
}

fn observe_chronos(workstream: Workstream) -> (serde_json::Value, bool) {
    use chronos_client::observer::{Observer, ObserverLimits, RefreshError};

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
                    "capability": "tasks.observe.v1",
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
                    "capability": "tasks.observe.v1",
                    "status": status,
                }),
                false,
            )
        }
    }
}

fn observe_vellum(socket: PathBuf, credential: Option<&PathBuf>) -> (serde_json::Value, bool) {
    use vellum_sdk_client::{protocol::ServiceAccessErrorV1, workspace::WorkspaceClientErrorV1};

    let connector = match vellum_connector(socket, credential) {
        Ok(connector) => connector,
        Err(error) => {
            return (
                peer_failure(
                    "vellum",
                    "workspace.inventory.v1",
                    "configuration_refused",
                    error.to_string(),
                ),
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
                    "workspace.inventory.v1",
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
                    "workspace.inventory.v1",
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
                    "capability": "workspace.inventory.v1",
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
                peer_failure(
                    "vellum",
                    "workspace.inventory.v1",
                    status,
                    "client access failure",
                ),
                false,
            )
        }
    };
    match workspace.disconnect() {
        Ok(_) => result,
        Err(failure) => (
            peer_failure(
                "vellum",
                "workspace.inventory.v1",
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
