//! Native peer-service ownership for one Pantheon process.
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
};
use std::{path::Path, thread, time::Duration};

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

use crate::reconfiguration::ReconfigurationPlan;
#[cfg(feature = "native-chronos")]
use crate::registry::StartError;
use crate::registry::{
    HostRegistry, HostedPeerId, RegistryError, RegistryLifecycle, RestartFailure, RestartReceipt,
    RuntimeStatus, ShutdownDisposition, StartupFailure, TopologyShutdown,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyRestartReceipt {
    pub operation_id: String,
    pub stopped: TopologyShutdown,
    pub started: Vec<HostedPeerId>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TopologyRestartFailure<E> {
    Plan,
    Lifecycle(RegistryError),
    Stop(TopologyRestartReceipt),
    Build {
        cause: E,
        receipt: TopologyRestartReceipt,
        rollback: Option<TopologyShutdown>,
    },
    Seal {
        cause: RegistryError,
        receipt: TopologyRestartReceipt,
        rollback: Option<TopologyShutdown>,
    },
}

pub struct NativeHost {
    registry: HostRegistry,
    #[cfg(feature = "native-chronos")]
    trusted_worker: Option<ManagedChronosWorker>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedChronosWorkerStatus {
    pub workstream: String,
    pub state: &'static str,
}

#[cfg(feature = "native-chronos")]
struct ManagedChronosWorker {
    workstream: String,
    worker: moira_chronos_worker::TrustedWorkerV1<chronos_standalone::TrustedWorkerHandle>,
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

/// Deployment-neutral options for a Pantheon-owned Chronos service.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChronosHostOptions {
    pub allow_writes: bool,
    pub atomic_state: bool,
    pub observer_history: bool,
    pub allocation_receipts: bool,
}

/// Deployment-neutral inputs for the one private worker capability retained
/// with a hosted Chronos owner. These values contain no grant or task evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronosTrustedWorkerOptions {
    pub agent: String,
    pub grant_validity: Duration,
    pub lease_duration: Duration,
}

/// Opaque deployment inputs for Chronos's read-only product-alignment
/// composition. Pantheon owns readiness and drain only; owner connections,
/// credentials, stakeholder bindings and disclosure stay in the config.
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronosProductAlignmentProcessOptions {
    pub workstream: String,
    pub executable: PathBuf,
    pub config: PathBuf,
    pub listener_directory: PathBuf,
}

/// Explicit deployment inputs for a Pantheon-owned, sovereign Hermes daemon.
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

/// Explicit deployment inputs for one production Daimon application. Model or
/// interactive-agent selection, provider, credential, authority, attempt,
/// storage and recovery policy remain in the opaque owner configuration.
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

/// Explicit deployment inputs for a Pantheon-owned, sovereign Aegis daemon.
/// These values are passed to Aegis unchanged; Pantheon does not interpret
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
/// The config remains Agora-owned; Pantheon uses the socket only for
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
            #[cfg(feature = "native-chronos")]
            trusted_worker: None,
        }
    }

    pub fn status(&self) -> RuntimeStatus {
        self.registry.status()
    }

    /// Credential-free status for the one private worker capability retained
    /// with this topology. No grant, lease, task key, or result is projected.
    pub fn managed_chronos_worker_status(&self) -> Option<ManagedChronosWorkerStatus> {
        #[cfg(feature = "native-chronos")]
        {
            self.trusted_worker
                .as_ref()
                .map(|managed| ManagedChronosWorkerStatus {
                    workstream: managed.workstream.clone(),
                    state: if managed.worker.is_fenced() {
                        "fenced"
                    } else if managed.worker.active().is_some() {
                        "active"
                    } else {
                        "idle"
                    },
                })
        }
        #[cfg(not(feature = "native-chronos"))]
        {
            None
        }
    }

    /// Replaces one exact hosted peer after topology assembly. The peer's
    /// ordinary client is returned unchanged, alongside a stop/start receipt.
    /// The registry never invokes `start` after a refused or uncertain stop.
    pub fn restart_peer<C, E, S>(
        &mut self,
        peer: HostedPeerId,
        start: impl FnOnce() -> Result<(C, S), E>,
    ) -> Result<(C, RestartReceipt), RestartFailure<E>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.restart(peer, start)
    }

    /// Applies the host portion of a validated full-topology restart plan.
    /// The old topology drains before replacement assembly begins. The caller
    /// retains a concrete typed client bundle; the host retains only lifecycle
    /// callbacks. Any partial replacement is rolled back before return.
    pub fn restart_topology<C, E>(
        mut self,
        plan: &ReconfigurationPlan,
        build: impl FnOnce(&mut NativeHost) -> Result<C, E>,
    ) -> Result<(NativeHost, C, TopologyRestartReceipt), TopologyRestartFailure<E>> {
        if plan.version != 1
            || plan.scope != "single_user_local_machine"
            || plan.operation_id.is_empty()
            || plan.minimum_action != "full_local_topology_restart"
            || plan.changes.is_empty()
        {
            return Err(TopologyRestartFailure::Plan);
        }
        let stopped = self.shutdown().map_err(TopologyRestartFailure::Lifecycle)?;
        let mut receipt = TopologyRestartReceipt {
            operation_id: plan.operation_id.clone(),
            stopped,
            started: Vec::new(),
        };
        if !receipt.stopped.clean() {
            return Err(TopologyRestartFailure::Stop(receipt));
        }

        let mut replacement = NativeHost::new();
        let clients = match build(&mut replacement) {
            Ok(clients) => clients,
            Err(cause) => {
                receipt.started = replacement.registry.hosted_peers();
                let rollback = rollback_replacement(&mut replacement);
                return Err(TopologyRestartFailure::Build {
                    cause,
                    receipt,
                    rollback,
                });
            }
        };
        receipt.started = replacement.registry.hosted_peers();
        if let Err(cause) = replacement.seal() {
            let rollback = rollback_replacement(&mut replacement);
            return Err(TopologyRestartFailure::Seal {
                cause,
                receipt,
                rollback,
            });
        }
        Ok((replacement, clients, receipt))
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
            self.start_chronos_owner(name.into(), directory, options, |_| Ok(()))
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

    /// Binds one Chronos owner and returns a Moira behavior object holding its
    /// sole native worker capability. The handle never crosses a socket or UI
    /// boundary; dropping or draining the owner makes later calls fail closed.
    #[cfg(feature = "native-chronos")]
    pub fn start_chronos_worker(
        &mut self,
        name: impl Into<String>,
        directory: &Path,
        options: ChronosHostOptions,
        configuration: moira_chronos_worker::ConfigurationV1,
    ) -> Result<
        moira_chronos_worker::TrustedWorkerV1<chronos_standalone::TrustedWorkerHandle>,
        StartupFailure<String>,
    > {
        self.start_chronos_owner(name.into(), directory, options, |server| {
            let handle = server
                .trusted_worker()
                .map_err(|error| format!("Chronos trusted worker refused: {error}"))?;
            moira_chronos_worker::TrustedWorkerV1::new(handle, configuration)
                .map_err(|_| "Moira Chronos worker configuration refused".to_owned())
        })
    }

    /// Starts one hosted Chronos owner and retains its sole Moira-wrapped
    /// worker capability inside the Pantheon-owned topology. Owner-derived
    /// workflow code may use this capability later; presentation clients never
    /// receive the handle itself.
    pub fn start_managed_chronos_worker(
        &mut self,
        name: impl Into<String>,
        directory: &Path,
        options: ChronosHostOptions,
        worker_options: ChronosTrustedWorkerOptions,
    ) -> Result<(), StartupFailure<String>> {
        #[cfg(feature = "native-chronos")]
        {
            if self.trusted_worker.is_some() {
                return Err(StartupFailure {
                    cause: StartError::Peer(
                        "one trusted Chronos worker is already retained".into(),
                    ),
                    rollback: None,
                });
            }
            let configuration = moira_chronos_worker::ConfigurationV1::new(
                worker_options.agent,
                worker_options.grant_validity,
                worker_options.lease_duration,
            )
            .map_err(|_| StartupFailure {
                cause: StartError::Peer("trusted Chronos worker configuration refused".into()),
                rollback: None,
            })?;
            let workstream = name.into();
            let worker =
                self.start_chronos_worker(workstream.clone(), directory, options, configuration)?;
            self.trusted_worker = Some(ManagedChronosWorker { workstream, worker });
            Ok(())
        }
        #[cfg(not(feature = "native-chronos"))]
        {
            let name = name.into();
            let _ = (directory, options, worker_options);
            self.registry
                .start_or_rollback(HostedPeerId::Chronos(name), || {
                    Err::<((), fn() -> ShutdownDisposition), _>(
                        "native Chronos trusted-worker hosting is not compiled into this deployment"
                            .to_owned(),
                    )
                })
        }
    }

    #[cfg(feature = "native-chronos")]
    fn start_chronos_owner<C>(
        &mut self,
        name: String,
        directory: &Path,
        options: ChronosHostOptions,
        compose: impl FnOnce(&mut Server) -> Result<C, String>,
    ) -> Result<C, StartupFailure<String>> {
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
                let client = compose(&mut server)?;
                let (wake, writer) = UnixStream::pair()
                    .map_err(|error| format!("Chronos shutdown channel refused: {error}"))?;
                let owner = thread::Builder::new()
                    .name("workbench-chronos-owner".into())
                    .spawn(move || server.run_with_shutdown(&wake))
                    .map_err(|error| format!("Chronos owner thread refused: {error}"))?;
                Ok((client, chronos_shutdown(writer, owner)))
            })
    }

    /// Starts the sovereign read-only product-alignment composition unchanged.
    /// The public listener is the readiness boundary; Pantheon never parses the
    /// opaque owner configuration or gains proposal/task mutation authority.
    #[cfg(unix)]
    pub fn start_chronos_product_alignment_process(
        &mut self,
        options: ChronosProductAlignmentProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        let peer = HostedPeerId::ChronosAlignment(options.workstream.clone());
        self.registry.start_or_rollback(peer, || {
            let socket = chronos_local::alignment_socket_path(&options.listener_directory);
            let prior_socket = socket_identity(&socket);
            let mut child = Command::new(&options.executable)
                .arg("--supervised")
                .arg(&options.config)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Chronos alignment process start refused: {error}"))?;
            if let Err(error) =
                wait_for_process_ready(&mut child, &socket, "Chronos alignment", prior_socket)
            {
                stop_failed_process_start(&mut child);
                return Err(error);
            }
            let mut input = child.stdin.take();
            Ok(((), move || stop_supervised_process(&mut child, &mut input)))
        })
    }

    /// Starts an Agora owner supplied by the deployment and returns its
    /// concrete client or presentation input unchanged. Pantheon owns only
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
    /// Pantheon owns only readiness and process drain. Authority, bootstrap,
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
    /// policy; Pantheon retains only the typed client and shutdown receipt.
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
    /// Pantheon owns only readiness and the exact one-byte drain channel; it
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

    /// Starts one production Daimon application in its owner-defined supervised
    /// mode. Pantheon owns readiness and drain only and attaches through the
    /// ordinary public model or agent client selected by the profile.
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
    /// process. Pantheon owns readiness and shutdown only; all ledger,
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

    /// Starts a deployment-supplied Aegis owner. Pantheon does not construct
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
    /// Pantheon owns readiness and shutdown only; Aegis retains authority,
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

    /// Starts a deployment-supplied Tellus/Dendrite owner. The returned
    /// client remains the typed Tellus presentation boundary; this does not
    /// turn MCP into a Pantheon lifecycle or semantic protocol.
    pub fn start_tellus<C, S>(
        &mut self,
        start: impl FnOnce() -> Result<(C, S), String>,
    ) -> Result<C, StartupFailure<String>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        self.registry.start_or_rollback(HostedPeerId::Tellus, start)
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
        allow_add_file: bool,
    ) -> Result<(), StartupFailure<String>> {
        self.registry
            .start_or_rollback(HostedPeerId::Vellum, || -> Result<_, String> {
                let config = VellumOwnerConfig {
                    socket: socket.to_path_buf(),
                    roots: roots.to_vec(),
                    credential: credential.map(Path::to_path_buf),
                    allow_edit,
                    allow_save,
                    allow_add_file,
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
        _allow_add_file: bool,
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

fn rollback_replacement(host: &mut NativeHost) -> Option<TopologyShutdown> {
    match host.registry.lifecycle() {
        RegistryLifecycle::Assembling => host.registry.abort().ok(),
        RegistryLifecycle::Ready | RegistryLifecycle::Degraded => host.registry.shutdown().ok(),
        RegistryLifecycle::Draining | RegistryLifecycle::Stopped => None,
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
    allow_add_file: bool,
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
        if config.allow_add_file {
            actions = actions.join(AuthorityActionSetV1::singleton(
                AuthorityActionV1::AdmitFile,
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
    use crate::reconfiguration::{ReconfigurationChange, ReconfigurationPlan};
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
        let tellus: &'static str = host
            .start_tellus(factory("typed-tellus-client", "tellus", order.clone()))
            .unwrap();

        assert_eq!((moira, daimon, hermes, aegis), (1, 2, 3, 4));
        assert_eq!(tellus, "typed-tellus-client");
        host.seal().unwrap();
        let receipt = host.shutdown().unwrap();
        assert!(receipt.clean());
        assert_eq!(
            *order.lock().unwrap(),
            ["tellus", "aegis", "hermes", "daimon", "moira"]
        );
        assert_eq!(
            receipt
                .receipts
                .iter()
                .map(|receipt| &receipt.peer)
                .collect::<Vec<_>>(),
            [
                &HostedPeerId::Tellus,
                &HostedPeerId::Aegis,
                &HostedPeerId::Hermes,
                &HostedPeerId::Daimon,
                &HostedPeerId::Moira,
            ]
        );
    }

    #[test]
    fn deployment_owned_peer_restart_returns_typed_client_and_receipt() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut host = NativeHost::new();
        host.start_moira(factory(1_u8, "first", order.clone()))
            .unwrap();
        host.seal().unwrap();
        let (client, receipt) = host
            .restart_peer(
                HostedPeerId::Moira,
                factory(2_u16, "replacement", order.clone()),
            )
            .unwrap();
        assert_eq!(client, 2_u16);
        assert_eq!(
            receipt,
            RestartReceipt {
                peer: HostedPeerId::Moira,
                stopped: ShutdownDisposition::Clean,
                replacement_started: true,
            }
        );
        assert_eq!(*order.lock().unwrap(), ["first"]);
        assert!(host.shutdown().unwrap().clean());
        assert_eq!(*order.lock().unwrap(), ["first", "replacement"]);
    }

    fn topology_plan(operation_id: &str) -> ReconfigurationPlan {
        ReconfigurationPlan {
            version: 1,
            operation_id: operation_id.into(),
            scope: "single_user_local_machine",
            live_apply_supported: false,
            minimum_action: "full_local_topology_restart",
            changes: vec![ReconfigurationChange {
                target: "moira",
                change: "changed",
                required_action: "full_local_topology_restart_required",
            }],
        }
    }

    #[test]
    fn full_topology_restart_drains_then_returns_typed_replacement_bundle() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut host = NativeHost::new();
        host.start_moira(factory(1_u8, "old", order.clone()))
            .unwrap();
        host.seal().unwrap();

        let (mut replacement, client, receipt) = host
            .restart_topology(&topology_plan("replace-1"), |replacement| {
                replacement.start_moira(factory(2_u16, "new", order.clone()))
            })
            .unwrap();
        assert_eq!(client, 2_u16);
        assert_eq!(receipt.operation_id, "replace-1");
        assert_eq!(
            receipt.stopped.receipts,
            vec![crate::registry::ShutdownReceipt {
                peer: HostedPeerId::Moira,
                disposition: ShutdownDisposition::Clean,
            }]
        );
        assert_eq!(receipt.started, vec![HostedPeerId::Moira]);
        let status = replacement.status();
        assert_eq!(status.lifecycle, RegistryLifecycle::Ready);
        assert_eq!(status.hosted_peers, vec![HostedPeerId::Moira]);
        assert!(status.reconfiguration_allowed);
        assert_eq!(*order.lock().unwrap(), ["old"]);
        assert!(replacement.shutdown().unwrap().clean());
        assert_eq!(*order.lock().unwrap(), ["old", "new"]);
    }

    #[test]
    fn full_topology_restart_rolls_back_partial_replacement() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut host = NativeHost::new();
        host.start_moira(factory(1_u8, "old", order.clone()))
            .unwrap();
        host.seal().unwrap();

        let failure = host.restart_topology(&topology_plan("replace-2"), |replacement| {
            replacement
                .start_daimon(factory(2_u16, "partial", order.clone()))
                .unwrap();
            Err::<(), _>("replacement refused")
        });
        let Err(failure) = failure else {
            panic!("partial replacement unexpectedly succeeded");
        };
        assert_eq!(
            failure,
            TopologyRestartFailure::Build {
                cause: "replacement refused",
                receipt: TopologyRestartReceipt {
                    operation_id: "replace-2".into(),
                    stopped: TopologyShutdown {
                        receipts: vec![crate::registry::ShutdownReceipt {
                            peer: HostedPeerId::Moira,
                            disposition: ShutdownDisposition::Clean,
                        }],
                    },
                    started: vec![HostedPeerId::Daimon],
                },
                rollback: Some(TopologyShutdown {
                    receipts: vec![crate::registry::ShutdownReceipt {
                        peer: HostedPeerId::Daimon,
                        disposition: ShutdownDisposition::Clean,
                    }],
                }),
            }
        );
        assert_eq!(*order.lock().unwrap(), ["old", "partial"]);
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
    use chronos_client::{
        WireClientV1,
        observer::{Observer, ObserverLimits},
    };
    use chronos_protocol::{Command, Outcome, Request, RetrySafety, Revision, TaskSpec};
    use moira_chronos_worker::{
        ClaimIntentV1, ConfigurationV1, ErrorV1, ResultBindingV1, SettlementV1,
    };
    use moira_protocol::{DispositionV1, EffectIdV1, ResultReceiptV1};
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        time::{Duration, SystemTime},
    };

    fn scratch(name: &str) -> PathBuf {
        PathBuf::from("/tmp").join(format!("pnt-{}-{name}", std::process::id()))
    }

    #[test]
    fn hosts_observes_and_cleanly_drains_real_chronos() {
        let directory = scratch("observe");
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
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

    #[test]
    fn hosted_worker_capability_fences_after_owner_drain() {
        let directory = scratch("worker");
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let mut host = NativeHost::new();
        let configuration = ConfigurationV1::new(
            "pantheon-j1-worker",
            Duration::from_secs(60),
            Duration::from_secs(30),
        )
        .unwrap();
        let mut worker = host
            .start_chronos_worker(
                "Atlas",
                &directory,
                ChronosHostOptions {
                    allow_writes: true,
                    atomic_state: true,
                    ..Default::default()
                },
                configuration,
            )
            .unwrap();
        host.seal().unwrap();
        assert!(host.shutdown().unwrap().clean());

        assert!(matches!(
            worker.claim(ClaimIntentV1 {
                task_id: "unreachable-after-drain".into(),
                external_task_key: [1; 16],
                expected_revision: 0,
                now: SystemTime::UNIX_EPOCH,
            }),
            Err(ErrorV1::OutcomeUnknown(_))
        ));
        assert!(worker.is_fenced());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn managed_worker_is_retained_and_projected_without_authority_material() {
        let directory = scratch("managed-worker");
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let mut host = NativeHost::new();
        host.start_managed_chronos_worker(
            "Product",
            &directory,
            ChronosHostOptions {
                allow_writes: true,
                atomic_state: true,
                ..Default::default()
            },
            ChronosTrustedWorkerOptions {
                agent: "product-manager".into(),
                grant_validity: Duration::from_secs(60),
                lease_duration: Duration::from_secs(30),
            },
        )
        .unwrap();
        host.seal().unwrap();

        assert_eq!(
            host.managed_chronos_worker_status(),
            Some(ManagedChronosWorkerStatus {
                workstream: "Product".into(),
                state: "idle",
            })
        );
        assert!(host.shutdown().unwrap().clean());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn hosted_worker_claims_and_completes_one_exact_task() {
        let directory = scratch("worker-complete");
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();

        let mut host = NativeHost::new();
        let configuration = ConfigurationV1::new(
            "pantheon-j1-worker",
            Duration::from_secs(60),
            Duration::from_secs(30),
        )
        .unwrap();
        let mut worker = host
            .start_chronos_worker(
                "Atlas",
                &directory,
                ChronosHostOptions {
                    allow_writes: true,
                    atomic_state: true,
                    ..Default::default()
                },
                configuration,
            )
            .unwrap();
        host.seal().unwrap();

        let mut client = chronos_local::LocalClient::new(directory.clone());
        let response = client
            .request(Request {
                version: 1,
                command: Command::SubmitGraph {
                    expected: Revision::Finite("0".into()),
                    tasks: vec![TaskSpec {
                        id: "j1-owner-packaging".into(),
                        priority: 0,
                        dependencies: vec![],
                        capabilities: vec![],
                        max_attempts: 1,
                        retry_safety: RetrySafety::FenceUnknown,
                    }],
                },
            })
            .unwrap();
        assert!(matches!(response.outcome, Outcome::Applied {}));

        let claim = worker
            .claim(ClaimIntentV1 {
                task_id: "j1-owner-packaging".into(),
                external_task_key: [7; 16],
                expected_revision: 1,
                now: SystemTime::UNIX_EPOCH,
            })
            .unwrap();
        assert_eq!(claim.receipt.task.external_task_key, [7; 16]);
        assert_eq!(claim.receipt.attempt_number, 1);

        let binding = ResultBindingV1 {
            effect: EffectIdV1([8; 16]),
            execution_id: [9; 16],
        };
        worker.bind_result(binding).unwrap();
        assert_eq!(
            worker
                .settle(
                    ResultReceiptV1 {
                        effect: binding.effect,
                        execution_id: binding.execution_id,
                        task: claim.receipt.task,
                        attempt_number: claim.receipt.attempt_number,
                        disposition: DispositionV1::Completed,
                    },
                    SystemTime::UNIX_EPOCH,
                )
                .unwrap(),
            SettlementV1::Completed
        );
        assert!(worker.active().is_none());

        let response = client
            .request(Request {
                version: 1,
                command: Command::Snapshot {
                    task: "j1-owner-packaging".into(),
                },
            })
            .unwrap();
        let Outcome::Snapshot {
            value: Some(entry), ..
        } = response.outcome
        else {
            panic!("completed task snapshot missing");
        };
        assert!(matches!(
            entry.snapshot.phase,
            chronos_protocol::Phase::Succeeded
        ));

        assert!(host.shutdown().unwrap().clean());
        fs::remove_dir_all(directory).unwrap();
    }
}
