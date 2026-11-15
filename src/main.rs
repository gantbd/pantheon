#![forbid(unsafe_code)]

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_tui_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
use chronos_tui::{Hub, Workstream};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};
use vellum_tui::ServiceEditor;
use workbench_tui::{
    Focus, Shell, draw,
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
    shell: Shell,
    agora: Option<AgoraProfile>,
    chronos_streams: Vec<Workstream>,
    vellum_socket: Option<PathBuf>,
    vellum_credential: Option<PathBuf>,
    chronos_writes: bool,
    snapshot: bool,
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
    let mut agora = None;
    let mut vellum_socket = None;
    let mut vellum_credential = None;
    let mut moira_bindings = Vec::new();
    let mut daimon_bindings = Vec::new();
    let mut chronos_planning = None;
    let mut chronos_streams = if workspace_profile {
        let path = required_arg(&mut args, "--workspace-profile requires PATH")?;
        let profile = WorkspaceProfile::load(std::path::Path::new(&path))?;
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
    let mut snapshot = false;
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
        } else if argument == "--snapshot" && !snapshot {
            snapshot = true;
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
        shell,
        agora,
        chronos_streams,
        vellum_socket,
        vellum_credential,
        chronos_writes,
        snapshot,
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
    "workbench-tui (--workspace-profile PATH | CHRONOS_PRIVATE_SERVICE_DIRECTORY | --chronos-profile PATH) [--vellum-service SOCKET] [--vellum-credential PATH] [--chronos-allow-writes] [--chronos-watch] [--chronos-scheduling] [--chronos-planning PATH] [--chronos-cancel-receipts] [--chronos-workstream NAME DIRECTORY]… [--chronos-moira WORKSTREAM SOCKET SCOPE] [--chronos-daimon WORKSTREAM SOCKET] [--snapshot] [--check-profile]\n[/] switches configured surfaces; Tab switches focus inside Workspace; Ctrl-Q exits after persistence checks. Authority-bearing write access remains an explicit launch option."
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
    if options.snapshot {
        return snapshot(
            chronos.ok_or("--snapshot requires Chronos configuration")?,
            vellum.ok_or("--snapshot requires Vellum configuration")?,
        );
    }
    run_multi_peer_terminal(options.shell, chronos, vellum, agora)
}

fn snapshot(
    mut chronos: Hub,
    vellum: ServiceEditor<RemoteTransport<VellumConnector>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let vellum_frame = vellum.frame(12);
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 18))?;
    loop {
        chronos.poll();
        terminal.draw(|frame| {
            draw(
                frame,
                &chronos,
                vellum_frame.as_ref(),
                Focus::Chronos,
                "dual-service snapshot",
            )
        })?;
        if chronos.has_current_tasks() && vellum_frame.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            return Err("dual-service snapshot was not current".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    vellum
        .disconnect()
        .map_err(|failure| format!("Vellum disconnect refused: {failure:?}"))?;
    println!("dual-service snapshot current");
    Ok(())
}
