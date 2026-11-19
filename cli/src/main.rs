#![forbid(unsafe_code)]

use agora_client::v2::ClientV2 as AgoraClientV2;
use agora_local_client::{LocalClientLimits, LocalServicePort as AgoraLocalServicePort};
use chronos_tui::{Hub, Workstream};
use pantheon::{DaimonModelView, Shell};
use pantheon_cli::{
    AgoraControlConfig, MoiraDecisionPacketComponent, MoiraHumanGateResponseConfig,
    OperationalPeers,
    capabilities::{self, ChronosFeatures, Mode},
    headless::{HeadlessObservation, HeadlessRequest, observe as observe_headless},
    management::LocalManagementSnapshot,
    native_host::{
        AegisProcessOptions, AgoraProcessOptions, ChronosHostOptions,
        ChronosProductAlignmentProcessOptions, ChronosTrustedWorkerOptions, DaimonProcessOptions,
        HermesProcessOptions, MoiraProcessOptions, NativeHost,
    },
    profile::{
        AegisOpenMode, AegisRole, AgoraProfile, AgoraTransportProfile, AlignmentBinding,
        ChronosProductAlignmentProfile, ChronosTrustedWorkerProfile, DeploymentMode,
        HermesAccessProfile, HermesOpenMode, MoiraAgentWorkerProfile, WorkspaceProfile,
    },
    registry::{HostedPeerId, RegistryLifecycle, ShutdownDisposition, TopologyShutdown},
    run_terminal_with_all_peers_managed_configuration,
};
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
    os::unix::process::CommandExt,
    path::PathBuf,
    process::Command,
};
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};
use vellum_tui::ServiceEditor;

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
    operation_id: Option<String>,
    workspace_profile: Option<WorkspaceProfile>,
    workspace_name: String,
    shell: Shell,
    agora: Option<AgoraProfile>,
    agora_host: Option<AgoraProcessOptions>,
    moira_socket: Option<PathBuf>,
    moira_participation_socket: Option<PathBuf>,
    moira_participation: Option<moira_protocol::agent_participation::ParticipationIdV1>,
    moira_deliberation_socket: Option<PathBuf>,
    moira_deliberation: Option<moira_protocol::deliberation::DeliberationIdV1>,
    moira_agent_worker: Option<MoiraAgentWorkerProfile>,
    moira_host: Option<MoiraProcessOptions>,
    daimon_socket: Option<PathBuf>,
    daimon_model_socket: Option<PathBuf>,
    daimon_agent_socket: Option<PathBuf>,
    daimon_agent_session: Option<daimon_protocol::AgentSessionIdV1>,
    daimon_agent_sessions: Vec<daimon_protocol::AgentSessionIdV1>,
    daimon_agent_open: Option<pantheon_cli::profile::DaimonAgentOpenProfile>,
    daimon_execution_policy: Option<pantheon_cli::profile::DaimonExecutionPolicyProfile>,
    daimon_host: Option<DaimonProcessOptions>,
    hermes_socket: Option<PathBuf>,
    hermes_stream: Option<hermes_protocol::StreamIdV1>,
    hermes_consumer: Option<hermes_protocol::ConsumerIdV1>,
    hermes_host: Option<HermesProcessOptions>,
    aegis_socket: Option<PathBuf>,
    aegis_host: Option<AegisProcessOptions>,
    tellus_endpoint: Option<std::net::SocketAddr>,
    tellus_token: Option<PathBuf>,
    chronos_streams: Vec<Workstream>,
    alignment_bindings: Vec<AlignmentBinding>,
    vellum_socket: Option<PathBuf>,
    vellum_credential: Option<PathBuf>,
    chronos_writes: bool,
    headless: bool,
    management: bool,
    chronos_watch: bool,
    chronos_scheduling: bool,
    chronos_planning: Option<PathBuf>,
    chronos_cancel_receipts: bool,
    chronos_trusted_worker: Option<ChronosTrustedWorkerProfile>,
    chronos_product_alignment: Option<ChronosProductAlignmentProfile>,
    host_chronos: bool,
    host_vellum: bool,
    vellum_roots: Vec<PathBuf>,
    vellum_allow_edit: bool,
    vellum_allow_save: bool,
    vellum_allow_add_file: bool,
    moira_bindings: Vec<(String, PathBuf, String)>,
    daimon_bindings: Vec<(String, PathBuf)>,
    check_profile: bool,
    capability_plan: Vec<(&'static str, Vec<capabilities::Capability>)>,
    not_exercised_profile_requirements: Vec<capabilities::RequirementDisposition>,
}

fn required_arg(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    usage: &'static str,
) -> Result<std::ffi::OsString, &'static str> {
    args.next()
        .filter(|value| !value.is_empty() && !value.as_encoded_bytes().starts_with(b"--"))
        .ok_or(usage)
}

fn valid_operation_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn generated_operation_id() -> String {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("pantheon-{}-{:x}", std::process::id(), elapsed.as_nanos())
}

