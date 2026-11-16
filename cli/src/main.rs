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
    OperationalPeers,
    headless::{HeadlessObservation, HeadlessRequest, observe as observe_headless},
    native_host::{AegisProcessOptions, ChronosHostOptions, HermesProcessOptions, NativeHost},
    profile::{
        AegisOpenMode, AegisRole, AgoraProfile, DeploymentMode, HermesOpenMode, WorkspaceProfile,
    },
    registry::{HostedPeerId, ShutdownDisposition, TopologyShutdown},
    run_terminal_with_all_peers,
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
    moira_socket: Option<PathBuf>,
    moira_requires: Vec<String>,
    daimon_socket: Option<PathBuf>,
    daimon_requires: Vec<String>,
    hermes_socket: Option<PathBuf>,
    hermes_stream: Option<hermes_protocol::StreamIdV1>,
    hermes_consumer: Option<hermes_protocol::ConsumerIdV1>,
    hermes_requires: Vec<String>,
    hermes_host: Option<HermesProcessOptions>,
    aegis_socket: Option<PathBuf>,
    aegis_requires: Vec<String>,
    aegis_host: Option<AegisProcessOptions>,
    grounding_endpoint: Option<std::net::SocketAddr>,
    grounding_token: Option<PathBuf>,
    grounding_requires: Vec<String>,
    chronos_streams: Vec<Workstream>,
    chronos_requires: Vec<String>,
    vellum_socket: Option<PathBuf>,
    vellum_credential: Option<PathBuf>,
    vellum_requires: Vec<String>,
    chronos_writes: bool,
    headless: bool,
    chronos_watch: bool,
    chronos_scheduling: bool,
    chronos_planning: Option<PathBuf>,
    chronos_cancel_receipts: bool,
    host_chronos: bool,
    host_vellum: bool,
    vellum_roots: Vec<PathBuf>,
    vellum_allow_edit: bool,
    vellum_allow_save: bool,
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
    let Some(mut first) = args.next() else {
        return Err(usage().into());
    };
    let explicit_mode = if first == "interactive" {
        first = args
            .next()
            .ok_or("interactive requires a workspace or Chronos configuration")?;
        Some(false)
    } else if first == "headless" {
        first = args
            .next()
            .ok_or("headless requires a workspace or Chronos configuration")?;
        Some(true)
    } else {
        None
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
    let mut moira_socket = None;
    let mut moira_requires = Vec::new();
    let mut daimon_socket = None;
    let mut daimon_requires = Vec::new();
    let mut hermes_socket = None;
    let mut hermes_stream = None;
    let mut hermes_consumer = None;
    let mut hermes_requires = Vec::new();
    let mut hermes_host = None;
    let mut aegis_socket = None;
    let mut aegis_requires = Vec::new();
    let mut aegis_host = None;
    let mut grounding_endpoint = None;
    let mut grounding_token = None;
    let mut grounding_requires = Vec::new();
    let mut vellum_socket = None;
    let mut vellum_credential = None;
    let mut moira_bindings = Vec::new();
    let mut daimon_bindings = Vec::new();
    let mut chronos_planning = None;
    let mut agora_requires = Vec::new();
    let mut chronos_requires = Vec::new();
    let mut host_chronos = false;
    let mut host_vellum = false;
    let mut vellum_roots = Vec::new();
    let mut vellum_requires = Vec::new();
    let mut chronos_streams = if workspace_profile {
        let path = required_arg(&mut args, "--workspace-profile requires PATH")?;
        let profile = WorkspaceProfile::load(std::path::Path::new(&path))?;
        workspace_name = profile.name.clone();
        shell = profile.shell()?;
        agora = profile.agora.inspect(|configured| {
            agora_requires = configured.requires.clone();
        });
        if let Some(moira) = profile.moira {
            moira_socket = Some(moira.socket);
            moira_requires = moira.requires;
        }
        if let Some(daimon) = profile.daimon {
            daimon_socket = Some(daimon.socket);
            daimon_requires = daimon.requires;
        }
        if let Some(hermes) = profile.hermes {
            if hermes.deployment == DeploymentMode::Hosted {
                let host = hermes
                    .host
                    .as_ref()
                    .expect("validated hosted Hermes configuration");
                hermes_host = Some(HermesProcessOptions {
                    executable: host.executable.clone(),
                    directory: host.directory.clone(),
                    socket: hermes.socket.clone(),
                    create: host.mode == HermesOpenMode::Create,
                    effective_uid: host.effective_uid,
                    effective_gid: host.effective_gid,
                    subject: host.subject.clone(),
                });
            }
            hermes_socket = Some(hermes.socket);
            hermes_stream = hermes
                .stream
                .as_deref()
                .and_then(hermes_tui::parse_stream_id);
            hermes_consumer = hermes
                .consumer
                .as_deref()
                .and_then(hermes_tui::parse_consumer_id);
            hermes_requires = hermes.requires;
        }
        if let Some(aegis) = profile.aegis {
            if aegis.deployment == DeploymentMode::Hosted {
                let host = aegis
                    .host
                    .as_ref()
                    .expect("validated hosted Aegis configuration");
                aegis_host = Some(AegisProcessOptions {
                    executable: host.executable.clone(),
                    directory: host.directory.clone(),
                    socket: aegis.socket.clone(),
                    create: host.mode == AegisOpenMode::Create,
                    authority: host.authority.clone(),
                    effective_uid: host.effective_uid,
                    effective_gid: host.effective_gid,
                    principal: host.principal.clone(),
                    session_namespace: host.session_namespace.clone(),
                    administrator: host.role == AegisRole::Administrator,
                });
            }
            aegis_socket = Some(aegis.socket);
            aegis_requires = aegis.requires;
        }
        if let Some(grounding) = profile.grounding {
            grounding_endpoint = Some(grounding.endpoint);
            grounding_token = grounding.token;
            grounding_requires = grounding.requires;
        }
        if let Some(vellum) = profile.vellum {
            host_vellum = vellum.deployment == DeploymentMode::Hosted;
            vellum_requires = vellum.requires;
            vellum_socket = Some(vellum.socket);
            vellum_credential = vellum.credential;
            vellum_roots = vellum.roots;
        }
        if let Some(chronos) = profile.chronos {
            host_chronos = chronos.deployment == DeploymentMode::Hosted;
            chronos_requires = chronos.requires;
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
    let mut headless = explicit_mode.unwrap_or(false);
    let mut chronos_watch = false;
    let mut chronos_scheduling = false;
    let mut chronos_cancel_receipts = false;
    let mut check_profile = false;
    let mut vellum_allow_edit = false;
    let mut vellum_allow_save = false;
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
        } else if argument == "--host-chronos" && !host_chronos {
            host_chronos = true;
        } else if argument == "--host-vellum" && !host_vellum {
            host_vellum = true;
        } else if argument == "--vellum-root" {
            vellum_roots.push(required_arg(&mut args, "--vellum-root requires PATH")?.into());
        } else if argument == "--vellum-allow-edit" && !vellum_allow_edit {
            vellum_allow_edit = true;
        } else if argument == "--vellum-allow-save" && !vellum_allow_save {
            vellum_allow_save = true;
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
        } else if (argument == "--headless" || argument == "--snapshot")
            && !headless
            && explicit_mode.is_none()
        {
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
    if host_vellum && vellum_socket.is_none() {
        return Err("--host-vellum requires --vellum-service or a Vellum profile".into());
    }
    if host_vellum && vellum_roots.is_empty() {
        return Err("hosted Vellum requires at least one --vellum-root or profile root".into());
    }
    if !host_vellum && !vellum_roots.is_empty() {
        return Err("--vellum-root requires hosted Vellum".into());
    }
    if (vellum_allow_edit || vellum_allow_save) && !host_vellum {
        return Err("Vellum authority ceiling flags require hosted Vellum".into());
    }
    let mode = if headless { "headless" } else { "interactive" };
    negotiate_capabilities(
        "agora",
        mode,
        &agora_requires,
        if headless {
            agora_tui::CAPABILITIES.headless
        } else {
            agora_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "moira",
        mode,
        &moira_requires,
        if headless {
            moira_tui::CAPABILITIES.headless
        } else {
            moira_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "daimon",
        mode,
        &daimon_requires,
        if headless {
            daimon_tui::CAPABILITIES.headless
        } else {
            daimon_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "hermes",
        mode,
        &hermes_requires,
        if headless {
            hermes_tui::CAPABILITIES.headless
        } else {
            hermes_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "aegis",
        mode,
        &aegis_requires,
        if headless {
            aegis_tui::CAPABILITIES.headless
        } else {
            aegis_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "grounding",
        mode,
        &grounding_requires,
        if headless {
            grounding_tui::CAPABILITIES.headless
        } else {
            grounding_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "chronos",
        mode,
        &chronos_requires,
        if headless {
            chronos_tui::CAPABILITIES.headless
        } else {
            chronos_tui::CAPABILITIES.interactive
        },
    )?;
    negotiate_capabilities(
        "vellum",
        mode,
        &vellum_requires,
        if headless {
            vellum_tui::CAPABILITIES.headless
        } else {
            vellum_tui::CAPABILITIES.interactive
        },
    )?;
    Ok(Options {
        workspace_name,
        shell,
        agora,
        moira_socket,
        moira_requires,
        daimon_socket,
        daimon_requires,
        hermes_socket,
        hermes_stream,
        hermes_consumer,
        hermes_requires,
        hermes_host,
        aegis_socket,
        aegis_requires,
        aegis_host,
        grounding_endpoint,
        grounding_token,
        grounding_requires,
        chronos_streams,
        chronos_requires,
        vellum_socket,
        vellum_credential,
        vellum_requires,
        chronos_writes,
        headless,
        chronos_watch,
        chronos_scheduling,
        chronos_planning,
        chronos_cancel_receipts,
        host_chronos,
        host_vellum,
        vellum_roots,
        vellum_allow_edit,
        vellum_allow_save,
        moira_bindings,
        daimon_bindings,
        check_profile,
    })
}

fn negotiate_capabilities(
    peer: &str,
    mode: &str,
    required: &[String],
    supported: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    for capability in required {
        if !supported.contains(&capability.as_str()) {
            return Err(
                format!("{peer} does not support required {mode} capability {capability}").into(),
            );
        }
    }
    Ok(())
}

fn usage() -> &'static str {
    "workbench-tui [interactive|headless] (--workspace-profile PATH | CHRONOS_PRIVATE_SERVICE_DIRECTORY | --chronos-profile PATH) [--host-chronos] [--vellum-service SOCKET] [--host-vellum --vellum-root PATH…] [--vellum-credential PATH] [--vellum-allow-edit] [--vellum-allow-save] [--chronos-allow-writes] [--chronos-watch] [--chronos-scheduling] [--chronos-planning PATH] [--chronos-cancel-receipts] [--chronos-workstream NAME DIRECTORY]… [--chronos-moira WORKSTREAM SOCKET SCOPE] [--chronos-daimon WORKSTREAM SOCKET] [--check-profile]\nworkbench-tui aegis-revoke-epoch SOCKET EXPECTED_EPOCH 'REVOKE AEGIS EPOCH EXPECTED_EPOCH'\nworkbench-tui aegis-attenuate-authorize-once SOCKET EXPECTED_EPOCH NOW PARENT_EXPIRES_AT CHILD_EXPIRES_AT ENGINE RESOURCE_HEX ACTION 'AUTHORIZE AEGIS EPOCH EXPECTED_EPOCH ENGINE RESOURCE_HEX ACTION UNTIL CHILD_EXPIRES_AT'\nworkbench-tui aegis-manage-session SOCKET\nworkbench-tui hermes-disclose-events SOCKET STREAM_HEX START LIMIT 'DISCLOSE HERMES STREAM STREAM_HEX FROM START LIMIT LIMIT'\nworkbench-tui hermes-acknowledge SOCKET CONSUMER_HEX STREAM_HEX EXPECTED_NEXT THROUGH 'ACKNOWLEDGE HERMES CONSUMER CONSUMER_HEX STREAM STREAM_HEX FROM EXPECTED_NEXT THROUGH THROUGH'\nworkbench-tui hermes-compact SOCKET STREAM_HEX EXPECTED_RETAINED_FROM EXPECTED_PUBLISHED_EDGE THROUGH 'COMPACT HERMES STREAM STREAM_HEX RETAINED FROM EXPECTED_RETAINED_FROM PUBLISHED EDGE EXPECTED_PUBLISHED_EDGE THROUGH THROUGH'\nThe explicit headless command emits one bounded, versioned JSON observation for every configured peer without constructing TUI state. --headless and --snapshot remain compatibility aliases when no command is supplied. Hosted Chronos and Vellum reuse the same peer clients while Workbench owns service startup and drain. Peer commands reuse their owner guards and application contracts: Aegis never retries uncertain authority mutations; Hermes disclosure stays bounded while acknowledgement and compaction observe exact current state then mutate once on one connection without retrying uncertain delivery. [/] switches configured surfaces; Tab switches focus inside Workspace; Ctrl-Q exits after persistence checks. Authority-bearing write access remains an explicit launch option."
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
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "hermes-disclose-events")
    {
        return hermes_disclose_events(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "hermes-acknowledge")
    {
        return hermes_acknowledge(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "hermes-compact")
    {
        return hermes_compact(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "aegis-revoke-epoch")
    {
        return aegis_revoke_epoch(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "aegis-attenuate-authorize-once")
    {
        return aegis_attenuate_authorize_once(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "aegis-manage-session")
    {
        return aegis_manage_session(std::env::args_os().skip(2));
    }
    let explicit_headless = std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "headless");
    let options = match options() {
        Ok(options) => options,
        Err(error) if explicit_headless => {
            return workbench_headless_configuration_failure(&error.to_string());
        }
        Err(error) => return Err(error),
    };
    if options.check_profile {
        println!(
            "workspace profile valid: {} surfaces",
            options.shell.surfaces().len()
        );
        return Ok(());
    }
    let mut native_host = start_native_host(&options)?;
    if options.headless {
        let observation = headless(options);
        let shutdown = shutdown_native_host(&mut native_host);
        return finish_headless(observation, shutdown);
    }
    let result = interactive(options);
    let shutdown = shutdown_native_host(&mut native_host);
    finish_interactive(result, shutdown)
}

fn hermes_compact(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let stream = args
        .next()
        .and_then(|value| hermes_tui::parse_stream_id(&value.to_string_lossy()))
        .ok_or("stream must be 32 hexadecimal digits")?;
    let expected_retained_from = parse_hermes_sequence(args.next())?;
    let expected_published_edge = parse_hermes_sequence(args.next())?;
    let through = parse_hermes_sequence(args.next())?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let port = match hermes_tui_local_client::LocalPort::connect(
        socket,
        hermes_tui_local_client::ClientLimits::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return hermes_compact_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    let request = hermes_operator_command::CompactRequest {
        stream,
        expected_retained_from: hermes_protocol::EventSequenceV1(expected_retained_from),
        expected_published_edge: hermes_protocol::EventSequenceV1(expected_published_edge),
        through: hermes_protocol::EventSequenceV1(through),
    };
    match hermes_operator_command::compact(
        hermes_tui_local_client::CONNECTION_SESSION,
        port,
        request,
        &confirmation,
    ) {
        Ok(outcome) => {
            let (status, removed, retained_from) = match outcome {
                hermes_protocol::CompactionOutcomeV1::Compacted {
                    removed,
                    retained_from,
                } => ("compacted", removed, retained_from.0),
                hermes_protocol::CompactionOutcomeV1::Identity { retained_from } => {
                    ("identity", 0, retained_from.0)
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "hermes",
                    "capability": hermes_operator_command::CAPABILITY,
                    "complete": true,
                    "status": status,
                    "removed": removed,
                    "retained_from": retained_from,
                }))?
            );
            Ok(())
        }
        Err(hermes_operator_command::CompactError::ConfirmationMismatch { required }) => {
            hermes_compact_failure(
                "confirmation_refused",
                "exact compaction confirmation required",
                Some(&required),
                None,
            )
        }
        Err(hermes_operator_command::CompactError::InvalidRange) => hermes_compact_failure(
            "range_refused",
            "compaction boundary must be inside the observed retained range",
            None,
            None,
        ),
        Err(hermes_operator_command::CompactError::StreamMissing) => {
            hermes_compact_failure("stream_absent", "stream is absent", None, None)
        }
        Err(hermes_operator_command::CompactError::StreamChanged { actual }) => {
            hermes_compact_failure(
                "stale_stream",
                "stream retention or publication edge changed",
                None,
                Some((
                    actual.retained_from.0,
                    actual.published_edge.map(|edge| edge.0),
                )),
            )
        }
        Err(hermes_operator_command::CompactError::MutationTransport(
            hermes_tui_local_client::ClientError::OutcomeUnknown(error),
        )) => hermes_compact_failure("outcome_unknown", &format!("{error:?}"), None, None),
        Err(hermes_operator_command::CompactError::MutationRefused(refusal)) => {
            hermes_compact_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => hermes_compact_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn parse_hermes_sequence(
    value: Option<std::ffi::OsString>,
) -> Result<u64, Box<dyn std::error::Error>> {
    value
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| usage().into())
}

fn hermes_compact_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual_stream: Option<(u64, Option<u64>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "hermes",
            "capability": hermes_operator_command::CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_retained_from": actual_stream.map(|value| value.0),
            "actual_published_edge": actual_stream.and_then(|value| value.1),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn hermes_acknowledge(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let consumer = args
        .next()
        .and_then(|value| hermes_tui::parse_consumer_id(&value.to_string_lossy()))
        .ok_or("consumer must be 32 hexadecimal digits")?;
    let stream = args
        .next()
        .and_then(|value| hermes_tui::parse_stream_id(&value.to_string_lossy()))
        .ok_or("stream must be 32 hexadecimal digits")?;
    let expected_next = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(usage)?;
    let through = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let port = match hermes_tui_local_client::LocalPort::connect(
        socket,
        hermes_tui_local_client::ClientLimits::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return hermes_acknowledge_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    let request = hermes_consumer_command::AcknowledgeRequest {
        consumer,
        stream,
        expected_next: hermes_protocol::EventSequenceV1(expected_next),
        through: hermes_protocol::EventSequenceV1(through),
    };
    match hermes_consumer_command::acknowledge(
        hermes_tui_local_client::CONNECTION_SESSION,
        port,
        request,
        &confirmation,
    ) {
        Ok(cursor) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "hermes",
                    "capability": hermes_consumer_command::CAPABILITY,
                    "complete": true,
                    "status": "acknowledged",
                    "previous_next_sequence": expected_next,
                    "next_sequence": cursor.next_sequence.0,
                }))?
            );
            Ok(())
        }
        Err(hermes_consumer_command::AcknowledgeError::ConfirmationMismatch { required }) => {
            hermes_acknowledge_failure(
                "confirmation_refused",
                "exact acknowledgement confirmation required",
                Some(&required),
                None,
            )
        }
        Err(hermes_consumer_command::AcknowledgeError::InvalidRange) => hermes_acknowledge_failure(
            "range_refused",
            "through must not precede the observed next sequence",
            None,
            None,
        ),
        Err(hermes_consumer_command::AcknowledgeError::CursorMissing) => {
            hermes_acknowledge_failure("cursor_absent", "consumer cursor is absent", None, None)
        }
        Err(hermes_consumer_command::AcknowledgeError::CursorChanged { actual }) => {
            hermes_acknowledge_failure(
                "stale_cursor",
                "consumer cursor changed",
                None,
                Some(actual.0),
            )
        }
        Err(hermes_consumer_command::AcknowledgeError::MutationTransport(
            hermes_tui_local_client::ClientError::OutcomeUnknown(error),
        )) => hermes_acknowledge_failure("outcome_unknown", &format!("{error:?}"), None, None),
        Err(hermes_consumer_command::AcknowledgeError::MutationRefused(refusal)) => {
            hermes_acknowledge_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => hermes_acknowledge_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn hermes_acknowledge_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual_next_sequence: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "hermes",
            "capability": hermes_consumer_command::CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_next_sequence": actual_next_sequence,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn hermes_disclose_events(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let stream = args
        .next()
        .and_then(|value| hermes_tui::parse_stream_id(&value.to_string_lossy()))
        .ok_or("stream must be 32 hexadecimal digits")?;
    let start = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(usage)?;
    let limit = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let port = match hermes_tui_local_client::LocalPort::connect(
        socket,
        hermes_tui_local_client::ClientLimits::default(),
    ) {
        Ok(port) => port,
        Err(error) => return hermes_command_failure("not_sent", &format!("{error:?}"), None),
    };
    let request = hermes_disclosure_command::DisclosureRequest {
        stream,
        start: hermes_protocol::EventSequenceV1(start),
        limit,
    };
    match hermes_disclosure_command::disclose(
        hermes_tui_local_client::CONNECTION_SESSION,
        port,
        request,
        &confirmation,
    ) {
        Ok(batch) => {
            let events = batch
                .events
                .into_iter()
                .map(|event| {
                    serde_json::json!({
                        "sequence": event.sequence.0,
                        "producer": hermes_disclosure_command::hex(&event.envelope.producer.0),
                        "schema": hermes_disclosure_command::hex(&event.envelope.schema.0),
                        "idempotency_key": hermes_disclosure_command::hex(&event.envelope.idempotency_key.0),
                        "correlation": hermes_disclosure_command::hex(&event.envelope.correlation.0),
                        "causation": event.envelope.causation.map(|value| hermes_disclosure_command::hex(&value.0)),
                        "payload_hex": hermes_disclosure_command::hex(&event.envelope.payload),
                    })
                })
                .collect::<Vec<_>>();
            let gap = batch.gap.map(|gap| match gap {
                hermes_protocol::ReadGapV1::BeforeRetention { requested, retained_from } => {
                    serde_json::json!({"kind":"before_retention", "requested":requested.0, "retained_from":retained_from.0})
                }
                hermes_protocol::ReadGapV1::AheadOfPublication { requested, next_sequence } => {
                    serde_json::json!({"kind":"ahead_of_publication", "requested":requested.0, "next_sequence":next_sequence.0})
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "hermes",
                    "capability": hermes_disclosure_command::CAPABILITY,
                    "complete": true,
                    "status": "current",
                    "start": start,
                    "limit": limit,
                    "retained_from": batch.retained_from.0,
                    "published_edge": batch.published_edge.map(|value| value.0),
                    "gap": gap,
                    "events": events,
                }))?
            );
            Ok(())
        }
        Err(hermes_disclosure_command::DisclosureError::ConfirmationMismatch { required }) => {
            hermes_command_failure(
                "confirmation_refused",
                "exact disclosure confirmation required",
                Some(&required),
            )
        }
        Err(hermes_disclosure_command::DisclosureError::InvalidLimit) => hermes_command_failure(
            "limit_refused",
            "event limit must be between 1 and 16",
            None,
        ),
        Err(hermes_disclosure_command::DisclosureError::Transport(
            hermes_tui_local_client::ClientError::OutcomeUnknown(error),
        )) => hermes_command_failure("outcome_unknown", &format!("{error:?}"), None),
        Err(hermes_disclosure_command::DisclosureError::Refused(refusal)) => {
            hermes_command_failure("refused", &format!("{refusal:?}"), None)
        }
        Err(error) => hermes_command_failure("unavailable", &format!("{error:?}"), None),
    }
}

fn hermes_command_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "hermes",
            "capability": hermes_disclosure_command::CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn aegis_revoke_epoch(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let expected_epoch = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let (session, port) = match aegis_tui_local_client::connect(
        socket,
        aegis_wire::LimitsV1::default(),
        std::time::Duration::from_secs(5),
    ) {
        Ok(value) => value,
        Err(error) => return aegis_command_failure("not_sent", &format!("{error:?}"), None),
    };
    match aegis_admin_command::revoke_epoch(
        session,
        port,
        aegis_admin_command::RevokeEpoch { expected_epoch },
        &confirmation,
    ) {
        Ok(epoch) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "aegis",
                    "capability": aegis_admin_command::CAPABILITY,
                    "complete": true,
                    "status": "accepted",
                    "expected_epoch": expected_epoch,
                    "epoch": epoch
                }))?
            );
            Ok(())
        }
        Err(aegis_admin_command::RevokeError::ConfirmationMismatch { required }) => {
            aegis_command_failure(
                "confirmation_refused",
                "exact confirmation required",
                Some(&required),
            )
        }
        Err(aegis_admin_command::RevokeError::StaleEpoch { actual, .. }) => {
            aegis_command_failure("stale_epoch", &format!("current epoch is {actual}"), None)
        }
        Err(aegis_admin_command::RevokeError::MutationTransport(
            aegis_tui_local_client::ClientError::OutcomeUnknown(error),
        )) => aegis_command_failure("outcome_unknown", &error.to_string(), None),
        Err(aegis_admin_command::RevokeError::MutationRefused(refusal)) => {
            aegis_command_failure("refused", &format!("{refusal:?}"), None)
        }
        Err(error) => aegis_command_failure("unavailable", &format!("{error:?}"), None),
    }
}

fn aegis_command_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "aegis",
            "capability": aegis_admin_command::CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn aegis_attenuate_authorize_once(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let expected_epoch = aegis_u64(args.next()).ok_or_else(usage)?;
    let now = aegis_u64(args.next()).ok_or_else(usage)?;
    let parent_expires_at = aegis_u64(args.next()).ok_or_else(usage)?;
    let child_expires_at = aegis_u64(args.next()).ok_or_else(usage)?;
    let engine = aegis_text(args.next())
        .and_then(|value| aegis_admin_command::parse_engine(&value))
        .ok_or_else(usage)?;
    let resource = aegis_text(args.next())
        .and_then(|value| aegis_admin_command::parse_resource(&value))
        .ok_or_else(usage)?;
    let action = aegis_text(args.next())
        .and_then(|value| aegis_admin_command::parse_action(&value))
        .ok_or_else(usage)?;
    let confirmation = aegis_text(args.next()).ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let request = aegis_admin_command::AuthorityJourney {
        expected_epoch,
        now,
        parent_expires_at,
        child_expires_at,
        engine,
        resource,
        action,
    };
    let (session, port) = match aegis_tui_local_client::connect(
        socket,
        aegis_wire::LimitsV1::default(),
        std::time::Duration::from_secs(5),
    ) {
        Ok(value) => value,
        Err(error) => return aegis_journey_failure("not_sent", &format!("{error:?}"), None),
    };
    match aegis_admin_command::attenuate_authorize_once(session, port, request, &confirmation) {
        Ok(result) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "aegis",
                    "capability": aegis_admin_command::JOURNEY_CAPABILITY,
                    "complete": true,
                    "status": "authorized",
                    "epoch": result.epoch,
                    "expires_at": result.expires_at,
                    "engine": aegis_admin_command::engine_name(result.engine),
                    "action": aegis_admin_command::action_name(result.action)
                }))?
            );
            Ok(())
        }
        Err(aegis_admin_command::JourneyError::ConfirmationMismatch { required }) => {
            aegis_journey_failure(
                "confirmation_refused",
                "exact confirmation required",
                Some(&required),
            )
        }
        Err(aegis_admin_command::JourneyError::StaleEpoch { actual, .. }) => {
            aegis_journey_failure("stale_epoch", &format!("current epoch is {actual}"), None)
        }
        Err(aegis_admin_command::JourneyError::Transport {
            error: aegis_tui_local_client::ClientError::OutcomeUnknown(error),
            ..
        }) => aegis_journey_failure("outcome_unknown", &error.to_string(), None),
        Err(aegis_admin_command::JourneyError::Refused { refusal, .. }) => {
            aegis_journey_failure("refused", &format!("{refusal:?}"), None)
        }
        Err(error) => aegis_journey_failure("unavailable", &format!("{error:?}"), None),
    }
}

