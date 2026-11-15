//! Native peer-service ownership for one Workbench process.
//!
//! This module owns only operational lifecycle. Each peer keeps its own
//! service, protocol, state root, authority configuration, and concrete client.

use std::{
    io::Write,
    os::unix::net::UnixStream,
    path::Path,
    thread::{self, JoinHandle},
};
#[cfg(target_os = "macos")]
use std::{path::PathBuf, sync::mpsc};

use chronos_standalone::{Server, ServerOptions};
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
        options: ServerOptions,
    ) -> Result<(), StartupFailure<String>> {
        let name = name.into();
        self.registry
            .start_or_rollback(HostedPeerId::Chronos(name), || -> Result<_, String> {
                let mut server =
                    Server::bind_with_options(directory, options).map_err(ToOwned::to_owned)?;
                let (wake, writer) = UnixStream::pair()
                    .map_err(|error| format!("Chronos shutdown channel refused: {error}"))?;
                let owner = thread::Builder::new()
                    .name("workbench-chronos-owner".into())
                    .spawn(move || server.run_with_shutdown(&wake))
                    .map_err(|error| format!("Chronos owner thread refused: {error}"))?;
                Ok(((), chronos_shutdown(writer, owner)))
            })
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
        host.start_chronos("Atlas", &directory, ServerOptions::default())
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