fn options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let Some(mut first) = args.next() else {
        return Err(format!(
            "pantheon controls\npantheon reconfigure-plan CURRENT_PROFILE NEXT_PROFILE [--operation-id ID]\npantheon manage --workspace-profile PATH [--operation-id ID]\n{}",
            usage()
        )
        .into());
    };
    let (explicit_mode, management) = if first == "interactive" {
        first = args
            .next()
            .ok_or("interactive requires a workspace or Chronos configuration")?;
        (Some(false), false)
    } else if first == "headless" {
        first = args
            .next()
            .ok_or("headless requires a workspace or Chronos configuration")?;
        (Some(true), false)
    } else if first == "manage" {
        first = args.next().ok_or("manage requires a workspace profile")?;
        (Some(true), true)
    } else {
        (None, false)
    };
    if first == "--help" {
        println!(
            "pantheon controls\npantheon reconfigure-plan CURRENT_PROFILE NEXT_PROFILE [--operation-id ID]\npantheon manage --workspace-profile PATH [--operation-id ID]\n{}\nhosted Vellum runtime admission: --vellum-allow-add-file",
            usage()
        );
        std::process::exit(0);
    }
    let workspace_profile = first == "--workspace-profile";
    let chronos_profile = first == "--chronos-profile";
    if management && !workspace_profile {
        return Err("manage requires --workspace-profile PATH".into());
    }
    let mut shell = Shell::workbench();
    let mut workspace_name = "Pantheon".to_owned();
    let mut loaded_workspace_profile = None;
    let mut agora = None;
    let mut agora_host = None;
    let mut moira_socket = None;
    let mut moira_participation_socket = None;
    let mut moira_participation = None;
    let mut moira_deliberation_socket = None;
    let mut moira_deliberation = None;
    let mut moira_requires = Vec::new();
    let mut moira_host = None;
    let mut moira_agent_worker = None;
    let mut daimon_socket = None;
    let mut daimon_model_socket = None;
    let mut daimon_agent_socket = None;
    let mut daimon_agent_session = None;
    let mut daimon_agent_sessions = Vec::new();
    let mut daimon_agent_open = None;
    let mut daimon_execution_policy = None;
    let mut daimon_requires = Vec::new();
    let mut daimon_host = None;
    let mut hermes_socket = None;
    let mut hermes_stream = None;
    let mut hermes_consumer = None;
    let mut hermes_requires = Vec::new();
    let mut hermes_host = None;
    let mut aegis_socket = None;
    let mut aegis_requires = Vec::new();
    let mut aegis_host = None;
    let mut tellus_endpoint = None;
    let mut tellus_token = None;
    let mut tellus_requires = Vec::new();
    let mut vellum_socket = None;
    let mut vellum_credential = None;
    let mut moira_bindings = Vec::new();
    let mut daimon_bindings = Vec::new();
    let mut alignment_bindings = Vec::new();
    let mut chronos_planning = None;
    let mut agora_requires = Vec::new();
    let mut task_conversation_socket = None;
    let mut task_conversation_requires = Vec::new();
    let mut chronos_requires = Vec::new();
    let mut chronos_trusted_worker = None;
    let mut chronos_product_alignment = None;
    let mut host_chronos = false;
    let mut host_vellum = false;
    let mut vellum_roots = Vec::new();
    let mut vellum_requires = Vec::new();
    let mut vellum_allow_edit = false;
    let mut vellum_allow_save = false;
    let mut vellum_allow_add_file = false;
    let mut chronos_streams = if workspace_profile {
        let path = required_arg(&mut args, "--workspace-profile requires PATH")?;
        let path = PathBuf::from(path);
        let (profile, profile_digest) = WorkspaceProfile::load_with_digest(&path)?;
        if let Some(expected) = std::env::var_os("PANTHEON_EXPECT_PROFILE_SHA256") {
            let expected = expected
                .into_string()
                .map_err(|_| "expected workspace-profile digest must be UTF-8")?;
            if expected != profile_digest {
                return Err("workspace profile changed after interactive preflight".into());
            }
        }
        loaded_workspace_profile = Some(profile.clone());
        workspace_name = profile.name.clone();
        shell = profile.shell()?;
        agora = profile.agora.inspect(|configured| {
            agora_requires = configured.requires.clone();
        });
        if let Some(task_conversation) = profile.task_conversation {
            task_conversation_socket = Some(task_conversation.socket);
            task_conversation_requires = task_conversation.requires;
        }
        if let Some(configured) = &agora
            && configured.deployment == DeploymentMode::Hosted
        {
            let host = configured
                .host
                .as_ref()
                .expect("validated hosted Agora configuration");
            agora_host = Some(AgoraProcessOptions {
                executable: host.executable.clone(),
                config: host.config.clone(),
                socket: configured.socket.clone(),
            });
        }
        if let Some(moira) = profile.moira {
            moira_agent_worker = moira.agent_worker.clone();
            if moira.deployment == DeploymentMode::Hosted {
                let host = moira
                    .host
                    .as_ref()
                    .expect("validated hosted Moira configuration");
                moira_host = Some(MoiraProcessOptions {
                    executable: host.executable.clone(),
                    config: host.config.clone(),
                    socket: moira
                        .socket
                        .clone()
                        .expect("validated hosted Moira saga socket"),
                });
            }
            moira_participation_socket = moira.participation_socket;
            moira_participation = moira
                .participation
                .as_deref()
                .map(parse_fixed_hex::<16>)
                .transpose()?
                .map(moira_protocol::agent_participation::ParticipationIdV1);
            moira_deliberation_socket = moira.deliberation_socket;
            moira_deliberation = moira
                .deliberation
                .as_deref()
                .map(parse_fixed_hex::<16>)
                .transpose()?
                .map(moira_protocol::deliberation::DeliberationIdV1);
            moira_socket = moira.socket;
            moira_requires = moira.requires;
        }
        if let Some(daimon) = profile.daimon {
            daimon_execution_policy = daimon.execution.clone();
            daimon_agent_open = daimon.agent_open.clone();
            if daimon.deployment == DeploymentMode::Hosted {
                let host = daimon
                    .host
                    .as_ref()
                    .expect("validated hosted Daimon configuration");
                daimon_host = Some(DaimonProcessOptions {
                    executable: host.executable.clone(),
                    config: host.config.clone(),
                    socket: daimon
                        .model_socket
                        .clone()
                        .or_else(|| daimon.agent_socket.clone())
                        .expect("validated hosted Daimon owner endpoint"),
                });
            }
            daimon_socket = daimon.socket;
            daimon_model_socket = daimon.model_socket;
            daimon_agent_socket = daimon.agent_socket;
            daimon_agent_sessions = daimon
                .agent_session
                .iter()
                .chain(daimon.agent_sessions.iter())
                .filter_map(|value| daimon_tui::parse_agent_session(value))
                .collect();
            daimon_agent_session = daimon_agent_sessions.first().copied();
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
                    operator_stream: (host.access == HermesAccessProfile::Operator).then(|| {
                        host.operator_stream
                            .clone()
                            .expect("validated operator Hermes stream")
                    }),
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
        if let Some(tellus) = profile.tellus {
            tellus_endpoint = Some(tellus.endpoint);
            tellus_token = tellus.token;
            tellus_requires = tellus.requires;
        }
        if let Some(vellum) = profile.vellum {
            host_vellum = vellum.deployment == DeploymentMode::Hosted;
            vellum_requires = vellum.requires;
            vellum_socket = Some(vellum.socket);
            vellum_credential = vellum.credential;
            vellum_roots = vellum.roots;
            vellum_allow_add_file = vellum.allow_add_file;
        }
        if let Some(chronos) = profile.chronos {
            host_chronos = chronos.deployment == DeploymentMode::Hosted;
            chronos_requires = chronos.requires;
            chronos_trusted_worker = chronos.trusted_worker;
            chronos_product_alignment = chronos.product_alignment;
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
            alignment_bindings = chronos.alignment;
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
    let mut operation_id = None;
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
        } else if argument == "--vellum-allow-add-file" && !vellum_allow_add_file {
            vellum_allow_add_file = true;
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
        } else if argument == "--chronos-alignment" {
            let usage = "--chronos-alignment requires WORKSTREAM DIRECTORY SCOPE";
            let workstream = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| usage)?;
            let directory = required_arg(&mut args, usage)?.into();
            let scope = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| usage)?;
            alignment_bindings.push(AlignmentBinding {
                workstream,
                directory,
                scope,
            });
        } else if (argument == "--headless" || argument == "--snapshot")
            && !headless
            && explicit_mode.is_none()
        {
            headless = true;
        } else if argument == "--check-profile" && workspace_profile && !check_profile {
            check_profile = true;
        } else if argument == "--operation-id" && operation_id.is_none() {
            let value = required_arg(&mut args, "--operation-id requires ID")?
                .into_string()
                .map_err(|_| "--operation-id must be UTF-8")?;
            if !valid_operation_id(&value) {
                return Err(
                    "--operation-id must be 1-64 ASCII letters, digits, '.', '_', ':', or '-'"
                        .into(),
                );
            }
            operation_id = Some(value);
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
    if (vellum_allow_edit || vellum_allow_save || vellum_allow_add_file) && !host_vellum {
        return Err("Vellum authority ceiling flags require hosted Vellum".into());
    }
    if operation_id.is_some() && !headless {
        return Err("--operation-id is available only in headless mode".into());
    }
    for (index, binding) in alignment_bindings.iter().enumerate() {
        if binding.workstream.is_empty()
            || binding.workstream.len() > 96
            || binding.workstream.chars().any(char::is_control)
            || binding.directory.as_os_str().is_empty()
            || binding.scope.is_empty()
            || binding.scope.len() > 256
            || binding.scope.chars().any(char::is_control)
            || alignment_bindings[..index]
                .iter()
                .any(|prior| prior.workstream == binding.workstream)
            || chronos_streams
                .iter()
                .filter(|stream| stream.name == binding.workstream)
                .count()
                != 1
        {
            return Err("invalid, duplicate, or unmatched --chronos-alignment binding".into());
        }
    }
    let mode = if headless {
        Mode::Headless
    } else {
        Mode::Interactive
    };
    let moira_human_response = loaded_workspace_profile
        .as_ref()
        .and_then(|profile| profile.moira.as_ref())
        .is_some_and(|moira| moira.human_gate_response.is_some());
    let agora_capabilities = capabilities::agora(mode, agora_host.is_some());
    let task_conversation_capabilities = capabilities::task_conversation(mode);
    let moira_capabilities = capabilities::moira(
        mode,
        moira_host.is_some(),
        moira_socket.is_some(),
        moira_participation.is_some(),
        moira_deliberation.is_some(),
        moira_human_response,
    );
    let daimon_capabilities = capabilities::daimon(
        mode,
        daimon_host.is_some(),
        daimon_socket.is_some(),
        daimon_model_socket.is_some(),
        daimon_agent_socket.is_some() && !daimon_agent_sessions.is_empty(),
        daimon_agent_socket.is_some() && daimon_agent_open.is_some(),
    );
    let hermes_capabilities =
        capabilities::hermes(mode, hermes_host.is_some(), hermes_consumer.is_some());
    let aegis_capabilities = capabilities::aegis(aegis_host.is_some());
    let tellus_capabilities = capabilities::tellus();
    let chronos_capabilities = capabilities::chronos(
        mode,
        host_chronos,
        ChronosFeatures {
            writes: chronos_writes,
            scheduling: chronos_scheduling,
            watch: chronos_watch,
            planning: chronos_planning.is_some(),
            cancel_receipts: chronos_cancel_receipts,
            moira_attempts: !moira_bindings.is_empty(),
            daimon_attempts: !daimon_bindings.is_empty(),
            alignment: !alignment_bindings.is_empty(),
        },
    );
    let vellum_capabilities = capabilities::vellum(mode, host_vellum, vellum_credential.is_some());
    let configured_chronos_features = ChronosFeatures {
        writes: chronos_writes,
        scheduling: chronos_scheduling,
        watch: chronos_watch,
        planning: chronos_planning.is_some(),
        cancel_receipts: chronos_cancel_receipts,
        moira_attempts: !moira_bindings.is_empty(),
        daimon_attempts: !daimon_bindings.is_empty(),
        alignment: !alignment_bindings.is_empty(),
    };
    let mut not_exercised_profile_requirements = Vec::new();
    let mut negotiate_selected_route = |peer: &'static str,
                                        required: &[String],
                                        observed: &[capabilities::Capability],
                                        configured: Vec<capabilities::Capability>|
     -> Result<(), Box<dyn std::error::Error>> {
        if mode == Mode::Headless {
            for capability in
                capabilities::negotiate_observation(peer, required, observed, &configured)?
            {
                not_exercised_profile_requirements.push(
                    capabilities::RequirementDisposition::not_exercised(peer, capability),
                );
            }
            Ok(())
        } else {
            negotiate_capabilities(peer, mode, required, observed)
        }
    };
    negotiate_selected_route(
        "agora",
        &agora_requires,
        &agora_capabilities,
        capabilities::agora(Mode::Interactive, agora_host.is_some()),
    )?;
    negotiate_selected_route(
        "task-conversation",
        &task_conversation_requires,
        &[capabilities::Capability {
            id: "task-conversation.v1",
            plane: capabilities::Plane::Observation,
        }],
        task_conversation_capabilities.clone(),
    )?;
    negotiate_selected_route(
        "moira",
        &moira_requires,
        &moira_capabilities,
        capabilities::moira(
            Mode::Interactive,
            moira_host.is_some(),
            moira_socket.is_some(),
            moira_participation.is_some(),
            moira_deliberation.is_some(),
            moira_human_response,
        ),
    )?;
    negotiate_selected_route(
        "daimon",
        &daimon_requires,
        &daimon_capabilities,
        capabilities::daimon(
            Mode::Interactive,
            daimon_host.is_some(),
            daimon_socket.is_some(),
            daimon_model_socket.is_some(),
            daimon_agent_socket.is_some() && !daimon_agent_sessions.is_empty(),
            daimon_agent_socket.is_some() && daimon_agent_open.is_some(),
        ),
    )?;
    negotiate_selected_route(
        "hermes",
        &hermes_requires,
        &hermes_capabilities,
        capabilities::hermes(
            Mode::Interactive,
            hermes_host.is_some(),
            hermes_consumer.is_some(),
        ),
    )?;
    negotiate_selected_route(
        "aegis",
        &aegis_requires,
        &aegis_capabilities,
        capabilities::aegis(aegis_host.is_some()),
    )?;
    negotiate_selected_route(
        "tellus",
        &tellus_requires,
        &tellus_capabilities,
        capabilities::tellus(),
    )?;
    negotiate_selected_route(
        "chronos",
        &chronos_requires,
        &chronos_capabilities,
        capabilities::chronos(Mode::Interactive, host_chronos, configured_chronos_features),
    )?;
    negotiate_selected_route(
        "vellum",
        &vellum_requires,
        &vellum_capabilities,
        capabilities::vellum(Mode::Interactive, host_vellum, vellum_credential.is_some()),
    )?;
    let mut capability_plan = Vec::new();
    if agora.is_some() {
        capability_plan.push(("agora", agora_capabilities));
    }
    if task_conversation_socket.is_some() {
        capability_plan.push(("task-conversation", task_conversation_capabilities));
    }
    if moira_socket.is_some() {
        capability_plan.push(("moira", moira_capabilities));
    }
    if daimon_socket.is_some() || daimon_model_socket.is_some() || daimon_agent_socket.is_some() {
        capability_plan.push(("daimon", daimon_capabilities));
    }
    if hermes_socket.is_some() {
        capability_plan.push(("hermes", hermes_capabilities));
    }
    if aegis_socket.is_some() {
        capability_plan.push(("aegis", aegis_capabilities));
    }
    if tellus_endpoint.is_some() {
        capability_plan.push(("tellus", tellus_capabilities));
    }
    if !chronos_streams.is_empty() {
        capability_plan.push(("chronos", chronos_capabilities));
    }
    if vellum_socket.is_some() {
        capability_plan.push(("vellum", vellum_capabilities));
    }
    Ok(Options {
        operation_id,
        workspace_profile: loaded_workspace_profile,
        workspace_name,
        shell,
        agora,
        agora_host,
        moira_socket,
        moira_participation_socket,
        moira_participation,
        moira_deliberation_socket,
        moira_deliberation,
        moira_agent_worker,
        moira_host,
        daimon_socket,
        daimon_model_socket,
        daimon_agent_socket,
        daimon_agent_session,
        daimon_agent_sessions,
        daimon_agent_open,
        daimon_execution_policy,
        daimon_host,
        hermes_socket,
        hermes_stream,
        hermes_consumer,
        hermes_host,
        aegis_socket,
        aegis_host,
        tellus_endpoint,
        tellus_token,
        chronos_streams,
        alignment_bindings,
        vellum_socket,
        vellum_credential,
        chronos_writes,
        headless,
        management,
        chronos_watch,
        chronos_scheduling,
        chronos_planning,
        chronos_cancel_receipts,
        chronos_trusted_worker,
        chronos_product_alignment,
        host_chronos,
        host_vellum,
        vellum_roots,
        vellum_allow_edit,
        vellum_allow_save,
        vellum_allow_add_file,
        moira_bindings,
        daimon_bindings,
        check_profile,
        capability_plan,
        not_exercised_profile_requirements,
    })
}

fn negotiate_capabilities(
    peer: &str,
    mode: Mode,
    required: &[String],
    supported: &[capabilities::Capability],
) -> Result<(), Box<dyn std::error::Error>> {
    capabilities::negotiate(peer, mode, required, supported).map_err(Into::into)
}