fn aegis_text(value: Option<std::ffi::OsString>) -> Option<String> {
    value?.into_string().ok()
}

fn aegis_u64(value: Option<std::ffi::OsString>) -> Option<u64> {
    aegis_text(value)?.parse().ok()
}

fn aegis_journey_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "aegis",
            "capability": aegis_admin_command::JOURNEY_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn aegis_manage_session(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = match (args.next(), args.next()) {
        (Some(socket), None) => PathBuf::from(socket),
        _ => return Err(usage().into()),
    };
    let (session, port) = aegis_tui_local_client::connect(
        socket,
        aegis_wire::LimitsV1::default(),
        std::time::Duration::from_secs(5),
    )
    .map_err(|error| std::io::Error::other(format!("Aegis connection refused: {error:?}")))?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match aegis_admin_session::run(
        session,
        port,
        stdin.lock(),
        stdout.lock(),
        aegis_admin_session::LimitsV1::default(),
    )? {
        aegis_admin_session::EndV1::Eof | aegis_admin_session::EndV1::Closed => Ok(()),
        aegis_admin_session::EndV1::CommandLimit => {
            Err(std::io::Error::other("Aegis administration command limit reached").into())
        }
        aegis_admin_session::EndV1::Fenced => {
            Err(std::io::Error::other("Aegis administration session fenced").into())
        }
    }
}

