//! Operational lifecycle registry for a hosted Pantheon topology.
//!
//! The registry erases only shutdown callbacks. Service clients are returned
//! to the caller with their concrete types, so peer protocols, authority, and
//! semantic errors never enter a universal command bus.

use std::collections::HashSet;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HostedPeerId {
    Agora,
    Moira,
    Daimon,
    Hermes,
    Aegis,
    Grounding,
    Chronos(String),
    Vellum,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RegistryLifecycle {
    #[default]
    Assembling,
    Ready,
    Degraded,
    Draining,
    Stopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShutdownDisposition {
    Clean,
    Refused,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownReceipt {
    pub peer: HostedPeerId,
    pub disposition: ShutdownDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyShutdown {
    pub receipts: Vec<ShutdownReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestartReceipt {
    pub peer: HostedPeerId,
    pub stopped: ShutdownDisposition,
    pub replacement_started: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeStatus {
    pub lifecycle: RegistryLifecycle,
    pub hosted_peers: Vec<HostedPeerId>,
    pub reconfiguration_allowed: bool,
    pub restart_receipts: Vec<RestartReceipt>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RestartFailure<E> {
    Lifecycle(RegistryLifecycle),
    Missing,
    Stop(RestartReceipt),
    Start { cause: E, receipt: RestartReceipt },
}

impl TopologyShutdown {
    pub fn clean(&self) -> bool {
        self.receipts
            .iter()
            .all(|receipt| receipt.disposition == ShutdownDisposition::Clean)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StartError<E> {
    Duplicate,
    Lifecycle(RegistryLifecycle),
    Peer(E),
}

#[derive(Debug, Eq, PartialEq)]
pub struct StartupFailure<E> {
    pub cause: StartError<E>,
    pub rollback: Option<TopologyShutdown>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryError {
    Empty,
    Lifecycle(RegistryLifecycle),
}

type Shutdown = Box<dyn FnMut() -> ShutdownDisposition + Send>;

pub struct HostRegistry {
    lifecycle: RegistryLifecycle,
    peers: HashSet<HostedPeerId>,
    shutdown: Vec<(HostedPeerId, Shutdown)>,
    restart_receipts: Vec<RestartReceipt>,
}

impl Default for HostRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HostRegistry {
    pub fn new() -> Self {
        Self {
            lifecycle: RegistryLifecycle::Assembling,
            peers: HashSet::new(),
            shutdown: Vec::new(),
            restart_receipts: Vec::new(),
        }
    }

    pub fn lifecycle(&self) -> RegistryLifecycle {
        self.lifecycle
    }

    pub fn hosted_peers(&self) -> Vec<HostedPeerId> {
        self.shutdown.iter().map(|(peer, _)| peer.clone()).collect()
    }

    pub fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            lifecycle: self.lifecycle,
            hosted_peers: self.hosted_peers(),
            reconfiguration_allowed: self.lifecycle == RegistryLifecycle::Ready,
            restart_receipts: self.restart_receipts.clone(),
        }
    }

    /// Starts one peer and returns its concrete client while retaining only its
    /// operational shutdown callback. Failed starts add no registry entry.
    pub fn start<C, E, S>(
        &mut self,
        peer: HostedPeerId,
        start: impl FnOnce() -> Result<(C, S), E>,
    ) -> Result<C, StartError<E>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        if self.lifecycle != RegistryLifecycle::Assembling {
            return Err(StartError::Lifecycle(self.lifecycle));
        }
        if self.peers.contains(&peer) {
            return Err(StartError::Duplicate);
        }
        let (client, shutdown) = start().map_err(StartError::Peer)?;
        self.peers.insert(peer.clone());
        self.shutdown.push((peer, Box::new(shutdown)));
        Ok(client)
    }

    /// Starts one peer as part of an all-or-nothing topology assembly. Any
    /// failure while assembling aborts already-started peers in reverse order
    /// and returns their exact shutdown receipts. Lifecycle errors after the
    /// topology is sealed do not disturb the running topology.
    pub fn start_or_rollback<C, E, S>(
        &mut self,
        peer: HostedPeerId,
        start: impl FnOnce() -> Result<(C, S), E>,
    ) -> Result<C, StartupFailure<E>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        match self.start(peer, start) {
            Ok(client) => Ok(client),
            Err(cause) => {
                let rollback = if self.lifecycle == RegistryLifecycle::Assembling {
                    Some(self.abort().expect("assembling topology can be aborted"))
                } else {
                    None
                };
                Err(StartupFailure { cause, rollback })
            }
        }
    }

    pub fn seal(&mut self) -> Result<(), RegistryError> {
        if self.lifecycle != RegistryLifecycle::Assembling {
            return Err(RegistryError::Lifecycle(self.lifecycle));
        }
        if self.shutdown.is_empty() {
            return Err(RegistryError::Empty);
        }
        self.lifecycle = RegistryLifecycle::Ready;
        Ok(())
    }

    /// Restarts one exact hosted peer without disturbing its siblings. The
    /// replacement factory is invoked only after a confirmed clean stop. A
    /// refused or uncertain stop is retained as unresolved drain evidence and
    /// fences further reconfiguration by moving the registry to `Degraded`.
    pub fn restart<C, E, S>(
        &mut self,
        peer: HostedPeerId,
        start: impl FnOnce() -> Result<(C, S), E>,
    ) -> Result<(C, RestartReceipt), RestartFailure<E>>
    where
        S: FnMut() -> ShutdownDisposition + Send + 'static,
    {
        if self.lifecycle != RegistryLifecycle::Ready {
            return Err(RestartFailure::Lifecycle(self.lifecycle));
        }
        let Some(index) = self
            .shutdown
            .iter()
            .position(|(candidate, _)| candidate == &peer)
        else {
            return Err(RestartFailure::Missing);
        };
        let (_, mut shutdown) = self.shutdown.remove(index);
        let stopped = shutdown();
        let receipt = RestartReceipt {
            peer: peer.clone(),
            stopped,
            replacement_started: false,
        };
        if stopped != ShutdownDisposition::Clean {
            self.lifecycle = RegistryLifecycle::Degraded;
            self.restart_receipts.push(receipt.clone());
            self.shutdown.insert(
                index,
                (
                    peer,
                    Box::new(move || stopped) as Box<dyn FnMut() -> ShutdownDisposition + Send>,
                ),
            );
            return Err(RestartFailure::Stop(receipt));
        }
        self.peers.remove(&peer);
        let (client, replacement_shutdown) = match start() {
            Ok(replacement) => replacement,
            Err(cause) => {
                self.lifecycle = RegistryLifecycle::Degraded;
                self.restart_receipts.push(receipt.clone());
                return Err(RestartFailure::Start { cause, receipt });
            }
        };
        self.peers.insert(peer.clone());
        self.shutdown
            .insert(index, (peer.clone(), Box::new(replacement_shutdown)));
        let receipt = RestartReceipt {
            peer,
            stopped,
            replacement_started: true,
        };
        self.restart_receipts.push(receipt.clone());
        Ok((client, receipt))
    }

    /// Drains in reverse startup order. Every callback is attempted exactly
    /// once; refused or uncertain peers do not suppress later receipts.
    pub fn shutdown(&mut self) -> Result<TopologyShutdown, RegistryError> {
        if !matches!(
            self.lifecycle,
            RegistryLifecycle::Assembling | RegistryLifecycle::Ready | RegistryLifecycle::Degraded
        ) {
            return Err(RegistryError::Lifecycle(self.lifecycle));
        }
        if self.shutdown.is_empty() {
            return Err(RegistryError::Empty);
        }
        Ok(self.drain())
    }

    /// Aborts topology assembly. Unlike orderly shutdown, aborting an empty
    /// assembly is valid and yields an empty clean receipt set.
    pub fn abort(&mut self) -> Result<TopologyShutdown, RegistryError> {
        if self.lifecycle != RegistryLifecycle::Assembling {
            return Err(RegistryError::Lifecycle(self.lifecycle));
        }
        Ok(self.drain())
    }

    fn drain(&mut self) -> TopologyShutdown {
        self.lifecycle = RegistryLifecycle::Draining;
        let mut receipts = Vec::with_capacity(self.shutdown.len());
        for (peer, mut shutdown) in self.shutdown.drain(..).rev() {
            receipts.push(ShutdownReceipt {
                peer,
                disposition: shutdown(),
            });
        }
        self.lifecycle = RegistryLifecycle::Stopped;
        TopologyShutdown { receipts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn preserves_typed_clients_and_drains_in_reverse_start_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut registry = HostRegistry::new();
        let first_order = order.clone();
        let chronos: u64 = registry
            .start(HostedPeerId::Chronos("Atlas".into()), || {
                Ok::<_, ()>((7_u64, move || {
                    first_order.lock().unwrap().push("chronos");
                    ShutdownDisposition::Clean
                }))
            })
            .unwrap();
        let second_order = order.clone();
        let vellum: String = registry
            .start(HostedPeerId::Vellum, || {
                Ok::<_, ()>(("vellum-client".to_owned(), move || {
                    second_order.lock().unwrap().push("vellum");
                    ShutdownDisposition::Clean
                }))
            })
            .unwrap();
        assert_eq!(chronos, 7);
        assert_eq!(vellum, "vellum-client");
        registry.seal().unwrap();
        let receipt = registry.shutdown().unwrap();
        assert!(receipt.clean());
        assert_eq!(*order.lock().unwrap(), ["vellum", "chronos"]);
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Stopped);
    }

    #[test]
    fn partial_start_and_uncertain_shutdown_remain_explicit() {
        let mut registry = HostRegistry::new();
        registry
            .start(HostedPeerId::Agora, || {
                Ok::<_, &'static str>(((), || ShutdownDisposition::OutcomeUnknown))
            })
            .unwrap();
        assert_eq!(
            registry.start_or_rollback(HostedPeerId::Vellum, || {
                Err::<((), fn() -> ShutdownDisposition), _>("refused")
            }),
            Err(StartupFailure {
                cause: StartError::Peer("refused"),
                rollback: Some(TopologyShutdown {
                    receipts: vec![ShutdownReceipt {
                        peer: HostedPeerId::Agora,
                        disposition: ShutdownDisposition::OutcomeUnknown,
                    }],
                }),
            })
        );
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Stopped);
    }

    #[test]
    fn first_start_failure_aborts_to_an_empty_stopped_topology() {
        let mut registry = HostRegistry::new();
        assert_eq!(
            registry.start_or_rollback(HostedPeerId::Chronos("Atlas".into()), || {
                Err::<((), fn() -> ShutdownDisposition), _>("refused")
            }),
            Err(StartupFailure {
                cause: StartError::Peer("refused"),
                rollback: Some(TopologyShutdown { receipts: vec![] }),
            })
        );
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Stopped);
    }

    #[test]
    fn duplicate_and_post_seal_start_are_refused_without_invoking_factories() {
        let mut registry = HostRegistry::new();
        registry
            .start(HostedPeerId::Vellum, || {
                Ok::<_, ()>(((), || ShutdownDisposition::Clean))
            })
            .unwrap();
        assert_eq!(
            registry.start(HostedPeerId::Vellum, || {
                panic!("duplicate factory must not run");
                #[allow(unreachable_code)]
                Ok::<_, ()>(((), || ShutdownDisposition::Clean))
            }),
            Err(StartError::Duplicate)
        );
        registry.seal().unwrap();
        assert_eq!(
            registry.start(HostedPeerId::Agora, || {
                panic!("sealed factory must not run");
                #[allow(unreachable_code)]
                Ok::<_, ()>(((), || ShutdownDisposition::Clean))
            }),
            Err(StartError::Lifecycle(RegistryLifecycle::Ready))
        );
    }

    #[test]
    fn restart_requires_clean_stop_and_preserves_exact_receipts() {
        let mut registry = HostRegistry::new();
        registry
            .start(HostedPeerId::Chronos("Atlas".into()), || {
                Ok::<_, ()>(("first", || ShutdownDisposition::Clean))
            })
            .unwrap();
        registry.seal().unwrap();
        let (client, receipt) = registry
            .restart(HostedPeerId::Chronos("Atlas".into()), || {
                Ok::<_, ()>(("replacement", || ShutdownDisposition::Clean))
            })
            .unwrap();
        assert_eq!(client, "replacement");
        assert_eq!(
            receipt,
            RestartReceipt {
                peer: HostedPeerId::Chronos("Atlas".into()),
                stopped: ShutdownDisposition::Clean,
                replacement_started: true,
            }
        );
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Ready);
        assert_eq!(
            registry.status(),
            RuntimeStatus {
                lifecycle: RegistryLifecycle::Ready,
                hosted_peers: vec![HostedPeerId::Chronos("Atlas".into())],
                reconfiguration_allowed: true,
                restart_receipts: vec![receipt],
            }
        );
        assert!(registry.shutdown().unwrap().clean());
    }

    #[test]
    fn uncertain_restart_fences_reconfiguration_and_survives_final_drain() {
        let mut registry = HostRegistry::new();
        registry
            .start(HostedPeerId::Agora, || {
                Ok::<_, ()>(((), || ShutdownDisposition::OutcomeUnknown))
            })
            .unwrap();
        registry.seal().unwrap();
        let mut replacement_invoked = false;
        let failure = registry.restart(HostedPeerId::Agora, || {
            replacement_invoked = true;
            Ok::<_, ()>(((), || ShutdownDisposition::Clean))
        });
        assert_eq!(
            failure,
            Err(RestartFailure::Stop(RestartReceipt {
                peer: HostedPeerId::Agora,
                stopped: ShutdownDisposition::OutcomeUnknown,
                replacement_started: false,
            }))
        );
        assert!(!replacement_invoked);
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Degraded);
        let status = registry.status();
        assert!(!status.reconfiguration_allowed);
        assert_eq!(status.hosted_peers, vec![HostedPeerId::Agora]);
        assert_eq!(status.restart_receipts.len(), 1);
        assert_eq!(
            status.restart_receipts[0].stopped,
            ShutdownDisposition::OutcomeUnknown
        );
        assert_eq!(
            registry.shutdown().unwrap().receipts,
            vec![ShutdownReceipt {
                peer: HostedPeerId::Agora,
                disposition: ShutdownDisposition::OutcomeUnknown,
            }]
        );
    }

    #[test]
    fn failed_replacement_leaves_a_degraded_but_drainable_topology() {
        let mut registry = HostRegistry::new();
        registry
            .start(HostedPeerId::Agora, || {
                Ok::<_, &'static str>(((), || ShutdownDisposition::Clean))
            })
            .unwrap();
        registry
            .start(HostedPeerId::Vellum, || {
                Ok::<_, &'static str>(((), || ShutdownDisposition::Clean))
            })
            .unwrap();
        registry.seal().unwrap();
        assert_eq!(
            registry.restart(HostedPeerId::Agora, || {
                Err::<((), fn() -> ShutdownDisposition), _>("replacement refused")
            }),
            Err(RestartFailure::Start {
                cause: "replacement refused",
                receipt: RestartReceipt {
                    peer: HostedPeerId::Agora,
                    stopped: ShutdownDisposition::Clean,
                    replacement_started: false,
                },
            })
        );
        assert_eq!(registry.lifecycle(), RegistryLifecycle::Degraded);
        assert_eq!(
            registry.shutdown().unwrap().receipts,
            vec![ShutdownReceipt {
                peer: HostedPeerId::Vellum,
                disposition: ShutdownDisposition::Clean,
            }]
        );
    }
}