fn usage() -> &'static str {
    concat!(
        "pantheon daimon-agent-open SOCKET REQUEST_JSON 'OPEN DAIMON SESSION …'\n",
        "pantheon task-conversation-inspect SOCKET <task SCOPE TASK INCARNATION ATTEMPT_OR_- | conversation ROOM THREAD_ROOT_OR_->\n",
        "pantheon alignment option: --chronos-alignment WORKSTREAM DIRECTORY SCOPE\n",
        "pantheon [interactive|headless] (--workspace-profile PATH | CHRONOS_PRIVATE_SERVICE_DIRECTORY | --chronos-profile PATH) [--host-chronos] [--vellum-service SOCKET] [--host-vellum --vellum-root PATH…] [--vellum-credential PATH] [--vellum-allow-edit] [--vellum-allow-save] [--chronos-allow-writes] [--chronos-watch] [--chronos-scheduling] [--chronos-planning PATH] [--chronos-cancel-receipts] [--chronos-workstream NAME DIRECTORY]… [--chronos-moira WORKSTREAM SOCKET SCOPE] [--chronos-daimon WORKSTREAM SOCKET] [--check-profile]\npantheon vellum-durable-save <execute|lookup> SOCKET CREDENTIAL vdo1.OPERATION_HEX DOCUMENT_ORDINAL EXPECTED_REVISION\npantheon agora-create-room SOCKET PARTICIPANT <conversation|composition> ROOM_ID NAME <open|invite-only> EXPECTED_REVISION 'CREATE AGORA ROOM ROOM_ID NAMED NAME ADMISSION <OPEN|INVITE ONLY> AT REVISION EXPECTED_REVISION'\npantheon daimon-agent-advance SOCKET AGENT_SESSION_HEX TURN_HEX EXPECTED_REVISION IDEMPOTENCY_KEY_HEX NOW 'ADVANCE DAIMON SESSION …'\npantheon daimon-agent-cancel SOCKET AGENT_SESSION_HEX TURN_HEX EXPECTED_REVISION IDEMPOTENCY_KEY_HEX NOW 'CANCEL DAIMON SESSION …'\npantheon daimon-agent-close SOCKET AGENT_SESSION_HEX EXPECTED_REVISION IDEMPOTENCY_KEY_HEX NOW 'CLOSE DAIMON SESSION …'\npantheon daimon-agent-reattach SOCKET REQUEST_JSON 'REATTACH DAIMON SESSION …'\npantheon daimon-agent-prompt SOCKET REQUEST_JSON PROMPT_FILE 'PROMPT DAIMON SESSION …'\npantheon daimon-agent-permission SOCKET REQUEST_JSON 'DECIDE DAIMON PERMISSION …'\npantheon aegis-revoke-epoch SOCKET EXPECTED_EPOCH 'REVOKE AEGIS EPOCH EXPECTED_EPOCH'\npantheon aegis-attenuate-authorize-once SOCKET EXPECTED_EPOCH NOW PARENT_EXPIRES_AT CHILD_EXPIRES_AT ENGINE RESOURCE_HEX ACTION 'AUTHORIZE AEGIS EPOCH EXPECTED_EPOCH ENGINE RESOURCE_HEX ACTION UNTIL CHILD_EXPIRES_AT'\npantheon aegis-manage-session SOCKET\npantheon hermes-disclose-events SOCKET STREAM_HEX START LIMIT 'DISCLOSE HERMES STREAM STREAM_HEX FROM START LIMIT LIMIT'\npantheon hermes-acknowledge SOCKET CONSUMER_HEX STREAM_HEX EXPECTED_NEXT THROUGH 'ACKNOWLEDGE HERMES CONSUMER CONSUMER_HEX STREAM STREAM_HEX FROM EXPECTED_NEXT THROUGH THROUGH'\npantheon hermes-compact SOCKET STREAM_HEX EXPECTED_RETAINED_FROM EXPECTED_PUBLISHED_EDGE THROUGH 'COMPACT HERMES STREAM STREAM_HEX RETAINED FROM EXPECTED_RETAINED_FROM PUBLISHED EDGE EXPECTED_PUBLISHED_EDGE THROUGH THROUGH'\npantheon moira-reconciliation-view SOCKET SAGA_ID DISPOSITION [ARGS]\npantheon moira-reconciliation-confirm SOCKET STORAGE_REVISION SAGA_ID DISPOSITION [ARGS] ACTION_HEX CONFIRMATION_DIGEST_HEX CONFIRM\npantheon moira-reconciliation-recover SOCKET SAGA_ID ACTION_HEX\nreconciliation dispositions: keep-uncertain | fail-terminal | retry EFFECT_ID NOW | compensate NOW | cancel NOW | admit-evidence EFFECT_ID <succeeded|failed> OBSERVED_AT\nThe explicit headless command emits one bounded, versioned JSON observation for every configured peer without constructing TUI state. --headless and --snapshot remain compatibility aliases when no command is supplied. Hosted Chronos and Vellum reuse the same peer clients while Pantheon owns service startup and drain. Peer commands reuse their owner guards and application contracts: Agora room creation binds the exact room facts and current revision, uses the actor-bound owner client and does not retry an uncertain outcome; Moira accepts only exact server-prepared reconciliation and recovers uncertainty only from durable receipts; Daimon cancellation, close, reattach and permission decisions bind exact current state, prompt bytes stay outside argv, and no uncertain mutation is retried; Aegis never retries uncertain authority mutations; Hermes disclosure stays bounded while acknowledgement and compaction observe exact current state then mutate once on one connection without retrying uncertain delivery. Vellum durable save execute/lookup uses the peer-owned public-client guard and caller-persisted identity; Pantheon grants no authentication, action authority, persistence or retry permission. [/] switches configured surfaces; Tab switches focus inside Workspace; Ctrl-Q exits after persistence checks. Authority-bearing write access remains an explicit launch option."
    )
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

fn guarded_control_catalog(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        return Err("pantheon controls accepts no arguments".into());
    }
    let controls = capabilities::GUARDED_CONTROL_ROUTES
        .iter()
        .map(|route| {
            serde_json::json!({
                "peer": route.peer,
                "capability": route.capability,
                "plane": "control",
                "command": route.command,
                "aggregate_route": false,
                "authority": "peer_owned",
                "confirmation": "route_specific",
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "mode": "guarded_control_catalog",
            "scope": "single_user_local_machine",
            "controls": controls,
        }))?
    );
    Ok(())
}

