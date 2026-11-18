//! Machine-readable local runtime management evidence.

use serde::Serialize;
use serde_json::Value;

use crate::registry::{HostedPeerId, RegistryLifecycle, ShutdownDisposition, TopologyShutdown};

#[derive(Clone, Debug, Serialize)]
pub struct PeerRuntime {
    pub peer: String,
    pub instance: Option<String>,
    pub deployment: &'static str,
    pub lifecycle: &'static str,
    pub observed_readiness: &'static str,
    pub certainty: &'static str,
    pub fencing: &'static str,
    pub fenced_work_items: Option<u64>,
    pub revision: Value,
    pub authority_mode: &'static str,
    pub reconfiguration: &'static str,
    pub observation_status: String,
    pub shutdown_disposition: Option<&'static str>,
}

impl PeerRuntime {
    pub fn from_observation(observation: &Value, hosted: bool) -> Self {
        let status = observation
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("invalid_reply");
        let uncertain = matches!(status, "outcome_unknown" | "unresolved_outcome");
        let readiness = match status {
            "current" => "ready",
            "partial" => "degraded",
            "outcome_unknown" | "unresolved_outcome" => "unknown",
            _ => "not_ready",
        };
        let fenced_work_items = observation
            .get("census")
            .and_then(|census| census.get("fenced"))
            .and_then(Value::as_u64);
        let fencing = match (status, fenced_work_items) {
            ("current", Some(count)) if count != 0 => "workload_fenced",
            ("current" | "partial", _) => "none_observed",
            ("resync_required", _) => "required",
            ("outcome_unknown" | "unresolved_outcome", _) => "uncertain",
            _ => "unknown",
        };
        Self {
            peer: observation
                .get("peer")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned(),
            instance: observation
                .get("instance")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            deployment: if hosted { "hosted" } else { "attached" },
            lifecycle: if hosted { "running" } else { "external" },
            observed_readiness: readiness,
            certainty: if uncertain { "unknown" } else { "confirmed" },
            fencing,
            fenced_work_items,
            revision: observation.get("revision").cloned().unwrap_or(Value::Null),
            authority_mode: "peer_owned_read_only",
            reconfiguration: "restart_required",
            observation_status: status.to_owned(),
            shutdown_disposition: None,
        }
    }

