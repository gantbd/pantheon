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

use chronos_standalone::{Server, ServerOptions};

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

    pub fn shutdown(&mut self) -> Result<TopologyShutdown, RegistryError> {
        self.registry.shutdown()
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