fn reconfiguration_plan(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current = PathBuf::from(required_arg(
        &mut args,
        "pantheon reconfigure-plan requires CURRENT_PROFILE NEXT_PROFILE",
    )?);
    let next = PathBuf::from(required_arg(
        &mut args,
        "pantheon reconfigure-plan requires CURRENT_PROFILE NEXT_PROFILE",
    )?);
    let operation_id = match (args.next(), args.next(), args.next()) {
        (None, None, None) => generated_operation_id(),
        (Some(flag), Some(value), None) if flag == "--operation-id" => {
            let value = value
                .into_string()
                .map_err(|_| "--operation-id must be UTF-8")?;
            if !valid_operation_id(&value) {
                return Err(
                    "--operation-id must be 1-64 ASCII letters, digits, '.', '_', ':', or '-'"
                        .into(),
                );
            }
            value
        }
        _ => {
            return Err(
                "pantheon reconfigure-plan accepts only CURRENT_PROFILE NEXT_PROFILE [--operation-id ID]"
                    .into(),
            );
        }
    };
    let current = WorkspaceProfile::load(&current)?;
    let next = WorkspaceProfile::load(&next)?;
    let plan = pantheon_cli::reconfiguration::plan(operation_id, &current, &next);
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

fn task_conversation_inspect(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use agora_chronos_adapter::task_conversation_local::{
        ConversationReferenceWireV1, LocalRequestV1, TASK_CONVERSATION_CAPABILITY_V1,
        TaskConversationLocalClientV1, TaskReferenceWireV1,
    };

    let usage = "pantheon task-conversation-inspect SOCKET <task SCOPE TASK INCARNATION ATTEMPT_OR_- | conversation ROOM THREAD_ROOT_OR_->";
    let socket = PathBuf::from(required_arg(&mut args, usage)?);
    let kind = required_arg(&mut args, usage)?
        .into_string()
        .map_err(|_| "task-conversation query kind must be UTF-8")?;
    let (request, query) = match kind.as_str() {
        "task" => {
            let scope = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "task scope must be UTF-8")?;
            let task = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "task identity must be UTF-8")?;
            let incarnation = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "task incarnation must be UTF-8")?;
            let attempt = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "task attempt must be UTF-8")?;
            let attempt =
                if attempt == "-" {
                    None
                } else {
                    Some(attempt.parse::<u32>().map_err(
                        |_| "task attempt must be a positive integer or '-' for task-level",
                    )?)
                };
            if attempt == Some(0) {
                return Err("task attempt must be positive".into());
            }
            let task_reference = TaskReferenceWireV1 {
                scope,
                task,
                incarnation,
                attempt,
            };
            (
                LocalRequestV1::ByTask {
                    task: task_reference.clone(),
                },
                serde_json::json!({"kind":"task","reference":task_reference}),
            )
        }
        "conversation" => {
            let room = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "conversation room must be UTF-8")?;
            let thread_root = required_arg(&mut args, usage)?
                .into_string()
                .map_err(|_| "conversation thread root must be UTF-8")?;
            let thread_root = if thread_root == "-" {
                None
            } else {
                Some(
                    thread_root
                        .parse::<u64>()
                        .map_err(|_| "conversation thread root must be an integer or '-'")?,
                )
            };
            let conversation = ConversationReferenceWireV1 { room, thread_root };
            (
                LocalRequestV1::ByConversation {
                    conversation: conversation.clone(),
                },
                serde_json::json!({"kind":"conversation","reference":conversation}),
            )
        }
        _ => return Err(usage.into()),
    };
    if args.next().is_some() {
        return Err(usage.into());
    }

    let reply = TaskConversationLocalClientV1::new(socket).request(&request)?;
    if reply.version != 1
        || reply.capability != TASK_CONVERSATION_CAPABILITY_V1
        || !reply.complete
        || reply.status != "current"
        || reply.revision.is_none()
    {
        return Err(format!(
            "task-conversation owner refused exact inspection: {} ({})",
            reply.status,
            reply.refusal.as_deref().unwrap_or("unclassified")
        )
        .into());
    }
    let exact = match &request {
        LocalRequestV1::ByTask { task } => reply.links.iter().all(|link| &link.task == task),
        LocalRequestV1::ByConversation { conversation } => reply
            .links
            .iter()
            .all(|link| &link.conversation == conversation),
        _ => false,
    };
    if !exact {
        return Err("task-conversation owner returned a link outside the exact query".into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "mode": "task_conversation_exact_observation",
            "owner": "task-conversation.v1",
            "status": "current",
            "owner_revision": reply.revision,
            "query": query,
            "link_count": reply.links.len(),
            "links": reply.links,
            "complete": true,
            "boundary": "typed relationship truth only; room activity is not task progress",
        }))?
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "controls")
    {
        return guarded_control_catalog(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "reconfigure-plan")
    {
        return reconfiguration_plan(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "task-conversation-inspect")
    {
        return task_conversation_inspect(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "vellum-durable-save")
    {
        return vellum_durable_save(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "agora-create-room")
    {
        return agora_create_room(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "moira-reconciliation-view")
    {
        return moira_reconciliation_view(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "moira-reconciliation-confirm")
    {
        return moira_reconciliation_confirm(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "moira-reconciliation-recover")
    {
        return moira_reconciliation_recover(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-open")
    {
        return daimon_agent_open(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-close")
    {
        return daimon_agent_close(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-reattach")
    {
        return daimon_agent_reattach(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-cancel")
    {
        return daimon_agent_cancel(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-permission")
    {
        return daimon_agent_permission(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-prompt")
    {
        return daimon_agent_prompt(std::env::args_os().skip(2));
    }
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "daimon-agent-advance")
    {
        return daimon_agent_advance(std::env::args_os().skip(2));
    }
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
    let explicit_machine_mode = std::env::args_os().nth(1).and_then(|arg| {
        if arg == "headless" {
            Some("headless")
        } else if arg == "manage" {
            Some("management")
        } else {
            None
        }
    });
    let options = match options() {
        Ok(options) => options,
        Err(error) if explicit_machine_mode.is_some() => {
            return workbench_machine_configuration_failure(
                explicit_machine_mode.expect("matched above"),
                &error.to_string(),
            );
        }
        Err(error) => return Err(error),
    };
    if options.check_profile {
        println!(
            "workspace profile valid: {} surfaces",
            options.shell.surfaces().len()
        );
        print_capability_plan(&options.capability_plan);
        for requirement in &options.not_exercised_profile_requirements {
            println!(
                "{}: not_exercised_by_observe={}",
                requirement.peer, requirement.capability
            );
        }
        return Ok(());
    }
    let mut native_host = match start_native_host(&options) {
        Ok(host) => host,
        Err(error) if explicit_machine_mode.is_some() => {
            return workbench_machine_startup_failure(
                explicit_machine_mode.expect("matched above"),
                &error.to_string(),
            );
        }
        Err(error) => return Err(error),
    };
    if options.management {
        return management_session(options, native_host);
    }
    if options.headless {
        let observation = headless(options);
        let shutdown = shutdown_native_host(&mut native_host);
        return finish_headless(observation, shutdown);
    }
    let result = interactive(options, &mut native_host);
    let shutdown = shutdown_native_host(&mut native_host);
    finish_interactive(result, shutdown)
}

fn print_capability_plan(plan: &[(&str, Vec<capabilities::Capability>)]) {
    for (peer, peer_capabilities) in plan {
        let rendered = [
            capabilities::Plane::Observation,
            capabilities::Plane::Operator,
            capabilities::Plane::Control,
            capabilities::Plane::Management,
        ]
        .into_iter()
        .filter_map(|plane| {
            let identifiers = peer_capabilities
                .iter()
                .filter(|capability| capability.plane == plane)
                .map(|capability| capability.id)
                .collect::<Vec<_>>();
            (!identifiers.is_empty())
                .then(|| format!("{}={}", plane.label(), identifiers.join(",")))
        })
        .collect::<Vec<_>>()
        .join(" ");
        println!("{peer}: {rendered}");
    }
}

fn vellum_durable_save(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use vellum_tui::{DurableSaveCommandV1, run_durable_save_v1};

    let command = match required_arg(&mut args, usage())?.to_str() {
        Some("execute") => DurableSaveCommandV1::Execute,
        Some("lookup") => DurableSaveCommandV1::Lookup,
        _ => return Err("Vellum durable save command must be execute or lookup".into()),
    };
    let socket = PathBuf::from(required_arg(&mut args, usage())?);
    let credential = PathBuf::from(required_arg(&mut args, usage())?);
    let operation = parse_vellum_durable_operation(
        required_arg(&mut args, usage())?
            .to_str()
            .ok_or("Vellum operation identity refused")?,
    )?;
    let document = required_arg(&mut args, usage())?
        .to_str()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value != 0)
        .ok_or("Vellum document ordinal refused")?;
    let revision = required_arg(&mut args, usage())?
        .to_str()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or("Vellum expected revision refused")?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let connector = vellum_connector(socket, Some(&credential))?;
    let transport = RemoteTransport::new(connector, TransportLimits::reference())
        .map_err(|error| format!("Vellum transport refused: {error:?}"))?;
    let report = run_durable_save_v1(transport, command, operation, document, revision)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "peer": "vellum",
            "host": "workbench",
            "capability": match command {
                DurableSaveCommandV1::Execute => "workspace.durable-save.execute.v1",
                DurableSaveCommandV1::Lookup => "workspace.durable-save.lookup.v1",
            },
            "status": report.outcome.status(),
            "disconnect": report.disconnect.status(),
        }))?
    );
    Ok(())
}

fn parse_vellum_durable_operation(
    value: &str,
) -> Result<vellum_sdk_client::protocol::DurableOperationRefV1, Box<dyn std::error::Error>> {
    let hex = value
        .strip_prefix("vdo1.")
        .filter(|hex| hex.len() == 32)
        .ok_or("Vellum operation identity refused")?;
    let mut bytes = [0_u8; 16];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(pair)?;
        bytes[index] =
            u8::from_str_radix(pair, 16).map_err(|_| "Vellum operation identity refused")?;
    }
    vellum_sdk_client::protocol::DurableOperationRefV1::new(bytes)
        .ok_or_else(|| "Vellum operation identity refused".into())
}

fn agora_create_room(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use agora_client::room_command::{
        CreateRoomErrorV1, CreateRoomRequestV1, confirm_create_room, create_room,
    };
    use agora_client::v2::ClientErrorV2;
    use agora_protocol::v2::AdmissionPolicyV2;

    let socket = PathBuf::from(required_arg(&mut args, usage())?);
    let participant = command_text(required_arg(&mut args, usage())?)?;
    let transport = command_text(required_arg(&mut args, usage())?)?;
    let room_id = command_text(required_arg(&mut args, usage())?)?;
    let name = command_text(required_arg(&mut args, usage())?)?;
    let admission = match command_text(required_arg(&mut args, usage())?)?.as_str() {
        "open" => AdmissionPolicyV2::Open,
        "invite-only" => AdmissionPolicyV2::InviteOnly,
        _ => return Err(usage().into()),
    };
    let expected_revision = command_text(required_arg(&mut args, usage())?)?.parse::<u64>()?;
    let confirmation = command_text(required_arg(&mut args, usage())?)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let selected = CreateRoomRequestV1 {
        room_id,
        name,
        admission,
        expected_revision,
    };
    if let Err(refusal) = confirm_create_room(&selected, &confirmation) {
        return agora_create_room_failure(
            "confirmation_refused",
            "exact room creation confirmation required",
            Some(&refusal.required),
        );
    }
    let connection = match transport.as_str() {
        "conversation" => {
            AgoraLocalServicePort::connect(&socket, &participant, LocalClientLimits::default())
        }
        "composition" => AgoraLocalServicePort::connect_composition(
            &socket,
            &participant,
            LocalClientLimits::default(),
        ),
        _ => return Err(usage().into()),
    };
    let (client_id, port) =
        connection.map_err(|error| format!("Agora service refused: {error:?}"))?;
    let mut client = AgoraClientV2::new(client_id, port);
    match create_room(&mut client, selected, &confirmation) {
        Ok(receipt) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "peer": "agora",
                    "capability": agora_client::room_command::CREATE_CAPABILITY,
                    "complete": true,
                    "status": "created",
                    "room_id": receipt.room_id,
                    "revision": receipt.revision,
                }))?
            );
            Ok(())
        }
        Err(CreateRoomErrorV1::ConfirmationRefused { required }) => agora_create_room_failure(
            "confirmation_refused",
            "exact room creation confirmation required",
            Some(&required),
        ),
        Err(CreateRoomErrorV1::Client(ClientErrorV2::NotSent(error))) => {
            agora_create_room_failure("not_sent", &format!("{error:?}"), None)
        }
        Err(CreateRoomErrorV1::Client(ClientErrorV2::OutcomeUnknown(error))) => {
            agora_create_room_failure("outcome_unknown", &format!("{error:?}"), None)
        }
        Err(CreateRoomErrorV1::Client(ClientErrorV2::UnresolvedOutcome)) => {
            agora_create_room_failure("outcome_unknown", "unresolved Agora outcome", None)
        }
        Err(CreateRoomErrorV1::Client(ClientErrorV2::RequestIdExhausted)) => {
            agora_create_room_failure("request_id_exhausted", "request identity exhausted", None)
        }
        Err(CreateRoomErrorV1::Refused(refusal)) => {
            agora_create_room_failure("refused", &format!("{refusal:?}"), None)
        }
        Err(CreateRoomErrorV1::UnexpectedReply) => {
            agora_create_room_failure("invalid_reply", "unexpected Agora reply", None)
        }
    }
}

fn command_text(value: std::ffi::OsString) -> Result<String, &'static str> {
    value
        .into_string()
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(usage)
}

fn agora_create_room_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "peer": "agora",
            "capability": agora_client::room_command::CREATE_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn moira_reconciliation_view(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use moira_reconciliation_command::{PrepareError, prepare};

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let saga = moira_protocol::SagaIdV1(parse_moira_u64(args.next())?);
    let disposition = parse_moira_disposition(&mut args)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let port = match moira_client::unix::SagaUnixPort::connect(
        socket,
        moira_client::unix::SagaClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return moira_reconciliation_failure(
                moira_reconciliation_command::PREPARE_CAPABILITY,
                "not_sent",
                &format!("{error:?}"),
                None,
            );
        }
    };
    let proposal = moira_protocol::ReconciliationProposalV1 { saga, disposition };
    match prepare(port, proposal) {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "moira",
                    "capability": moira_reconciliation_command::PREPARE_CAPABILITY,
                    "complete": true,
                    "status": "current",
                    "storage_revision": value.storage_revision,
                    "saga": value.saga.0,
                    "disposition": format!("{:?}", value.disposition),
                    "action": moira_hex(&value.action.0),
                    "confirmation_digest": moira_hex(&value.confirmation_digest),
                }))?
            );
            Ok(())
        }
        Err(PrepareError::NotAvailable) => moira_reconciliation_failure(
            moira_reconciliation_command::PREPARE_CAPABILITY,
            "not_available",
            "no current reconciliation challenge",
            None,
        ),
        Err(PrepareError::Refused(refusal)) => moira_reconciliation_failure(
            moira_reconciliation_command::PREPARE_CAPABILITY,
            "refused",
            "reconciliation preparation refused",
            Some(format!("{refusal:?}")),
        ),
        Err(PrepareError::Transport(error)) => moira_reconciliation_failure(
            moira_reconciliation_command::PREPARE_CAPABILITY,
            "unavailable",
            &format!("{error:?}"),
            None,
        ),
        Err(PrepareError::InvalidReply) => moira_reconciliation_failure(
            moira_reconciliation_command::PREPARE_CAPABILITY,
            "invalid_reply",
            "unexpected reconciliation preparation reply",
            None,
        ),
    }
}

fn moira_reconciliation_confirm(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use moira_client::unix::SagaClientErrorV1;
    use moira_protocol::SagaOutcomeV1;
    use moira_reconciliation_command::{ConfirmError, confirm};

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let storage_revision = parse_moira_u64(args.next())?;
    let saga = moira_protocol::SagaIdV1(parse_moira_u64(args.next())?);
    let disposition = parse_moira_disposition(&mut args)?;
    let action = moira_protocol::OperatorActionIdV1(parse_moira_hex::<16>(args.next())?);
    let confirmation_digest = parse_moira_hex::<32>(args.next())?;
    if args.next().as_deref() != Some(std::ffi::OsStr::new("CONFIRM")) || args.next().is_some() {
        return Err(usage().into());
    }
    let action_hex = moira_hex(&action.0);
    let port = match moira_client::unix::SagaUnixPort::connect(
        socket,
        moira_client::unix::SagaClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return moira_reconciliation_failure(
                moira_reconciliation_command::CONFIRM_CAPABILITY,
                "not_sent",
                &format!("{error:?}"),
                Some(action_hex),
            );
        }
    };
    let confirmation = moira_protocol::ConfirmedReconciliationV1 {
        storage_revision,
        saga,
        disposition,
        action,
        confirmation_digest,
    };
    match confirm(port, confirmation) {
        Ok(SagaOutcomeV1::Applied {
            storage_revision,
            transition,
            replayed,
        }) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "moira",
                    "capability": moira_reconciliation_command::CONFIRM_CAPABILITY,
                    "complete": true,
                    "status": "accepted",
                    "storage_revision": storage_revision,
                    "transition": format!("{transition:?}"),
                    "action": action_hex,
                    "replayed": replayed,
                }))?
            );
            Ok(())
        }
        Ok(_) | Err(ConfirmError::InvalidReply) => moira_reconciliation_failure(
            moira_reconciliation_command::CONFIRM_CAPABILITY,
            "invalid_reply",
            "unexpected reconciliation confirmation reply",
            Some(action_hex),
        ),
        Err(ConfirmError::Refused(refusal)) => moira_reconciliation_failure(
            moira_reconciliation_command::CONFIRM_CAPABILITY,
            "refused",
            "reconciliation confirmation refused",
            Some(format!("{refusal:?}")),
        ),
        Err(ConfirmError::Transport(SagaClientErrorV1::NotSent(error))) => {
            moira_reconciliation_failure(
                moira_reconciliation_command::CONFIRM_CAPABILITY,
                "not_sent",
                &format!("{error:?}"),
                Some(action_hex),
            )
        }
        Err(ConfirmError::Transport(SagaClientErrorV1::OutcomeUnknown(error))) => {
            moira_reconciliation_failure(
                moira_reconciliation_command::CONFIRM_CAPABILITY,
                "outcome_unknown",
                &format!("{error:?}"),
                Some(action_hex),
            )
        }
        Err(ConfirmError::Transport(error)) => moira_reconciliation_failure(
            moira_reconciliation_command::CONFIRM_CAPABILITY,
            "unavailable",
            &format!("{error:?}"),
            Some(action_hex),
        ),
    }
}