fn workbench_headless_configuration_failure(
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "workspace": serde_json::Value::Null,
            "mode": "headless",
            "complete": false,
            "status": "configuration_refused",
            "peers": [],
            "hosted_shutdown": serde_json::Value::Null,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn shutdown_native_host(
    native_host: &mut Option<NativeHost>,
) -> Result<Option<TopologyShutdown>, String> {
    native_host
        .as_mut()
        .map(NativeHost::shutdown)
        .transpose()
        .map_err(|error| format!("hosted topology shutdown refused: {error:?}"))
}

fn finish_interactive(
    result: Result<(), Box<dyn std::error::Error>>,
    shutdown: Result<Option<TopologyShutdown>, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match (result, shutdown) {
        (Ok(()), Ok(Some(receipt))) if !receipt.clean() => Err(format!(
            "hosted topology shutdown incomplete: {:?}",
            receipt.receipts
        )
        .into()),
        (Ok(()), Ok(_)) => Ok(()),
        (Err(error), Ok(Some(receipt))) if !receipt.clean() => Err(format!(
            "{error}; hosted topology shutdown incomplete: {:?}",
            receipt.receipts
        )
        .into()),
        (Err(error), Ok(_)) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Err(shutdown)) => Err(format!("{error}; {shutdown}").into()),
    }
}