    fn apply_shutdown(&mut self, disposition: ShutdownDisposition) {
        let (lifecycle, certainty, shutdown_disposition) = match disposition {
            ShutdownDisposition::Clean => ("stopped", self.certainty, "clean"),
            ShutdownDisposition::Refused => ("shutdown_refused", self.certainty, "refused"),
            ShutdownDisposition::OutcomeUnknown => {
                ("shutdown_unknown", "unknown", "outcome_unknown")
            }
        };
        self.lifecycle = lifecycle;
        self.certainty = certainty;
        self.shutdown_disposition = Some(shutdown_disposition);
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalManagementSnapshot {
    pub scope: &'static str,
    pub operation_id: String,
    pub topology_lifecycle: &'static str,
    pub peers: Vec<PeerRuntime>,
}

impl LocalManagementSnapshot {
    pub fn live(
        operation_id: String,
        peers: Vec<PeerRuntime>,
        lifecycle: Option<RegistryLifecycle>,
    ) -> Self {
        let topology_lifecycle = match lifecycle {
            None => "external",
            Some(RegistryLifecycle::Assembling) => "local_host_assembling",
            Some(RegistryLifecycle::Ready) => "local_host_ready",
            Some(RegistryLifecycle::Degraded) => "local_host_degraded",
            Some(RegistryLifecycle::Draining) => "local_host_draining",
            Some(RegistryLifecycle::Stopped) => "local_host_stopped",
        };
        Self {
            scope: "single_user_local_machine",
            operation_id,
            topology_lifecycle,
            peers,
        }
    }

    pub fn after_shutdown(
        operation_id: String,
        mut peers: Vec<PeerRuntime>,
        shutdown: Result<Option<&TopologyShutdown>, ()>,
    ) -> Self {
        let topology_lifecycle = match shutdown {
            Ok(Some(shutdown)) => {
                for receipt in &shutdown.receipts {
                    let (peer, instance) = peer_identity(&receipt.peer);
                    if let Some(runtime) = peers.iter_mut().find(|runtime| {
                        runtime.peer == peer && runtime.instance.as_deref() == instance.as_deref()
                    }) {
                        runtime.apply_shutdown(receipt.disposition);
                    }
                }
                if shutdown.clean() {
                    "local_host_stopped"
                } else {
                    "local_host_shutdown_incomplete"
                }
            }
            Ok(None) => "external",
            Err(()) => {
                for runtime in &mut peers {
                    if runtime.deployment == "hosted" {
                        runtime.lifecycle = "shutdown_unknown";
                        runtime.certainty = "unknown";
                    }
                }
                "local_host_shutdown_unknown"
            }
        };
        Self {
            scope: "single_user_local_machine",
            operation_id,
            topology_lifecycle,
            peers,
        }
    }
}

fn peer_identity(peer: &HostedPeerId) -> (&'static str, Option<String>) {
    match peer {
        HostedPeerId::Agora => ("agora", None),
        HostedPeerId::Moira => ("moira", None),
        HostedPeerId::Daimon => ("daimon", None),
        HostedPeerId::Hermes => ("hermes", None),
        HostedPeerId::Aegis => ("aegis", None),
        HostedPeerId::Tellus => ("tellus", None),
        HostedPeerId::Chronos(instance) => ("chronos", Some(instance.clone())),
        HostedPeerId::Vellum => ("vellum", None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ShutdownReceipt;

    #[test]
    fn observation_does_not_equate_attachment_with_readiness() {
        let observation = serde_json::json!({
            "peer": "vellum",
            "status": "connect_refused"
        });
        let runtime = PeerRuntime::from_observation(&observation, false);
        assert_eq!(runtime.deployment, "attached");
        assert_eq!(runtime.lifecycle, "external");
        assert_eq!(runtime.observed_readiness, "not_ready");
        assert_eq!(runtime.revision, Value::Null);
    }

    #[test]
    fn shutdown_receipt_updates_only_the_exact_hosted_instance() {
        let first = PeerRuntime::from_observation(
            &serde_json::json!({"peer":"chronos", "instance":"first", "status":"current", "revision":"Finite(3)"}),
            true,
        );
        let second = PeerRuntime::from_observation(
            &serde_json::json!({"peer":"chronos", "instance":"second", "status":"current", "revision":"Finite(4)"}),
            true,
        );
        let receipt = TopologyShutdown {
            receipts: vec![ShutdownReceipt {
                peer: HostedPeerId::Chronos("second".into()),
                disposition: ShutdownDisposition::OutcomeUnknown,
            }],
        };
        let snapshot = LocalManagementSnapshot::after_shutdown(
            "restart-7".into(),
            vec![first, second],
            Ok(Some(&receipt)),
        );
        assert_eq!(snapshot.operation_id, "restart-7");
        assert_eq!(snapshot.peers[0].lifecycle, "running");
        assert_eq!(snapshot.peers[1].lifecycle, "shutdown_unknown");
        assert_eq!(snapshot.peers[1].certainty, "unknown");
    }

    #[test]
    fn uncertainty_and_peer_owned_fencing_remain_distinct() {
        let uncertain = PeerRuntime::from_observation(
            &serde_json::json!({"peer":"agora", "status":"outcome_unknown"}),
            false,
        );
        assert_eq!(uncertain.observed_readiness, "unknown");
        assert_eq!(uncertain.certainty, "unknown");
        assert_eq!(uncertain.fencing, "uncertain");

        let fenced = PeerRuntime::from_observation(
            &serde_json::json!({
                "peer":"moira", "status":"current", "census":{"fenced":2}
            }),
            false,
        );
        assert_eq!(fenced.observed_readiness, "ready");
        assert_eq!(fenced.certainty, "confirmed");
        assert_eq!(fenced.fencing, "workload_fenced");
        assert_eq!(fenced.fenced_work_items, Some(2));
    }
}