fn moira_reconciliation_recover(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use moira_reconciliation_command::{RecoverError, recover};

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let saga = moira_protocol::SagaIdV1(parse_moira_u64(args.next())?);
    let action = moira_protocol::OperatorActionIdV1(parse_moira_hex::<16>(args.next())?);
    if args.next().is_some() {
        return Err(usage().into());
    }
    let action_hex = moira_hex(&action.0);
    let port = match moira_client::unix::SagaUnixPort::connect(
        socket,
        moira_client::unix::SagaClientLimitsV1::default(),
    ) {
        Ok(port) => port,
        Err(error) => {
            return moira_reconciliation_failure(
                moira_reconciliation_command::RECOVER_CAPABILITY,
                "not_sent",
                &format!("{error:?}"),
                Some(action_hex),
            );
        }
    };
    match recover(port, saga, action) {
        Ok(receipt) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "moira",
                    "capability": moira_reconciliation_command::RECOVER_CAPABILITY,
                    "complete": true,
                    "status": "recovered",
                    "action": action_hex,
                    "saga": receipt.saga.0,
                    "authority_decision": receipt.authority_decision,
                    "authority_epoch": receipt.authority_epoch,
                    "disposition": format!("{:?}", receipt.disposition),
                    "phase": format!("{:?}", receipt.phase),
                    "prepared_effect": receipt.prepared.map(|effect| effect.0),
                }))?
            );
            Ok(())
        }
        Err(RecoverError::NotFound) => moira_reconciliation_failure(
            moira_reconciliation_command::RECOVER_CAPABILITY,
            "not_found",
            "durable operator receipt not found",
            Some(action_hex),
        ),
        Err(RecoverError::Mismatch) => moira_reconciliation_failure(
            moira_reconciliation_command::RECOVER_CAPABILITY,
            "mismatch",
            "operator receipt identity mismatch",
            Some(action_hex),
        ),
        Err(RecoverError::Refused(refusal)) => moira_reconciliation_failure(
            moira_reconciliation_command::RECOVER_CAPABILITY,
            "refused",
            "operator receipt recovery refused",
            Some(format!("{refusal:?}")),
        ),
        Err(RecoverError::Transport(error)) => moira_reconciliation_failure(
            moira_reconciliation_command::RECOVER_CAPABILITY,
            "unavailable",
            &format!("{error:?}"),
            Some(action_hex),
        ),
        Err(RecoverError::InvalidReply) => moira_reconciliation_failure(
            moira_reconciliation_command::RECOVER_CAPABILITY,
            "invalid_reply",
            "unexpected operator receipt reply",
            Some(action_hex),
        ),
    }
}

fn parse_moira_disposition<I>(
    args: &mut I,
) -> Result<moira_protocol::OperatorDispositionV1, Box<dyn std::error::Error>>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    use moira_protocol::{OperatorDispositionV1, SagaEffectIdV1, SagaEffectOutcomeV1};
    let kind = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    Ok(match kind.as_str() {
        "keep-uncertain" => OperatorDispositionV1::KeepUncertain,
        "fail-terminal" => OperatorDispositionV1::FailTerminal,
        "retry" => OperatorDispositionV1::Retry {
            failed_effect: SagaEffectIdV1(parse_moira_u64(args.next())?),
            now: parse_moira_u64(args.next())?,
        },
        "compensate" => OperatorDispositionV1::Compensate {
            now: parse_moira_u64(args.next())?,
        },
        "cancel" => OperatorDispositionV1::Cancel {
            now: parse_moira_u64(args.next())?,
        },
        "admit-evidence" => {
            let effect = SagaEffectIdV1(parse_moira_u64(args.next())?);
            let outcome = match args
                .next()
                .and_then(|value| value.into_string().ok())
                .as_deref()
            {
                Some("succeeded") => SagaEffectOutcomeV1::Succeeded,
                Some("failed") => SagaEffectOutcomeV1::Failed,
                _ => return Err(usage().into()),
            };
            OperatorDispositionV1::AdmitEvidence {
                effect,
                outcome,
                observed_at: parse_moira_u64(args.next())?,
            }
        }
        _ => return Err(usage().into()),
    })
}

fn parse_moira_u64(value: Option<std::ffi::OsString>) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(value.ok_or_else(usage)?.to_string_lossy().parse()?)
}

fn parse_moira_hex<const N: usize>(
    value: Option<std::ffi::OsString>,
) -> Result<[u8; N], Box<dyn std::error::Error>> {
    let value = value
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(usage().into());
    }
    let mut result = [0_u8; N];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

fn parse_fixed_hex<const N: usize>(value: &str) -> Result<[u8; N], Box<dyn std::error::Error>> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("identifier must be fixed-width hexadecimal".into());
    }
    let mut result = [0_u8; N];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

