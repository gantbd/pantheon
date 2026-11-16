//! Native peer-service ownership for one Workbench process.
//!
//! This module owns only operational lifecycle. Each peer keeps its own
//! service, protocol, state root, authority configuration, and concrete client.

#[cfg(feature = "native-chronos")]
use std::os::unix::net::UnixStream;
#[cfg(target_os = "macos")]
use std::sync::mpsc;
#[cfg(any(feature = "native-chronos", target_os = "macos"))]
use std::thread::JoinHandle;
#[cfg(unix)]
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};
use std::{path::Path, thread};

#[cfg(feature = "native-chronos")]
use chronos_standalone::Server;
#[cfg(target_os = "macos")]
use vellum_adapter_local_authentication::{LocalCredentialStoreV1, LocalCredentialVerifierV1};
#[cfg(target_os = "macos")]
use vellum_adapter_local_fs::{FilesystemLimits, LocalFilesystemFactory};
#[cfg(target_os = "macos")]
use vellum_daemon::{
    DaemonLimits, DaemonServeError, DrainCompletion, DrainTransition, LocalDaemon,
    compose_startup_workspace, request_shutdown,
};
#[cfg(target_os = "macos")]
use vellum_host::authority::{AuthorityActionSetV1, AuthorityActionV1};

use crate::registry::{
    HostRegistry, HostedPeerId, RegistryError, ShutdownDisposition, StartupFailure,
    TopologyShutdown,
};

pub struct NativeHost {
    registry: HostRegistry,
}

#[cfg(unix)]
type SocketIdentity = (u64, u64, i64, i64);

#[cfg(unix)]
fn socket_identity(socket: &Path) -> Option<SocketIdentity> {
    fs::symlink_metadata(socket).ok().and_then(|metadata| {
        metadata.file_type().is_socket().then(|| {
            (
                metadata.dev(),
                metadata.ino(),
                metadata.ctime(),
                metadata.ctime_nsec(),
            )
        })
    })
}

#[cfg(unix)]
fn ready_socket_identity(socket: &Path) -> Option<SocketIdentity> {
    fs::symlink_metadata(socket).ok().and_then(|metadata| {
        (metadata.file_type().is_socket() && metadata.permissions().mode() & 0o077 == 0).then(
            || {
                (
                    metadata.dev(),
                    metadata.ino(),
                    metadata.ctime(),
                    metadata.ctime_nsec(),
                )
            },
        )
    })
}

#[cfg(unix)]
fn wait_for_process_ready(
    child: &mut Child,
    socket: &Path,
    peer: &str,
    prior_socket: Option<SocketIdentity>,
) -> Result<(), String> {
    for _ in 0..400 {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("{peer} readiness check failed: {error}"))?
        {
            return Err(format!("{peer} stopped before readiness: {status}"));
        }
        if ready_socket_identity(socket).is_some_and(|identity| Some(identity) != prior_socket) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(5));
    }
    Err(format!("{peer} readiness deadline expired"))
}

#[cfg(unix)]
fn signal_process(child: &Child) -> Result<(), nix::errno::Errno> {
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(i32::try_from(child.id()).unwrap_or(i32::MAX)),
        nix::sys::signal::Signal::SIGTERM,
    )
}

#[cfg(unix)]
fn stop_failed_process_start(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = signal_process(child);
        for _ in 0..100 {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(unix)]
fn stop_process(child: &mut Child) -> ShutdownDisposition {
    if let Ok(Some(status)) = child.try_wait() {
        return if status.success() {
            ShutdownDisposition::Clean
        } else {
            ShutdownDisposition::Refused
        };
    }
    if signal_process(child).is_err() {
        return ShutdownDisposition::Refused;
    }
    for _ in 0..400 {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    ShutdownDisposition::Clean
                } else {
                    ShutdownDisposition::Refused
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => return ShutdownDisposition::OutcomeUnknown,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    ShutdownDisposition::OutcomeUnknown
}

/// Deployment-neutral options for a Workbench-owned Chronos service.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChronosHostOptions {
    pub allow_writes: bool,
    pub atomic_state: bool,
    pub observer_history: bool,
    pub allocation_receipts: bool,
}

/// Explicit deployment inputs for a Workbench-owned, sovereign Hermes daemon.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HermesProcessOptions {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub socket: PathBuf,
    pub create: bool,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub subject: String,
    pub operator_stream: Option<String>,
}

