//! Pantheon application lifecycle for interactive and headless composition.
#![forbid(unsafe_code)]

pub mod capabilities;
pub mod headless;
pub mod hestia;
mod hosted;
pub mod management;
pub mod native_host;
pub mod profile;
mod provider;
pub mod reconfiguration;
pub mod registry;
pub mod task_agent;
mod terminal;

pub use hosted::{run_hosted_terminal, run_hosted_terminal_with_agora};
pub use terminal::{
    AgoraControlConfig, ManagedTerminalOutcome, MoiraAgentWorkerConfig,
    MoiraDecisionPacketComponent, MoiraHumanGateResponseConfig, OperationalPeers,
    RestartProfileRequest, VellumControlConfig, run_multi_peer_terminal, run_profiled_terminal,
    run_terminal, run_terminal_with_all_peers, run_terminal_with_all_peers_managed,
    run_terminal_with_all_peers_managed_configuration, run_terminal_with_moira,
    run_terminal_with_operational_peers,
};
