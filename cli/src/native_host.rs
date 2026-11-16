//! Native peer-service ownership for one Workbench process.
//!
//! This module owns only operational lifecycle. Each peer keeps its own
//! service, protocol, state root, authority configuration, and concrete client.

#[cfg(target_os = "macos")]
use std::sync::mpsc;
#[cfg(any(feature = "native-chronos", target_os = "macos"))]
use std::thread::JoinHandle;
#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::FileTypeExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
#[cfg(feature = "native-chronos")]
use std::{io::Write, os::unix::net::UnixStream};
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
fn wait_for_hermes_ready(child: &mut Child, socket: &Path) -> Result<(), String> {
    for _ in 0..400 {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("Hermes readiness check failed: {error}"))?
        {
            return Err(format!("Hermes stopped before readiness: {status}"));
        }
        if fs::symlink_metadata(socket).is_ok_and(|metadata| metadata.file_type().is_socket()) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(5));
    }
    Err("Hermes readiness deadline expired".to_owned())
}

#[cfg(unix)]
fn signal_hermes(child: &Child) -> Result<(), nix::errno::Errno> {
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(i32::try_from(child.id()).unwrap_or(i32::MAX)),
        nix::sys::signal::Signal::SIGTERM,
    )
}

#[cfg(unix)]
fn stop_failed_start(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = signal_hermes(child);
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
fn stop_hermes_process(child: &mut Child) -> ShutdownDisposition {
    if let Ok(Some(status)) = child.try_wait() {
        return if status.success() {
            ShutdownDisposition::Clean
        } else {
            ShutdownDisposition::Refused
        };
    }
    if signal_hermes(child).is_err() {
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

    /// Starts the owner-supplied read-only Hermes application as a sovereign
    /// process. Workbench owns readiness and shutdown only; all ledger,
    /// authentication, authorization, persistence and wire behavior remains in
    /// Hermes, and ordinary Hermes clients attach to the configured socket.
    #[cfg(unix)]
    pub fn start_hermes_process(
        &mut self,
        options: HermesProcessOptions,
    ) -> Result<(), StartupFailure<String>> {
        self.start_hermes(|| {
            let mut child = Command::new(&options.executable)
                .arg(if options.create { "create" } else { "open" })
                .arg(&options.directory)
                .arg(&options.socket)
                .arg(options.effective_uid.to_string())
                .arg(options.effective_gid.to_string())
                .arg(&options.subject)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("Hermes process start refused: {error}"))?;
            if let Err(error) = wait_for_hermes_ready(&mut child, &options.socket) {
                stop_failed_start(&mut child);
                return Err(error);
            }
            Ok(((), move || stop_hermes_process(&mut child)))
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
    use std::sync::{Arc, Mutex};

    type TestShutdown = Box<dyn FnMut() -> ShutdownDisposition + Send + 'static>;
    type TestStart<C> = Result<(C, TestShutdown), String>;

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