/// Explicit deployment inputs for Daimon's production interactive-agent
/// application. Provider, credential, authority, attempt, storage and recovery
/// policy remain in the opaque Daimon-owned configuration document.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DaimonProcessOptions {
    pub executable: PathBuf,
    pub config: PathBuf,
    pub socket: PathBuf,
}

/// Opaque deployment inputs for Moira's sovereign durable saga owner.
/// Store, peer mapping, bounds and operator-authority selection remain inside
/// the Moira-owned configuration document.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoiraProcessOptions {
    pub executable: PathBuf,
    pub config: PathBuf,
    pub socket: PathBuf,
}

/// Explicit deployment inputs for a Workbench-owned, sovereign Aegis daemon.
/// These values are passed to Aegis unchanged; Workbench does not interpret
/// the authority namespace, principal, session namespace, or role as grants.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AegisProcessOptions {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub socket: PathBuf,
    pub create: bool,
    pub authority: String,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub principal: String,
    pub session_namespace: String,
    pub administrator: bool,
}

/// Opaque deployment inputs for Agora's sovereign private-local application.
/// The config remains Agora-owned; Workbench uses the socket only for
/// readiness and ordinary public-client attachment.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgoraProcessOptions {
    pub executable: PathBuf,
    pub config: PathBuf,
    pub socket: PathBuf,
}

impl Default for NativeHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeHost {
    pub fn new() -> Self {
        Self {
            registry: HostRegistry::new(),
        }
    }

    /// Binds and starts one Chronos owner. The endpoint is fully bound before
    /// this returns, so callers may construct the ordinary peer client without
    /// a readiness sleep or a second protocol.
    pub fn start_chronos(
        &mut self,
        name: impl Into<String>,
        directory: &Path,
        options: ChronosHostOptions,
    ) -> Result<(), StartupFailure<String>> {
        #[cfg(feature = "native-chronos")]
        {
            let name = name.into();
            self.registry
                .start_or_rollback(HostedPeerId::Chronos(name), || -> Result<_, String> {
                    let mut server = Server::bind_with_options(
                        directory,
                        chronos_standalone::ServerOptions {
                            allow_writes: options.allow_writes,
                            atomic_state: options.atomic_state,
                            observer_history: options.observer_history,
                            allocation_receipts: options.allocation_receipts,
                        },
                    )
                    .map_err(ToOwned::to_owned)?;
                    let (wake, writer) = UnixStream::pair()
                        .map_err(|error| format!("Chronos shutdown channel refused: {error}"))?;
                    let owner = thread::Builder::new()
                        .name("workbench-chronos-owner".into())
                        .spawn(move || server.run_with_shutdown(&wake))
                        .map_err(|error| format!("Chronos owner thread refused: {error}"))?;
                    Ok(((), chronos_shutdown(writer, owner)))
                })
        }
        #[cfg(not(feature = "native-chronos"))]
        {
            let name = name.into();
            let _ = (directory, options);
            self.registry
                .start_or_rollback(HostedPeerId::Chronos(name), || {
                    Err::<((), fn() -> ShutdownDisposition), _>(
                        "native Chronos hosting is not compiled into this deployment".to_owned(),
                    )
                })
        }
    }