fn finish_headless(
    observation: Result<HeadlessObservation, Box<dyn std::error::Error>>,
    shutdown: Result<Option<TopologyShutdown>, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let observation = match observation {
        Ok(observation) => observation,
        Err(error) => return finish_interactive(Err(error), shutdown),
    };
    let (shutdown_report, shutdown_clean) = match shutdown {
        Ok(None) => (serde_json::Value::Null, true),
        Ok(Some(receipt)) => {
            let clean = receipt.clean();
            let receipts = receipt
                .receipts
                .into_iter()
                .map(|receipt| {
                    let (peer, instance) = match receipt.peer {
                        HostedPeerId::Agora => ("agora", None),
                        HostedPeerId::Moira => ("moira", None),
                        HostedPeerId::Daimon => ("daimon", None),
                        HostedPeerId::Hermes => ("hermes", None),
                        HostedPeerId::Aegis => ("aegis", None),
                        HostedPeerId::Grounding => ("grounding", None),
                        HostedPeerId::Chronos(instance) => ("chronos", Some(instance)),
                        HostedPeerId::Vellum => ("vellum", None),
                    };
                    serde_json::json!({
                        "peer": peer,
                        "instance": instance,
                        "disposition": match receipt.disposition {
                            ShutdownDisposition::Clean => "clean",
                            ShutdownDisposition::Refused => "refused",
                            ShutdownDisposition::OutcomeUnknown => "outcome_unknown",
                        },
                    })
                })
                .collect::<Vec<_>>();
            (
                serde_json::json!({
                    "status": if clean { "clean" } else { "incomplete" },
                    "receipts": receipts,
                }),
                clean,
            )
        }
        Err(error) => (
            serde_json::json!({"status":"refused", "detail":error}),
            false,
        ),
    };
    let complete = observation.complete && shutdown_clean;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "workspace": observation.workspace,
            "mode": "headless",
            "complete": complete,
            "peers": observation.peers,
            "hosted_shutdown": shutdown_report,
        }))?
    );
    if complete {
        Ok(())
    } else {
        Err("one or more peer observations or hosted shutdowns were incomplete".into())
    }
}

