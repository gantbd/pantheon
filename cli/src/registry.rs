//! Operational lifecycle registry for a hosted Workbench topology.
//!
//! The registry erases only shutdown callbacks. Service clients are returned
//! to the caller with their concrete types, so peer protocols, authority, and
//! semantic errors never enter a universal command bus.

use std::collections::HashSet;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HostedPeerId {
    Agora,
    Chronos(String),
    Vellum,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RegistryLifecycle {
    #[default]
    Assembling,
    Ready,
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
        }
    }

    pub fn lifecycle(&self) -> RegistryLifecycle {
        self.lifecycle
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

    /// Drains in reverse startup order. Every callback is attempted exactly
    /// once; refused or uncertain peers do not suppress later receipts.
    pub fn shutdown(&mut self) -> Result<TopologyShutdown, RegistryError> {
        if !matches!(
            self.lifecycle,
            RegistryLifecycle::Assembling | RegistryLifecycle::Ready
        ) {
            return Err(RegistryError::Lifecycle(self.lifecycle));
        }
        if self.shutdown.is_empty() {
            return Err(RegistryError::Empty);
        }
        self.lifecycle = RegistryLifecycle::Draining;
        let mut receipts = Vec::with_capacity(self.shutdown.len());
        for (peer, mut shutdown) in self.shutdown.drain(..).rev() {
            receipts.push(ShutdownReceipt {
                peer,
                disposition: shutdown(),
            });
        }
        self.lifecycle = RegistryLifecycle::Stopped;
        Ok(TopologyShutdown { receipts })
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
            registry.start(HostedPeerId::Vellum, || {
                Err::<((), fn() -> ShutdownDisposition), _>("refused")
            }),
            Err(StartError::Peer("refused"))
        );
        let receipt = registry.shutdown().unwrap();
        assert!(!receipt.clean());
        assert_eq!(
            receipt.receipts,
            [ShutdownReceipt {
                peer: HostedPeerId::Agora,
                disposition: ShutdownDisposition::OutcomeUnknown,
            }]
        );
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
}