    /// Starts an Agora owner supplied by the deployment and returns its
    /// concrete client or presentation input unchanged. Workbench owns only
    /// lifecycle registration; the factory remains responsible for Aegis
    /// authentication, policy, trusted time, Host selection, and durability.
    pub fn start_agora<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Agora, start)
    }

    /// Starts Agora's standalone private-local application unchanged.
    /// Workbench owns only readiness and process drain. Authority, bootstrap,
    /// trusted time, durable storage, and grant selection stay in Agora's
    /// opaque deployment config, and clients attach to the public socket.
    #[cfg(unix)]
    pub fn start_agora_process(
        &mut self,
        options: AgoraProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_agora(|| {
            let prior_socket = socket_identity(&options.socket);
            let mut child = Command::new(&options.executable)
                .arg(&options.config)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Agora process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &options.socket, "Agora", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            let mut input = child.stdin.take();
            let mut output = child.stdout.take();
            Ok(((), move || {
                stop_agora_process(&mut child, &mut input, &mut output)
            }))
        })
    }

    /// Starts a deployment-supplied Moira owner. The deployment retains
    /// responsibility for its durable store, authority adapter and recovery
    /// policy; Workbench retains only the typed client and shutdown receipt.
    pub fn start_moira<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Moira, start)
    }

    /// Starts Moira's production saga application as a sovereign process.
    /// Workbench owns only readiness and the exact one-byte drain channel; it
    /// attaches through Moira's ordinary public saga client.
    #[cfg(unix)]
    pub fn start_moira_process(
        &mut self,
        options: MoiraProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_moira(|| {
            let prior_socket = socket_identity(&options.socket);
            let mut child = Command::new(&options.executable)
                .arg("--supervised")
                .arg(&options.config)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Moira process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &options.socket, "Moira", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            let mut input = child.stdin.take();
            Ok(((), move || stop_supervised_process(&mut child, &mut input)))
        })
    }

    /// Starts a deployment-supplied Daimon owner without acquiring provider,
    /// dispatch, identity or resume policy.
    pub fn start_daimon<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Daimon, start)
    }

    /// Starts Daimon's production local-agent application in its owner-defined
    /// supervised mode. Workbench owns readiness and drain only and attaches
    /// through the ordinary public agent client.
    #[cfg(unix)]
    pub fn start_daimon_process(
        &mut self,
        options: DaimonProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_daimon(|| {
            let prior_socket = socket_identity(&options.socket);
            let mut child = Command::new(&options.executable)
                .arg("--supervised")
                .arg(&options.config)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Daimon process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &options.socket, "Daimon", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            let mut input = child.stdin.take();
            Ok(((), move || stop_supervised_process(&mut child, &mut input)))
        })
    }

    /// Starts a deployment-supplied Hermes owner. Retention, authorization,
    /// compaction and persistence remain inputs to the owning factory.
    pub fn start_hermes<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Hermes, start)
    }

    /// Starts the owner-supplied selected Hermes application as a sovereign
    /// process. Workbench owns readiness and shutdown only; all ledger,
    /// authentication, authorization, persistence and wire behavior remains in
    /// Hermes, and ordinary Hermes clients attach to the configured socket.
    #[cfg(unix)]
    pub fn start_hermes_process(
        &mut self,
        options: HermesProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_hermes(|| {
            let prior_socket = socket_identity(&options.socket);
            let mut command = Command::new(&options.executable);
            command
                .arg(if options.create { "create" } else { "open" })
                .arg(&options.directory)
                .arg(&options.socket)
                .arg(options.effective_uid.to_string())
                .arg(options.effective_gid.to_string())
                .arg(&options.subject);
            match &options.operator_stream {
                Some(stream) => {
                    command.arg("operator").arg(stream);
                }
                None => {
                    command.arg("observer");
                }
            }
            let mut child = command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Hermes process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &options.socket, "Hermes", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            Ok(((), move || stop_process(&mut child)))
        })
    }

    /// Starts a deployment-supplied Aegis owner. Workbench does not construct
    /// grants, trusted time, peer identity or authority policy.
    pub fn start_aegis<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Aegis, start)
    }

    /// Starts the owner-supplied Aegis application as a sovereign process.
    /// Workbench owns readiness and shutdown only; Aegis retains authority,
    /// authentication, role, persistence, session and wire behavior.
    #[cfg(unix)]
    pub fn start_aegis_process(
        &mut self,
        options: AegisProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_aegis(|| {
            let prior_socket = socket_identity(&options.socket);
            let mut child = Command::new(&options.executable)
                .arg(if options.create { "create" } else { "open" })
                .arg(&options.directory)
                .arg(&options.socket)
                .arg(&options.authority)
                .arg(options.effective_uid.to_string())
                .arg(options.effective_gid.to_string())
                .arg(&options.principal)
                .arg(&options.session_namespace)
                .arg(if options.administrator {
                    "administrator"
                } else {
                    "principal"
                })
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Aegis process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &options.socket, "Aegis", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            Ok(((), move || stop_process(&mut child)))
        })
    }

    /// Starts a deployment-supplied Grounding/Dendrite owner. The returned
    /// client remains the typed Grounding presentation boundary; this does not
    /// turn MCP into a Workbench lifecycle or semantic protocol.
    pub fn start_grounding<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry
            .start_or_rollback(HostedPeerId::Grounding, start)
    }

    pub fn seal(&mut self) -> Result<(), RegistryError> {
        self.registry.seal()
    }

    /// Composes and starts one Vellum daemon with an explicit workspace and
    /// finite action ceiling. Credential material remains in its private file;
    /// neither the registry nor its receipts retain or project it.
    #[cfg(target_os = "macos")]
    pub fn start_vellum(
        &mut self,
        socket: &Path,
        roots: &[std::path::PathBuf],
        credential: Option<&Path>,
        allow_edit: bool,
        allow_save: bool,
    ) -> Result<(), StartupFailure<String>> {
        self.registry
            .start_or_rollback(HostedPeerId::Vellum, || -> Result<_, String> {
                let config = VellumOwnerConfig {
                    socket: socket.to_path_buf(),
                    roots: roots.to_vec(),
                    credential: credential.map(Path::to_path_buf),
                    allow_edit,
                    allow_save,
                };
                let (ready, readiness) = mpsc::sync_channel(1);
                let owner = thread::Builder::new()
                    .name("workbench-vellum-owner".into())
                    .spawn(move || run_vellum_owner(config, ready))
                    .map_err(|error| format!("Vellum owner thread refused: {error}"))?;
                let control_path = match readiness.recv() {
                    Ok(Ok(control_path)) => control_path,
                    Ok(Err(error)) => {
                        let _ = owner.join();
                        return Err(error);
                    }
                    Err(_) => {
                        let result = owner.join();
                        return Err(format!("Vellum readiness outcome unavailable: {result:?}"));
                    }
                };
                Ok(((), vellum_shutdown(control_path, owner)))
            })
    }

    #[cfg(not(target_os = "macos"))]
    pub fn start_vellum(
        &mut self,
        _socket: &Path,
        _roots: &[std::path::PathBuf],
        _credential: Option<&Path>,
        _allow_edit: bool,
        _allow_save: bool,
    ) -> Result<(), StartupFailure<String>> {
        self.registry.start_or_rollback(HostedPeerId::Vellum, || {
            Err::<((), fn() -> ShutdownDisposition), _>(
                "native Vellum hosting requires the qualified macOS adapter profile".to_owned(),
            )
        })
    }

    pub fn shutdown(&mut self) -> Result<TopologyShutdown, RegistryError> {
        self.registry.shutdown()
    }
}