fn moira_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn moira_reconciliation_failure(
    capability: &str,
    status: &str,
    message: &str,
    detail: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "moira",
            "capability": capability,
            "complete": false,
            "status": status,
            "detail": detail,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_advance(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{AdvanceError, AdvanceRequest};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let session = args
        .next()
        .and_then(|value| daimon_tui::parse_agent_session(&value.to_string_lossy()))
        .ok_or("agent session must be 32 hexadecimal digits")?;
    let turn = args
        .next()
        .and_then(|value| daimon_tui::parse_agent_turn(&value.to_string_lossy()))
        .ok_or("agent turn must be 32 hexadecimal digits")?;
    let expected_revision = parse_daimon_u64(args.next())?;
    let idempotency_key = args
        .next()
        .and_then(|value| daimon_tui::parse_idempotency_key(&value.to_string_lossy()))
        .ok_or("idempotency key must be 32 hexadecimal digits")?;
    let now = parse_daimon_u64(args.next())?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_advance_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    let request = AdvanceRequest {
        session,
        turn,
        expected_revision,
        idempotency_key,
        now,
    };
    match daimon_agent_command::advance(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::CAPABILITY,
                    "complete": true,
                    "status": "advanced",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_advance_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate advance outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_advance_failure(
            "invalid_reply",
            "unexpected Daimon advance reply",
            None,
            None,
        ),
        Err(AdvanceError::ConfirmationMismatch { required }) => daimon_advance_failure(
            "confirmation_refused",
            "exact Daimon advance confirmation required",
            Some(&required),
            None,
        ),
        Err(AdvanceError::SessionChanged { actual }) => daimon_advance_failure(
            "stale_session",
            "Daimon session revision, phase, or active turn changed",
            None,
            Some(&actual),
        ),
        Err(AdvanceError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_advance_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(AdvanceError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(AdvanceError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_advance_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(AdvanceError::ObservationRefused(refusal))
        | Err(AdvanceError::MutationRefused(refusal)) => {
            daimon_advance_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_advance_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn daimon_agent_cancel(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{CancelError, CancelRequest};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let session = args
        .next()
        .and_then(|value| daimon_tui::parse_agent_session(&value.to_string_lossy()))
        .ok_or("agent session must be 32 hexadecimal digits")?;
    let turn = args
        .next()
        .and_then(|value| daimon_tui::parse_agent_turn(&value.to_string_lossy()))
        .ok_or("agent turn must be 32 hexadecimal digits")?;
    let expected_revision = parse_daimon_u64(args.next())?;
    let idempotency_key = args
        .next()
        .and_then(|value| daimon_tui::parse_idempotency_key(&value.to_string_lossy()))
        .ok_or("idempotency key must be 32 hexadecimal digits")?;
    let now = parse_daimon_u64(args.next())?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_cancel_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    let request = CancelRequest {
        session,
        turn,
        expected_revision,
        idempotency_key,
        now,
    };
    match daimon_agent_command::cancel(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::CANCEL_CAPABILITY,
                    "complete": true,
                    "status": "cancelled",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_cancel_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate cancellation outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_cancel_failure(
            "invalid_reply",
            "unexpected Daimon cancellation reply",
            None,
            None,
        ),
        Err(CancelError::ConfirmationMismatch { required }) => daimon_cancel_failure(
            "confirmation_refused",
            "exact Daimon cancellation confirmation required",
            Some(&required),
            None,
        ),
        Err(CancelError::SessionChanged { actual }) => daimon_cancel_failure(
            "stale_session",
            "Daimon session revision, phase, or active turn changed",
            None,
            Some(&actual),
        ),
        Err(CancelError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_cancel_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(CancelError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(CancelError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_cancel_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(CancelError::ObservationRefused(refusal))
        | Err(CancelError::MutationRefused(refusal)) => {
            daimon_cancel_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_cancel_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn daimon_cancel_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::CANCEL_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_close(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{CloseError, CloseRequest};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let session = args
        .next()
        .and_then(|value| daimon_tui::parse_agent_session(&value.to_string_lossy()))
        .ok_or("agent session must be 32 hexadecimal digits")?;
    let expected_revision = parse_daimon_u64(args.next())?;
    let idempotency_key = args
        .next()
        .and_then(|value| daimon_tui::parse_idempotency_key(&value.to_string_lossy()))
        .ok_or("idempotency key must be 32 hexadecimal digits")?;
    let now = parse_daimon_u64(args.next())?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_close_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    let request = CloseRequest {
        session,
        expected_revision,
        idempotency_key,
        now,
    };
    match daimon_agent_command::close(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::CLOSE_CAPABILITY,
                    "complete": true,
                    "status": "closed",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_close_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate close outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => {
            daimon_close_failure("invalid_reply", "unexpected Daimon close reply", None, None)
        }
        Err(CloseError::ConfirmationMismatch { required }) => daimon_close_failure(
            "confirmation_refused",
            "exact Daimon close confirmation required",
            Some(&required),
            None,
        ),
        Err(CloseError::SessionChanged { actual }) => daimon_close_failure(
            "stale_session",
            "Daimon session must remain at the exact ready revision with no active turn",
            None,
            Some(&actual),
        ),
        Err(CloseError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_close_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(CloseError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(CloseError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_close_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(CloseError::ObservationRefused(refusal))
        | Err(CloseError::MutationRefused(refusal)) => {
            daimon_close_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_close_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn daimon_close_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::CLOSE_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_open(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{OpenError, parse_open_document};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let document_path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let document = read_daimon_bounded(&document_path, 64 * 1024)?;
    let request = parse_open_document(&document)
        .map_err(|error| format!("Daimon session-open document refused: {error:?}"))?;
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => return daimon_open_failure("not_sent", &format!("{error:?}"), None),
    };
    match daimon_agent_command::open(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::OPEN_CAPABILITY,
                    "complete": true,
                    "status": "opened",
                    "session": moira_hex(&view.session.into_bytes()),
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_open_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate session-open outcome; inspect the exact session before any further action",
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_open_failure(
            "invalid_reply",
            "unexpected Daimon session-open reply",
            None,
        ),
        Err(OpenError::ConfirmationMismatch { required }) => daimon_open_failure(
            "confirmation_refused",
            "exact Daimon session-open confirmation required",
            Some(&required),
        ),
        Err(OpenError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_open_failure("outcome_unknown", &format!("{error:?}"), None)
        }
        Err(OpenError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_open_failure("not_sent", &format!("{error:?}"), None)
        }
        Err(OpenError::MutationRefused(refusal)) => {
            daimon_open_failure("refused", &format!("{refusal:?}"), None)
        }
        Err(error) => daimon_open_failure("unavailable", &format!("{error:?}"), None),
    }
}

fn daimon_open_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::OPEN_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_reattach(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{ReattachError, parse_reattach_document};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let document_path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let document = read_daimon_bounded(&document_path, 64 * 1024)?;
    let request = parse_reattach_document(&document)
        .map_err(|error| format!("Daimon reattach document refused: {error:?}"))?;
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_reattach_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    match daimon_agent_command::reattach(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::REATTACH_CAPABILITY,
                    "complete": true,
                    "status": "reattached",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_reattach_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate reattach outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_reattach_failure(
            "invalid_reply",
            "unexpected Daimon reattach reply",
            None,
            None,
        ),
        Err(ReattachError::ConfirmationMismatch { required }) => daimon_reattach_failure(
            "confirmation_refused",
            "exact Daimon reattach confirmation required",
            Some(&required),
            None,
        ),
        Err(ReattachError::SessionChanged { actual }) => daimon_reattach_failure(
            "stale_session",
            "Daimon session must remain at the exact resume-capable detached revision",
            None,
            Some(&actual),
        ),
        Err(ReattachError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_reattach_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(ReattachError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(ReattachError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_reattach_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(ReattachError::ObservationRefused(refusal))
        | Err(ReattachError::MutationRefused(refusal)) => {
            daimon_reattach_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_reattach_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn daimon_reattach_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::REATTACH_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_prompt(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{PromptError, parse_prompt_document};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let document_path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let prompt_path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let document = read_daimon_bounded(&document_path, 64 * 1024)?;
    let prompt_bytes = read_daimon_bounded(&prompt_path, daimon_agent_command::MAX_PROMPT_BYTES)?;
    let request = parse_prompt_document(&document, prompt_bytes)
        .map_err(|error| format!("Daimon prompt document refused: {error:?}"))?;
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_prompt_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    match daimon_agent_command::prompt(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::PROMPT_CAPABILITY,
                    "complete": true,
                    "status": "dispatched",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_prompt_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate prompt outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_prompt_failure(
            "invalid_reply",
            "unexpected Daimon prompt reply",
            None,
            None,
        ),
        Err(PromptError::ConfirmationMismatch { required }) => daimon_prompt_failure(
            "confirmation_refused",
            "exact Daimon prompt confirmation required",
            Some(&required),
            None,
        ),
        Err(PromptError::SessionChanged { actual }) => daimon_prompt_failure(
            "stale_session",
            "Daimon session revision, phase, or active turn changed",
            None,
            Some(&actual),
        ),
        Err(PromptError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_prompt_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(PromptError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(PromptError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_prompt_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(PromptError::ObservationRefused(refusal))
        | Err(PromptError::MutationRefused(refusal)) => {
            daimon_prompt_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_prompt_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn read_daimon_bounded(
    path: &std::path::Path,
    limit: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Read;

    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(limit)?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds {limit} bytes", path.display()).into());
    }
    Ok(bytes)
}

fn daimon_prompt_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::PROMPT_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn daimon_agent_permission(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_agent_command::{PermissionError, parse_permission_document};
    use daimon_client::unix::AgentClientErrorV1;
    use daimon_protocol::agent::AgentOutcomeV1;
    use std::time::Duration;

    let socket = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let document_path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let confirmation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let document = read_daimon_bounded(&document_path, 64 * 1024)?;
    let request = parse_permission_document(&document)
        .map_err(|error| format!("Daimon permission document refused: {error:?}"))?;
    let (service_session, port) = match daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    ) {
        Ok(connection) => connection,
        Err(error) => {
            return daimon_permission_failure("not_sent", &format!("{error:?}"), None, None);
        }
    };
    match daimon_agent_command::decide_permission(service_session, port, request, &confirmation) {
        Ok(AgentOutcomeV1::Session(view)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "version": 1,
                    "shell": "workbench",
                    "peer": "daimon",
                    "capability": daimon_agent_command::PERMISSION_CAPABILITY,
                    "complete": true,
                    "status": "decided",
                    "revision": view.revision,
                    "phase": format!("{:?}", view.phase),
                    "active_turn": view.active_turn.map(|_| "present"),
                    "last_event_cursor": view.last_event_cursor,
                }))?
            );
            Ok(())
        }
        Ok(AgentOutcomeV1::OutcomeUnknown { .. }) => daimon_permission_failure(
            "outcome_unknown",
            "Daimon retained an indeterminate permission outcome; inspect before any further action",
            None,
            None,
        ),
        Ok(AgentOutcomeV1::Events(_)) => daimon_permission_failure(
            "invalid_reply",
            "unexpected Daimon permission reply",
            None,
            None,
        ),
        Err(PermissionError::ConfirmationMismatch { required }) => daimon_permission_failure(
            "confirmation_refused",
            "exact Daimon permission confirmation required",
            Some(&required),
            None,
        ),
        Err(PermissionError::SessionChanged { actual }) => daimon_permission_failure(
            "stale_session",
            "Daimon session revision, phase, or active turn changed",
            None,
            Some(&actual),
        ),
        Err(PermissionError::ChallengeChanged) => daimon_permission_failure(
            "stale_challenge",
            "Daimon permission cursor, request, turn, or option changed",
            None,
            None,
        ),
        Err(PermissionError::MutationTransport(AgentClientErrorV1::OutcomeUnknown(error))) => {
            daimon_permission_failure("outcome_unknown", &format!("{error:?}"), None, None)
        }
        Err(PermissionError::ObservationTransport(AgentClientErrorV1::NotSent(error)))
        | Err(PermissionError::MutationTransport(AgentClientErrorV1::NotSent(error))) => {
            daimon_permission_failure("not_sent", &format!("{error:?}"), None, None)
        }
        Err(PermissionError::ObservationRefused(refusal))
        | Err(PermissionError::MutationRefused(refusal)) => {
            daimon_permission_failure("refused", &format!("{refusal:?}"), None, None)
        }
        Err(error) => daimon_permission_failure("unavailable", &format!("{error:?}"), None, None),
    }
}

fn daimon_permission_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::PERMISSION_CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn parse_daimon_u64(value: Option<std::ffi::OsString>) -> Result<u64, Box<dyn std::error::Error>> {
    value
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| usage().into())
}

fn daimon_advance_failure(
    status: &str,
    message: &str,
    required_confirmation: Option<&str>,
    actual: Option<&daimon_protocol::agent::AgentSessionViewV1>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "shell": "workbench",
            "peer": "daimon",
            "capability": daimon_agent_command::CAPABILITY,
            "complete": false,
            "status": status,
            "required_confirmation": required_confirmation,
            "actual_revision": actual.map(|view| view.revision),
            "actual_phase": actual.map(|view| format!("{:?}", view.phase)),
            "actual_active_turn": actual.and_then(|view| view.active_turn).map(|_| "present"),
        }))?
    );
    Err(std::io::Error::other(message).into())
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
    let port = match hermes_local_client::LocalPort::connect(
        socket,
        hermes_local_client::ClientLimits::default(),
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
        hermes_local_client::CONNECTION_SESSION,
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
            hermes_local_client::ClientError::OutcomeUnknown(error),
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
    let port = match hermes_local_client::LocalPort::connect(
        socket,
        hermes_local_client::ClientLimits::default(),
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
        hermes_local_client::CONNECTION_SESSION,
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
            hermes_local_client::ClientError::OutcomeUnknown(error),
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
    let port = match hermes_local_client::LocalPort::connect(
        socket,
        hermes_local_client::ClientLimits::default(),
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
        hermes_local_client::CONNECTION_SESSION,
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
            hermes_local_client::ClientError::OutcomeUnknown(error),
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
    let (session, port) = match aegis_local_client::connect(
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
            aegis_local_client::ClientErrorV1::OutcomeUnknown(error),
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
    let (session, port) = match aegis_local_client::connect(
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
            error: aegis_local_client::ClientErrorV1::OutcomeUnknown(error),
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
    let (session, port) = aegis_local_client::connect(
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

fn workbench_machine_configuration_failure(
    mode: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "workspace": serde_json::Value::Null,
            "mode": mode,
            "complete": false,
            "status": "configuration_refused",
            "peers": [],
            "hosted_shutdown": serde_json::Value::Null,
        }))?
    );
    Err(std::io::Error::other(message).into())
}

fn workbench_machine_startup_failure(
    mode: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "workspace": serde_json::Value::Null,
            "mode": mode,
            "complete": false,
            "status": "startup_refused",
            "detail": message,
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
    let operation_id = observation.operation_id;
    let management = LocalManagementSnapshot::after_shutdown(
        operation_id.clone(),
        observation.management,
        shutdown
            .as_ref()
            .map(|shutdown| shutdown.as_ref())
            .map_err(|_| ()),
    );
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
                        HostedPeerId::Tellus => ("tellus", None),
                        HostedPeerId::Chronos(instance) => ("chronos", Some(instance)),
                        HostedPeerId::ChronosAlignment(instance) => {
                            ("chronos-alignment", Some(instance))
                        }
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
            "operation_id": operation_id,
            "mode": "headless",
            "complete": complete,
            "peers": observation.peers,
            "provider_configuration": observation.provider_configuration,
            "operator_agenda": observation.operator_agenda,
            "management": management,
            "not_exercised_profile_requirements": observation.not_exercised_profile_requirements,
            "requirement_boundary": "These configured workspace capabilities were not exercised by the finite aggregate observation. Their presence is not a control receipt, and their omission is not a claim of headless control parity.",
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
        && options.chronos_product_alignment.is_none()
        && options.agora_host.is_none()
        && options.moira_host.is_none()
        && options.daimon_host.is_none()
        && options.hermes_host.is_none()
        && options.aegis_host.is_none()
    {
        return Ok(None);
    }
    if options.host_chronos && options.chronos_streams.is_empty() {
        return Err("--host-chronos requires at least one Chronos workstream".into());
    }
    if options.chronos_trusted_worker.is_some() && !options.chronos_writes {
        return Err("hosted Chronos trusted worker requires --chronos-allow-writes".into());
    }
    let mut host = NativeHost::new();
    for workstream in &options.chronos_streams {
        if !options.host_chronos {
            break;
        }
        let host_options = ChronosHostOptions {
            allow_writes: options.chronos_writes,
            atomic_state: options.chronos_watch
                || options.chronos_cancel_receipts
                || options.chronos_trusted_worker.is_some(),
            observer_history: options.chronos_watch,
            allocation_receipts: false,
        };
        if let Some(worker) = options
            .chronos_trusted_worker
            .as_ref()
            .filter(|worker| worker.workstream == workstream.name)
        {
            host.start_managed_chronos_worker(
                workstream.name.clone(),
                &workstream.directory,
                host_options,
                ChronosTrustedWorkerOptions {
                    agent: worker.agent.clone(),
                    grant_validity: std::time::Duration::from_secs(worker.grant_validity_seconds),
                    lease_duration: std::time::Duration::from_secs(worker.lease_duration_seconds),
                },
            )
            .map_err(|failure| {
                format!("hosted Chronos trusted-worker startup refused: {failure:?}")
            })?;
        } else {
            host.start_chronos(workstream.name.clone(), &workstream.directory, host_options)
                .map_err(|failure| format!("hosted Chronos startup refused: {failure:?}"))?;
        }
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
            options.vellum_allow_add_file,
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
    if let Some(moira) = &options.moira_host {
        host.start_moira_process(moira.clone())
            .map_err(|failure| format!("hosted Moira startup refused: {failure:?}"))?;
    }
    if let Some(daimon) = &options.daimon_host {
        host.start_daimon_process(daimon.clone())
            .map_err(|failure| format!("hosted Daimon startup refused: {failure:?}"))?;
    }
    if let Some(agora) = &options.agora_host {
        host.start_agora_process(agora.clone())
            .map_err(|failure| format!("hosted Agora startup refused: {failure:?}"))?;
    }
    if let Some(product_alignment) = &options.chronos_product_alignment {
        let listener_directory = options
            .alignment_bindings
            .iter()
            .find(|binding| binding.workstream == product_alignment.workstream)
            .expect("workspace profile validates product-alignment binding")
            .directory
            .clone();
        host.start_chronos_product_alignment_process(ChronosProductAlignmentProcessOptions {
            workstream: product_alignment.workstream.clone(),
            executable: product_alignment.executable.clone(),
            config: product_alignment.config.clone(),
            listener_directory,
        })
        .map_err(|failure| {
            format!("hosted Chronos product-alignment startup refused: {failure:?}")
        })?;
    }
    host.seal()
        .map_err(|error| format!("hosted topology seal refused: {error:?}"))?;
    Ok(Some(host))
}

fn interactive(
    options: Options,
    native_host: &mut Option<NativeHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_profile = options.workspace_profile.clone();
    let agora_control = options.agora.as_ref().map(|profile| AgoraControlConfig {
        socket: profile.socket.clone(),
        participant: profile.participant.clone(),
        transport: profile.transport,
    });
    let agora = if let Some(profile) = options.agora {
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
        let (client_id, port) =
            connection.map_err(|error| format!("Agora service refused: {error:?}"))?;
        Some(agora_tui::Component::new(AgoraClientV2::new(
            client_id, port,
        )))
    } else {
        None
    };
    let moira_control_socket = options.moira_socket.clone();
    let moira = if let Some(socket) = options.moira_socket {
        let preflight = moira_client::unix::SagaUnixPort::connect(
            &socket,
            moira_client::unix::SagaClientLimitsV1::default(),
        )
        .map_err(|error| format!("Moira service refused: {error:?}"))?;
        drop(preflight);
        Some(moira_tui::Component::new_with_connector(move || {
            moira_client::unix::SagaUnixPort::connect(
                &socket,
                moira_client::unix::SagaClientLimitsV1::default(),
            )
        }))
    } else {
        None
    };
    let moira_participation_control = options
        .moira_participation_socket
        .clone()
        .zip(options.moira_participation);
    let moira_participation = match (
        options.moira_participation_socket,
        options.moira_participation,
    ) {
        (Some(socket), Some(participation)) => {
            let port = moira_client::unix::AgentParticipationUnixPort::connect(
                socket,
                moira_client::unix::ParticipationClientLimitsV1::default(),
            )
            .map_err(|error| format!("Moira participation service refused: {error:?}"))?;
            Some(moira_tui::ParticipationComponent::new(port, participation))
        }
        (None, None) => None,
        _ => unreachable!("workspace profile validates the participation pair"),
    };
    let moira_deliberation = match (
        options.moira_deliberation_socket,
        options.moira_deliberation,
    ) {
        (Some(socket), Some(deliberation)) => {
            let port = moira_tui_local_client::LocalDeliberationPort::connect(
                &socket,
                moira_tui_local_client::ClientLimits::default(),
            )
            .map_err(|error| format!("Moira deliberation service refused: {error:?}"))?;
            Some(moira_tui::DeliberationComponent::new(port, deliberation))
        }
        (None, None) => None,
        _ => unreachable!("workspace profile validates the deliberation pair"),
    };
    let moira_decision_packet = if let Some(reader) = options
        .workspace_profile
        .as_ref()
        .and_then(|profile| profile.moira.as_ref())
        .and_then(|moira| moira.decision_packet_reader.as_ref())
    {
        let socket = options
            .vellum_socket
            .clone()
            .expect("workspace profile validates packet-reader Vellum configuration");
        let credential = options.vellum_credential.clone();
        let workspace = vellum_protocol::WorkspaceIdV1::new(reader.vellum_workspace)
            .ok_or("Vellum packet-reader workspace identity refused")?;
        let document =
            vellum_protocol::WorkspaceDocumentIdV1::new(workspace, reader.vellum_document)
                .ok_or("Vellum packet-reader document identity refused")?;
        let target = moira_vellum::ExternalDocumentRefV1 {
            workspace: parse_fixed_hex(&reader.owner_route)?,
            document: parse_fixed_hex(&reader.artifact_route)?,
        };
        let mut routes = BTreeMap::new();
        routes.insert(
            target,
            moira_vellum::DocumentRouteV1 {
                workspace,
                document,
            },
        );
        Some(MoiraDecisionPacketComponent::new_with_factory(
            move || {
                let connector = vellum_connector(socket.clone(), credential.as_ref())
                    .map_err(|error| format!("Vellum packet-reader connector refused: {error}"))?;
                RemoteTransport::new(connector, TransportLimits::reference())
                    .map_err(|error| format!("Vellum packet-reader transport refused: {error:?}"))
            },
            routes,
            reader.max_packet_bytes,
        ))
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
    let daimon_model = if let Some(socket) = options.daimon_model_socket {
        let (_session, port) = daimon_client::unix::connect_model(
            socket,
            daimon_wire::LimitsV1::default(),
            daimon_wire::ModelLimitsV1::default(),
            std::time::Duration::from_secs(5),
        )
        .map_err(|error| format!("Daimon model service refused: {error:?}"))?;
        drop(port);
        Some(DaimonModelView::attached())
    } else {
        None
    };
    let daimon_agent_control_socket = options.daimon_agent_socket.clone();
    let daimon_agent = if let Some(socket) = options.daimon_agent_socket {
        let connector = move || {
            daimon_client::unix::connect_agent(
                &socket,
                daimon_wire::LimitsV1::default(),
                daimon_wire::agent::AgentWireLimitsV1::default(),
                std::time::Duration::from_secs(5),
            )
        };
        if options.daimon_agent_sessions.is_empty() {
            Some(daimon_tui::AgentSetComponent::new_empty_with_connector(
                connector,
            ))
        } else {
            Some(daimon_tui::AgentSetComponent::new_with_connector(
                connector,
                options.daimon_agent_sessions,
            )?)
        }
    } else {
        None
    };
    let hermes_operator_socket = options
        .hermes_stream
        .is_some()
        .then(|| options.hermes_socket.clone())
        .flatten();
    let hermes_disclosure_socket = options
        .hermes_stream
        .is_some()
        .then(|| options.hermes_socket.clone())
        .flatten();
    let hermes_control_socket = options
        .hermes_consumer
        .is_some()
        .then(|| options.hermes_socket.clone())
        .flatten();
    let hermes = if let Some(socket) = options.hermes_socket {
        let port = hermes_local_client::LocalPort::connect(
            socket,
            hermes_local_client::ClientLimits::default(),
        )
        .map_err(|error| format!("Hermes service refused: {error:?}"))?;
        Some(hermes_tui::Component::new_with_selection(
            hermes_local_client::CONNECTION_SESSION,
            port,
            options.hermes_stream,
            options.hermes_consumer,
        ))
    } else {
        None
    };
    let aegis_control_socket = options.aegis_socket.clone();
    let aegis = if let Some(socket) = options.aegis_socket {
        let (session, port) = aegis_local_client::connect(
            socket,
            aegis_wire::LimitsV1::default(),
            std::time::Duration::from_secs(5),
        )
        .map_err(|error| format!("Aegis service refused: {error:?}"))?;
        Some(aegis_tui::Component::new(session, port))
    } else {
        None
    };
    let tellus = if let Some(endpoint) = options.tellus_endpoint {
        let token = options
            .tellus_token
            .as_ref()
            .map(read_tellus_token)
            .transpose()?;
        let port =
            tellus_local_client::LocalPort::new(endpoint, token, std::time::Duration::from_secs(5))
                .map_err(|error| format!("Tellus service refused: {error:?}"))?;
        Some(tellus::Component::new(port))
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
        for binding in options.alignment_bindings {
            hub.bind_alignment(&binding.workstream, binding.directory, binding.scope)?;
        }
    }
    let vellum_control = options.vellum_credential.as_ref().and_then(|credential| {
        options
            .vellum_socket
            .as_ref()
            .map(|socket| pantheon_cli::VellumControlConfig {
                socket: socket.clone(),
                credential: Some(credential.clone()),
            })
    });
    let moira_human_gate_response_config = options
        .workspace_profile
        .as_ref()
        .and_then(|profile| profile.moira.as_ref())
        .and_then(|moira| {
            moira
                .human_gate_response
                .as_ref()
                .map(|response| MoiraHumanGateResponseConfig {
                    deliberation_socket: moira
                        .deliberation_socket
                        .clone()
                        .expect("workspace profile validates response deliberation"),
                    vellum: pantheon_cli::VellumControlConfig {
                        socket: options
                            .vellum_socket
                            .clone()
                            .expect("workspace profile validates response Vellum"),
                        credential: options.vellum_credential.clone(),
                    },
                    rationale: response.clone(),
                })
        });
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
    let moira_agent_worker =
        options
            .moira_agent_worker
            .map(|worker| pantheon_cli::MoiraAgentWorkerConfig {
                executable: worker.executable,
                config: worker.config,
                participation: options
                    .moira_participation
                    .expect("workspace profile validates worker participation"),
            });
    let outcome = run_terminal_with_all_peers_managed_configuration(
        options.shell,
        chronos,
        vellum,
        OperationalPeers {
            agora,
            agora_control,
            moira,
            moira_participation,
            moira_deliberation,
            moira_decision_packet,
            moira_human_gate_response_config,
            moira_participation_control,
            moira_agent_worker,
            moira_control_socket,
            daimon,
            daimon_model,
            daimon_agent,
            daimon_agent_control_socket,
            daimon_agent_open: options.daimon_agent_open,
            daimon_execution_policy: options.daimon_execution_policy,
            hermes,
            hermes_operator_socket,
            hermes_disclosure_socket,
            hermes_control_socket,
            aegis,
            aegis_control_socket,
            tellus,
            vellum_control,
            vellum_allow_add_file: options.vellum_allow_add_file,
        },
        native_host.take(),
        current_profile,
    )?;
    finish_interactive(Ok(()), Ok(outcome.shutdown))?;
    if let Some(request) = outcome.restart_profile {
        restart_current_process_with_profile(&request.path, &request.sha256)?;
    }
    Ok(())
}

fn restart_current_process_with_profile(
    path: &std::path::Path,
    sha256: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let mut arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let profile = arguments
        .iter()
        .position(|argument| argument == "--workspace-profile")
        .and_then(|index| arguments.get_mut(index + 1))
        .ok_or("workspace-profile restart lost the original profile argument")?;
    *profile = path.as_os_str().to_owned();
    let error = Command::new(executable)
        .args(arguments)
        .env("PANTHEON_EXPECT_PROFILE_SHA256", sha256)
        .exec();
    Err(error.into())
}

fn headless(options: Options) -> Result<HeadlessObservation, Box<dyn std::error::Error>> {
    let operation_id = options
        .operation_id
        .clone()
        .unwrap_or_else(generated_operation_id);
    observe_headless(headless_request(&options, operation_id)).map_err(|error| error.into())
}

fn management_session(
    options: Options,
    native_host: Option<NativeHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut native_host = ManagementHostGuard(native_host);
    let session_id = options
        .operation_id
        .clone()
        .unwrap_or_else(generated_operation_id);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    write_json_line(
        &mut output,
        &serde_json::json!({
            "version": 1,
            "mode": "management",
            "event": "ready",
            "session_id": session_id,
            "scope": "single_user_local_machine",
            "request_flow": "one_in_flight",
            "commands": [
                "status",
                "observe [OPERATION_ID]",
                "reconfigure OPERATION_ID NEXT_PROFILE",
                "drain"
            ],
            "topology": runtime_status_value(native_host.0.as_ref()),
        }),
    )?;

    loop {
        let Some(line) = read_bounded_management_line(&mut input)? else {
            return drain_management_session(&mut output, &session_id, &mut native_host.0);
        };
        let line = line.trim();
        if line == "status" {
            write_json_line(
                &mut output,
                &serde_json::json!({
                    "version": 1,
                    "mode": "management",
                    "event": "status",
                    "session_id": session_id,
                    "topology": runtime_status_value(native_host.0.as_ref()),
                }),
            )?;
        } else if line == "observe" || line.starts_with("observe ") {
            let requested = line.strip_prefix("observe ").unwrap_or(&session_id);
            if !valid_operation_id(requested) {
                write_management_refusal(&mut output, &session_id, "operation_id_refused")?;
                continue;
            }
            match observe_headless(headless_request(&options, requested.to_owned())) {
                Ok(observation) => {
                    let lifecycle = native_host.0.as_ref().map(|host| host.status().lifecycle);
                    let management = LocalManagementSnapshot::live(
                        observation.operation_id.clone(),
                        observation.management,
                        lifecycle,
                    );
                    write_json_line(
                        &mut output,
                        &serde_json::json!({
                            "version": 1,
                            "mode": "management",
                            "event": "observation",
                            "session_id": session_id,
                            "operation_id": observation.operation_id,
                            "workspace": observation.workspace,
                            "complete": observation.complete,
                            "peers": observation.peers,
                            "management": management,
                        }),
                    )?;
                }
                Err(error) => write_json_line(
                    &mut output,
                    &serde_json::json!({
                        "version": 1,
                        "mode": "management",
                        "event": "observation",
                        "session_id": session_id,
                        "operation_id": requested,
                        "complete": false,
                        "status": "refused",
                        "detail": error,
                    }),
                )?,
            }
        } else if line == "reconfigure" || line.starts_with("reconfigure ") {
            reconfigure_management_session(
                &mut output,
                &session_id,
                line,
                &options,
                &mut native_host.0,
            )?;
        } else if line == "drain" {
            return drain_management_session(&mut output, &session_id, &mut native_host.0);
        } else {
            write_management_refusal(&mut output, &session_id, "unknown_command")?;
        }
    }
}

struct ManagementHostGuard(Option<NativeHost>);

impl Drop for ManagementHostGuard {
    fn drop(&mut self) {
        let _ = shutdown_native_host(&mut self.0);
    }
}

fn read_bounded_management_line(
    input: &mut impl BufRead,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    const MAX_COMMAND_BYTES: usize = 4 * 1_024;
    let mut bytes = Vec::new();
    loop {
        let available = input.fill_buf()?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            break;
        }
        let end = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if bytes.len() + end > MAX_COMMAND_BYTES {
            return Err("management command exceeds 4 KiB".into());
        }
        bytes.extend_from_slice(&available[..end]);
        input.consume(end);
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            break;
        }
    }
    Ok(Some(String::from_utf8(bytes)?))
}

fn write_json_line(
    output: &mut impl Write,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(())
}

fn write_management_refusal(
    output: &mut impl Write,
    session_id: &str,
    status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_line(
        output,
        &serde_json::json!({
            "version": 1,
            "mode": "management",
            "event": "refusal",
            "session_id": session_id,
            "status": status,
        }),
    )
}

fn reconfigure_management_session(
    output: &mut impl Write,
    session_id: &str,
    command: &str,
    options: &Options,
    native_host: &mut Option<NativeHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(arguments) = command.strip_prefix("reconfigure ") else {
        return write_management_refusal(output, session_id, "reconfiguration_syntax_refused");
    };
    let Some((operation_id, path)) = arguments.split_once(' ') else {
        return write_management_refusal(output, session_id, "reconfiguration_syntax_refused");
    };
    let path = path.trim();
    if !valid_operation_id(operation_id) || path.is_empty() {
        return write_management_refusal(output, session_id, "reconfiguration_syntax_refused");
    }
    let next_path = PathBuf::from(path);
    let next = match preflight_management_profile(&next_path) {
        Ok(profile) => profile,
        Err(detail) => {
            return write_json_line(
                output,
                &serde_json::json!({
                    "version": 1,
                    "mode": "management",
                    "event": "refusal",
                    "session_id": session_id,
                    "operation_id": operation_id,
                    "status": "replacement_profile_refused",
                    "detail": detail,
                }),
            );
        }
    };
    let current = options
        .workspace_profile
        .as_ref()
        .expect("management requires a workspace profile");
    let plan = pantheon_cli::reconfiguration::plan(operation_id.to_owned(), current, &next);
    if plan.minimum_action == "none" {
        return write_json_line(
            output,
            &serde_json::json!({
                "version": 1,
                "mode": "management",
                "event": "reconfigure",
                "session_id": session_id,
                "operation_id": operation_id,
                "complete": true,
                "status": "no_change",
                "plan": plan,
                "topology": runtime_status_value(native_host.as_ref()),
            }),
        );
    }

    let shutdown = shutdown_native_host(native_host);
    let (clean, shutdown) = management_shutdown_report(shutdown);
    if !clean {
        write_json_line(
            output,
            &serde_json::json!({
                "version": 1,
                "mode": "management",
                "event": "reconfigure",
                "session_id": session_id,
                "operation_id": operation_id,
                "complete": false,
                "status": "restart_fenced",
                "plan": plan,
                "shutdown": shutdown,
            }),
        )?;
        return Err("management reconfiguration fenced by incomplete shutdown".into());
    }
    write_json_line(
        output,
        &serde_json::json!({
            "version": 1,
            "mode": "management",
            "event": "reconfigure",
            "session_id": session_id,
            "operation_id": operation_id,
            "complete": false,
            "status": "old_topology_drained",
            "plan": plan,
            "shutdown": shutdown,
            "replacement": "exec_pending",
        }),
    )?;

    let error = Command::new(std::env::current_exe()?)
        .arg("manage")
        .arg("--workspace-profile")
        .arg(next_path)
        .arg("--operation-id")
        .arg(operation_id)
        .exec();
    write_json_line(
        output,
        &serde_json::json!({
            "version": 1,
            "mode": "management",
            "event": "reconfigure",
            "session_id": session_id,
            "operation_id": operation_id,
            "complete": false,
            "status": "replacement_exec_refused",
            "detail": error.to_string(),
        }),
    )?;
    Err(error.into())
}

fn preflight_management_profile(path: &std::path::Path) -> Result<WorkspaceProfile, String> {
    let profile = WorkspaceProfile::load(path).map_err(|error| error.to_string())?;
    let output = Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
        .arg("manage")
        .arg("--workspace-profile")
        .arg(path)
        .arg("--check-profile")
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(profile)
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if detail.is_empty() {
            "replacement profile preflight refused".to_owned()
        } else {
            detail
        })
    }
}

fn runtime_status_value(host: Option<&NativeHost>) -> serde_json::Value {
    let Some(status) = host.map(NativeHost::status) else {
        return serde_json::json!({
            "lifecycle": "external",
            "hosted_peers": [],
            "trusted_chronos_worker": null,
            "reconfiguration_allowed": false,
            "restart_receipts": [],
        });
    };
    let trusted_worker = host.and_then(NativeHost::managed_chronos_worker_status);
    serde_json::json!({
        "lifecycle": registry_lifecycle_label(status.lifecycle),
        "hosted_peers": status.hosted_peers.iter().map(hosted_peer_value).collect::<Vec<_>>(),
        "trusted_chronos_worker": trusted_worker.map(|worker| serde_json::json!({
            "workstream": worker.workstream,
            "state": worker.state,
            "authority_boundary": "private in-process capability; no grant, lease, task key, or result is projected",
        })),
        "reconfiguration_allowed": status.reconfiguration_allowed,
        "restart_receipts": status.restart_receipts.iter().map(|receipt| {
            let (peer, instance) = hosted_peer_identity(&receipt.peer);
            serde_json::json!({
                "peer": peer,
                "instance": instance,
                "stopped": shutdown_disposition_label(receipt.stopped),
                "replacement_started": receipt.replacement_started,
            })
        }).collect::<Vec<_>>(),
    })
}

fn hosted_peer_value(peer: &HostedPeerId) -> serde_json::Value {
    let (peer, instance) = hosted_peer_identity(peer);
    serde_json::json!({"peer":peer, "instance":instance})
}

fn hosted_peer_identity(peer: &HostedPeerId) -> (&'static str, Option<&str>) {
    match peer {
        HostedPeerId::Agora => ("agora", None),
        HostedPeerId::Moira => ("moira", None),
        HostedPeerId::Daimon => ("daimon", None),
        HostedPeerId::Hermes => ("hermes", None),
        HostedPeerId::Aegis => ("aegis", None),
        HostedPeerId::Tellus => ("tellus", None),
        HostedPeerId::Chronos(instance) => ("chronos", Some(instance)),
        HostedPeerId::ChronosAlignment(instance) => ("chronos-alignment", Some(instance)),
        HostedPeerId::Vellum => ("vellum", None),
    }
}

const fn registry_lifecycle_label(lifecycle: RegistryLifecycle) -> &'static str {
    match lifecycle {
        RegistryLifecycle::Assembling => "assembling",
        RegistryLifecycle::Ready => "ready",
        RegistryLifecycle::Degraded => "degraded",
        RegistryLifecycle::Draining => "draining",
        RegistryLifecycle::Stopped => "stopped",
    }
}

const fn shutdown_disposition_label(disposition: ShutdownDisposition) -> &'static str {
    match disposition {
        ShutdownDisposition::Clean => "clean",
        ShutdownDisposition::Refused => "refused",
        ShutdownDisposition::OutcomeUnknown => "outcome_unknown",
    }
}

fn drain_management_session(
    output: &mut impl Write,
    session_id: &str,
    native_host: &mut Option<NativeHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (complete, report) = management_shutdown_report(shutdown_native_host(native_host));
    write_json_line(
        output,
        &serde_json::json!({
            "version": 1,
            "mode": "management",
            "event": "drain",
            "session_id": session_id,
            "complete": complete,
            "shutdown": report,
        }),
    )?;
    if complete {
        Ok(())
    } else {
        Err("management drain incomplete".into())
    }
}

fn management_shutdown_report(
    shutdown: Result<Option<TopologyShutdown>, String>,
) -> (bool, serde_json::Value) {
    match shutdown {
        Ok(None) => (true, serde_json::Value::Null),
        Ok(Some(receipt)) => {
            let complete = receipt.clean();
            let receipts = receipt
                .receipts
                .iter()
                .map(|receipt| {
                    let (peer, instance) = hosted_peer_identity(&receipt.peer);
                    serde_json::json!({
                        "peer": peer,
                        "instance": instance,
                        "disposition": shutdown_disposition_label(receipt.disposition),
                    })
                })
                .collect::<Vec<_>>();
            (complete, serde_json::json!({"receipts":receipts}))
        }
        Err(error) => (false, serde_json::json!({"detail":error})),
    }
}

fn headless_request(options: &Options, operation_id: String) -> HeadlessRequest {
    HeadlessRequest {
        operation_id,
        workspace: options.workspace_name.clone(),
        agora: options.agora.clone(),
        agora_hosted: options.agora_host.is_some(),
        task_conversation: options
            .workspace_profile
            .as_ref()
            .and_then(|profile| profile.task_conversation.clone()),
        moira_socket: options.moira_socket.clone(),
        moira_participation_socket: options.moira_participation_socket.clone(),
        moira_participation: options.moira_participation,
        moira_deliberation_socket: options.moira_deliberation_socket.clone(),
        moira_deliberation: options.moira_deliberation,
        moira_decision_packet_reader: options
            .workspace_profile
            .as_ref()
            .and_then(|profile| profile.moira.as_ref())
            .and_then(|moira| moira.decision_packet_reader.clone()),
        moira_hosted: options.moira_host.is_some(),
        daimon_socket: options.daimon_socket.clone(),
        daimon_model_socket: options.daimon_model_socket.clone(),
        daimon_agent_socket: options.daimon_agent_socket.clone(),
        daimon_agent_session: options.daimon_agent_session,
        daimon_agent_open: options.daimon_agent_open.clone(),
        daimon_execution_policy: options.daimon_execution_policy.clone(),
        daimon_provider_owner: options
            .workspace_profile
            .as_ref()
            .and_then(|profile| profile.daimon.as_ref())
            .and_then(|daimon| daimon.host.clone()),
        daimon_hosted: options.daimon_host.is_some(),
        hestia: options
            .workspace_profile
            .as_ref()
            .and_then(|profile| profile.hestia.clone()),
        hermes_socket: options.hermes_socket.clone(),
        hermes_stream: options.hermes_stream,
        hermes_consumer: options.hermes_consumer,
        hermes_hosted: options.hermes_host.is_some(),
        aegis_socket: options.aegis_socket.clone(),
        aegis_hosted: options.aegis_host.is_some(),
        tellus_endpoint: options.tellus_endpoint,
        tellus_token: options.tellus_token.clone(),
        chronos: options
            .chronos_streams
            .iter()
            .map(|stream| Workstream {
                name: stream.name.clone(),
                directory: stream.directory.clone(),
            })
            .collect(),
        chronos_alignment: options.alignment_bindings.clone(),
        chronos_hosted: options.host_chronos,
        vellum_socket: options.vellum_socket.clone(),
        vellum_credential: options.vellum_credential.clone(),
        vellum_hosted: options.host_vellum,
        not_exercised_profile_requirements: options.not_exercised_profile_requirements.clone(),
    }
}

fn read_tellus_token(path: &std::path::PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    if std::fs::metadata(path)?.len() > 8 * 1024 {
        return Err("Tellus token exceeds 8 KiB".into());
    }
    let token = std::fs::read_to_string(path)?;
    let token = token.trim_end().to_owned();
    if token.is_empty() {
        return Err("Tellus token is empty".into());
    }
    Ok(token)
}