fn start_native_host(options: &Options) -> Result<Option<NativeHost>, Box<dyn std::error::Error>> {
    if !options.host_chronos
        && !options.host_vellum
        && options.hermes_host.is_none()
        && options.aegis_host.is_none()
    {
        return Ok(None);
    }
    if options.host_chronos && options.chronos_streams.is_empty() {
        return Err("--host-chronos requires at least one Chronos workstream".into());
    }
    let mut host = NativeHost::new();
    for workstream in &options.chronos_streams {
        if !options.host_chronos {
            break;
        }
        host.start_chronos(
            workstream.name.clone(),
            &workstream.directory,
            ChronosHostOptions {
                allow_writes: options.chronos_writes,
                atomic_state: options.chronos_watch || options.chronos_cancel_receipts,
                observer_history: options.chronos_watch,
                allocation_receipts: false,
            },
        )
        .map_err(|failure| format!("hosted Chronos startup refused: {failure:?}"))?;
    }
    if options.host_vellum {
        host.start_vellum(
            options
                .vellum_socket
                .as_deref()
                .expect("validated Vellum socket"),
            &options.vellum_roots,
            options.vellum_credential.as_deref(),
            options.vellum_allow_edit,
            options.vellum_allow_save,
        )
        .map_err(|failure| format!("hosted Vellum startup refused: {failure:?}"))?;
    }
    if let Some(hermes) = &options.hermes_host {
        host.start_hermes_process(hermes.clone())
            .map_err(|failure| format!("hosted Hermes startup refused: {failure:?}"))?;
    }
    if let Some(aegis) = &options.aegis_host {
        host.start_aegis_process(aegis.clone())
            .map_err(|failure| format!("hosted Aegis startup refused: {failure:?}"))?;
    }
    host.seal()
        .map_err(|error| format!("hosted topology seal refused: {error:?}"))?;
    Ok(Some(host))
}