#[cfg(unix)]
fn stop_agora_process(
    child: &mut Child,
    input: &mut Option<ChildStdin>,
    output: &mut Option<ChildStdout>,
) -> ShutdownDisposition {
    if child.try_wait().ok().flatten().is_some() {
        return ShutdownDisposition::Refused;
    }
    let Some(mut shutdown) = input.take() else {
        return ShutdownDisposition::Refused;
    };
    if shutdown.write_all(b"x").is_err() {
        return ShutdownDisposition::Refused;
    }
    drop(shutdown);
    for _ in 0..400 {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return ShutdownDisposition::Refused;
                }
                let clean = output
                    .take()
                    .is_some_and(|mut receipt| agora_cleanup_receipt_is_clean(&mut receipt));
                return if clean {
                    ShutdownDisposition::Clean
                } else {
                    ShutdownDisposition::Refused
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => return ShutdownDisposition::OutcomeUnknown,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    ShutdownDisposition::OutcomeUnknown
}

#[cfg(unix)]
fn stop_supervised_process(
    child: &mut Child,
    input: &mut Option<ChildStdin>,
) -> ShutdownDisposition {
    if child.try_wait().ok().flatten().is_some() {
        return ShutdownDisposition::Refused;
    }
    let Some(mut shutdown) = input.take() else {
        return ShutdownDisposition::Refused;
    };
    if shutdown.write_all(b"x").is_err() {
        return ShutdownDisposition::Refused;
    }
    drop(shutdown);
    for _ in 0..400 {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    ShutdownDisposition::Clean
                } else {
                    ShutdownDisposition::Refused
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => return ShutdownDisposition::OutcomeUnknown,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    ShutdownDisposition::OutcomeUnknown
}

#[cfg(unix)]
fn agora_cleanup_receipt_is_clean(output: &mut impl Read) -> bool {
    const MAX_RECEIPT_BYTES: u64 = 4 * 1_024;
    let mut bytes = Vec::new();
    if output
        .take(MAX_RECEIPT_BYTES + 1)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.is_empty()
        || bytes.len() as u64 > MAX_RECEIPT_BYTES
    {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(receipt) = value.as_object() else {
        return false;
    };
    receipt.len() == 6
        && receipt.get("schema").and_then(serde_json::Value::as_u64) == Some(1)
        && receipt
            .get("priorLifecycle")
            .and_then(serde_json::Value::as_str)
            == Some("draining")
        && receipt.get("cleanup").and_then(serde_json::Value::as_str) == Some("clean")
        && [
            "acceptedConnections",
            "completedConnections",
            "refusedConnections",
        ]
        .iter()
        .all(|key| {
            receipt
                .get(*key)
                .and_then(serde_json::Value::as_u64)
                .is_some()
        })
}

#[cfg(target_os = "macos")]
fn vellum_control_path(socket: &Path) -> Result<PathBuf, String> {
    let parent = socket
        .parent()
        .ok_or_else(|| "Vellum socket requires a parent directory".to_owned())?;
    let name = socket
        .file_name()
        .ok_or_else(|| "Vellum socket requires a file name".to_owned())?;
    let mut control = name.to_os_string();
    control.push(".control");
    Ok(parent.join(control))
}

#[cfg(target_os = "macos")]
struct VellumOwnerConfig {
    socket: PathBuf,
    roots: Vec<PathBuf>,
    credential: Option<PathBuf>,
    allow_edit: bool,
    allow_save: bool,
}

#[cfg(target_os = "macos")]
fn run_vellum_owner(
    config: VellumOwnerConfig,
    ready: mpsc::SyncSender<Result<PathBuf, String>>,
) -> Result<(), String> {
    let build = (|| {
        let limits = DaemonLimits::reference();
        let filesystem_limits = FilesystemLimits {
            max_resources: 16,
            max_bytes: 65_536,
            max_stage_attempts: 16,
            max_metadata_bytes: 65_536,
        };
        let workspace = compose_startup_workspace(
            &config.roots,
            vellum_workspace::reference_workspace_profile(
                LocalFilesystemFactory::new(filesystem_limits).erase(),
            ),
            limits,
        )
        .map_err(|error| format!("Vellum workspace refused: {error:?}"))?;
        let mut actions = AuthorityActionSetV1::read_only();
        if config.allow_edit {
            actions = actions.join(AuthorityActionSetV1::singleton(
                AuthorityActionV1::EditSource,
            ));
        }
        if config.allow_save {
            actions = actions.join(AuthorityActionSetV1::singleton(
                AuthorityActionV1::PersistSource,
            ));
        }
        let mut daemon = LocalDaemon::bind_with_persistent_identity_and_action_ceiling(
            &config.socket,
            workspace,
            limits,
            actions,
        )
        .map_err(|error| {
            format!(
                "Vellum daemon binding refused: {:?}",
                error.operational_class()
            )
        })?;
        if let Some(credential) = &config.credential {
            let record = LocalCredentialStoreV1::default()
                .load(credential)
                .map_err(|error| format!("Vellum credential refused: {error:?}"))?;
            daemon = daemon.require_authentication(
                LocalCredentialVerifierV1::new(record),
                (),
                limits.max_authentication_attempts,
            );
        }
        let control_path = vellum_control_path(&config.socket)?;
        let control = daemon
            .bind_shutdown_control(&control_path)
            .map_err(|error| format!("Vellum control binding refused: {error:?}"))?;
        let control_owner = thread::Builder::new()
            .name("workbench-vellum-control".into())
            .spawn(move || control.wait())
            .map_err(|error| format!("Vellum control thread refused: {error}"))?;
        Ok::<_, String>((daemon, control_path, control_owner))
    })();
    let (mut daemon, control_path, control_owner) = match build {
        Ok(parts) => parts,
        Err(error) => {
            let _ = ready.send(Err(error.clone()));
            return Err(error);
        }
    };
    if ready.send(Ok(control_path)).is_err() {
        return Err("Vellum readiness receiver unavailable".into());
    }
    loop {
        match daemon.serve_next() {
            Ok(()) | Err(DaemonServeError::Request(_)) | Err(DaemonServeError::Authentication) => {}
            Err(DaemonServeError::ShutdownRequested) => {
                if daemon.begin_drain() != DrainTransition::Started {
                    return Err("Vellum drain could not start".into());
                }
                if daemon.complete_drain() != DrainCompletion::Complete {
                    return Err("Vellum drain did not quiesce".into());
                }
                return match control_owner.join() {
                    Ok(Ok(())) => Ok(()),
                    result => Err(format!("Vellum control outcome unavailable: {result:?}")),
                };
            }
            Err(error) => return Err(format!("Vellum owner failed: {error:?}")),
        }
    }
}

#[cfg(target_os = "macos")]
fn vellum_shutdown(
    control_path: PathBuf,
    owner: JoinHandle<Result<(), String>>,
) -> impl FnMut() -> ShutdownDisposition + Send {
    let mut owner = Some(owner);
    move || {
        if request_shutdown(&control_path, DaemonLimits::reference().io_timeout).is_err() {
            return ShutdownDisposition::Refused;
        }
        let owner_result = owner.take().map(JoinHandle::join);
        match owner_result {
            Some(Ok(Ok(()))) => ShutdownDisposition::Clean,
            _ => ShutdownDisposition::OutcomeUnknown,
        }
    }
}

#[cfg(feature = "native-chronos")]
fn chronos_shutdown(
    mut writer: UnixStream,
    owner: JoinHandle<Result<(), &'static str>>,
) -> impl FnMut() -> ShutdownDisposition + Send {
    let mut owner = Some(owner);
    move || {
        let sent = writer.write_all(&[1]).is_ok();
        let Some(owner) = owner.take() else {
            return ShutdownDisposition::Refused;
        };
        match (sent, owner.join()) {
            (_, Ok(Ok(()))) => ShutdownDisposition::Clean,
            (true, Ok(Err(_)) | Err(_)) => ShutdownDisposition::OutcomeUnknown,
            (false, Ok(Err(_)) | Err(_)) => ShutdownDisposition::Refused,
        }
    }
}

#[cfg(test)]
mod deployment_factory_tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};

    type TestShutdown = Box<dyn FnMut() -> ShutdownDisposition + Send + 'static>;
    type TestStart<C> = Result<(C, TestShutdown), String>;

    #[cfg(unix)]
    #[test]
    fn agora_cleanup_receipt_requires_exact_bounded_clean_shape() {
        let clean = br#"{"schema":1,"priorLifecycle":"draining","cleanup":"clean","acceptedConnections":2,"completedConnections":2,"refusedConnections":0}
"#;
        assert!(agora_cleanup_receipt_is_clean(&mut Cursor::new(clean)));

        let residue = br#"{"schema":1,"priorLifecycle":"draining","cleanup":"residue-preserved","acceptedConnections":2,"completedConnections":2,"refusedConnections":0}
"#;
        assert!(!agora_cleanup_receipt_is_clean(&mut Cursor::new(residue)));
        let extra = br#"{"schema":1,"priorLifecycle":"draining","cleanup":"clean","acceptedConnections":2,"completedConnections":2,"refusedConnections":0,"authority":"leak"}
"#;
        assert!(!agora_cleanup_receipt_is_clean(&mut Cursor::new(extra)));
        assert!(!agora_cleanup_receipt_is_clean(&mut Cursor::new(vec![
            b'x';
            4 * 1_024 + 1
        ])));
    }

    #[test]
    fn deployment_owned_peer_factories_keep_typed_clients_and_reverse_drain() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut host = NativeHost::new();

        let moira: u8 = host
            .start_moira(factory(1_u8, "moira", order.clone()))
            .unwrap();
        let daimon: u16 = host
            .start_daimon(factory(2_u16, "daimon", order.clone()))
            .unwrap();
        let hermes: u32 = host
            .start_hermes(factory(3_u32, "hermes", order.clone()))
            .unwrap();
        let aegis: u64 = host
            .start_aegis(factory(4_u64, "aegis", order.clone()))
            .unwrap();
        let grounding: &'static str = host
            .start_grounding(factory(
                "typed-grounding-client",
                "grounding",
                order.clone(),
            ))
            .unwrap();

        assert_eq!((moira, daimon, hermes, aegis), (1, 2, 3, 4));
        assert_eq!(grounding, "typed-grounding-client");
        host.seal().unwrap();
        let receipt = host.shutdown().unwrap();
        assert!(receipt.clean());
        assert_eq!(
            *order.lock().unwrap(),
            ["grounding", "aegis", "hermes", "daimon", "moira"]
        );
        assert_eq!(
            receipt
                .receipts
                .iter()
                .map(|receipt| &receipt.peer)
                .collect::<Vec<_>>(),
            [
                &HostedPeerId::Grounding,
                &HostedPeerId::Aegis,
                &HostedPeerId::Hermes,
                &HostedPeerId::Daimon,
                &HostedPeerId::Moira,
            ]
        );
    }

    fn factory<C: Send + 'static>(
        client: C,
        name: &'static str,
        order: Arc<Mutex<Vec<&'static str>>>,
    ) -> impl FnOnce() -> TestStart<C> {
        move || {
            Ok((
                client,
                Box::new(move || {
                    order.lock().unwrap().push(name);
                    ShutdownDisposition::Clean
                }),
            ))
        }
    }
}

#[cfg(all(test, feature = "native-chronos"))]
mod tests {
    use super::*;
    use chronos_client::observer::{Observer, ObserverLimits};
    use std::{fs, path::PathBuf};

    fn scratch() -> PathBuf {
        PathBuf::from("/tmp").join(format!("wbc-{}", std::process::id()))
    }

    #[test]
    fn hosts_observes_and_cleanly_drains_real_chronos() {
        let directory = scratch();
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        let mut host = NativeHost::new();
        host.start_chronos("Atlas", &directory, ChronosHostOptions::default())
            .unwrap();
        host.seal().unwrap();

        let mut observer = Observer::new(ObserverLimits {
            page_size: 8,
            max_pages: 2,
            max_entries: 16,
            max_id_bytes: 128,
        })
        .unwrap();
        let mut client = chronos_local::LocalClient::new(directory.clone());
        observer.refresh(&mut client).unwrap();
        assert!(observer.view().is_some());

        let receipt = host.shutdown().unwrap();
        assert!(receipt.clean());
        assert_eq!(receipt.receipts.len(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
