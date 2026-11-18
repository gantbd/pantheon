//! Pantheon application lifecycle for interactive and headless composition.
#![forbid(unsafe_code)]

pub mod capabilities;
pub mod headless;
mod hosted;
pub mod management;
pub mod native_host;
pub mod profile;
pub mod reconfiguration;
pub mod registry;
mod terminal;

pub use hosted::{run_hosted_terminal, run_hosted_terminal_with_agora};
pub use terminal::{
    AgoraControlConfig, ManagedTerminalOutcome, MoiraAgentWorkerConfig, OperationalPeers,
    RestartProfileRequest, VellumControlConfig, run_multi_peer_terminal, run_profiled_terminal,
    run_terminal, run_terminal_with_all_peers, run_terminal_with_all_peers_managed,
    run_terminal_with_all_peers_managed_configuration, run_terminal_with_moira,
    run_terminal_with_operational_peers,
};