fn interactive(options: Options) -> Result<(), Box<dyn std::error::Error>> {
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
    let moira = if let Some(socket) = options.moira_socket {
        let port = moira_tui_local_client::LocalSagaPort::connect(
            socket,
            moira_tui_local_client::ClientLimits::default(),
        )
        .map_err(|error| format!("Moira service refused: {error:?}"))?;
        Some(moira_tui::Component::new(port))
    } else {
        None
    };
    let daimon = if let Some(socket) = options.daimon_socket {
        let (session, port) = daimon_client::unix::connect(
            socket,
            daimon_wire::LimitsV1::default(),
            std::time::Duration::from_secs(5),
        )
        .map_err(|error| format!("Daimon service refused: {error:?}"))?;
        Some(daimon_tui::Component::new(session, port))
    } else {
        None
    };
    let hermes = if let Some(socket) = options.hermes_socket {
        let port = hermes_tui_local_client::LocalPort::connect(
            socket,
            hermes_tui_local_client::ClientLimits::default(),
        )
        .map_err(|error| format!("Hermes service refused: {error:?}"))?;
        Some(hermes_tui::Component::new_with_selection(
            hermes_tui_local_client::CONNECTION_SESSION,
            port,
            options.hermes_stream,
            options.hermes_consumer,
        ))
    } else {
        None
    };
    let aegis = if let Some(socket) = options.aegis_socket {
        let (session, port) = aegis_tui_local_client::connect(
            socket,
            aegis_wire::LimitsV1::default(),
            std::time::Duration::from_secs(5),
        )
        .map_err(|error| format!("Aegis service refused: {error:?}"))?;
        Some(aegis_tui::Component::new(session, port))
    } else {
        None
    };
    let grounding = if let Some(endpoint) = options.grounding_endpoint {
        let token = options
            .grounding_token
            .as_ref()
            .map(read_grounding_token)
            .transpose()?;
        let port = grounding_tui_local_client::LocalPort::new(
            endpoint,
            token,
            std::time::Duration::from_secs(5),
        )
        .map_err(|error| format!("Grounding service refused: {error:?}"))?;
        Some(grounding_tui::Component::new(port))
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
    run_terminal_with_all_peers(
        options.shell,
        chronos,
        vellum,
        OperationalPeers {
            agora,
            moira,
            daimon,
            hermes,
            aegis,
            grounding,
        },
    )
}

fn headless(options: Options) -> Result<HeadlessObservation, Box<dyn std::error::Error>> {
    observe_headless(HeadlessRequest {
        workspace: options.workspace_name,
        agora: options.agora,
        moira_socket: options.moira_socket,
        moira_requires: options.moira_requires,
        daimon_socket: options.daimon_socket,
        daimon_requires: options.daimon_requires,
        hermes_socket: options.hermes_socket,
        hermes_stream: options.hermes_stream,
        hermes_consumer: options.hermes_consumer,
        hermes_requires: options.hermes_requires,
        aegis_socket: options.aegis_socket,
        aegis_requires: options.aegis_requires,
        grounding_endpoint: options.grounding_endpoint,
        grounding_token: options.grounding_token,
        grounding_requires: options.grounding_requires,
        chronos: options.chronos_streams,
        chronos_requires: options.chronos_requires,
        vellum_socket: options.vellum_socket,
        vellum_credential: options.vellum_credential,
        vellum_requires: options.vellum_requires,
    })
    .map_err(|error| error.into())
}

fn read_grounding_token(path: &std::path::PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    if std::fs::metadata(path)?.len() > 8 * 1024 {
        return Err("Grounding token exceeds 8 KiB".into());
    }
    let token = std::fs::read_to_string(path)?;
    let token = token.trim_end().to_owned();
    if token.is_empty() {
        return Err("Grounding token is empty".into());
    }
    Ok(token)
}
