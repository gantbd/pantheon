//! Pantheon application lifecycle for interactive and headless composition.
#![forbid(unsafe_code)]

pub mod agent_web;
pub mod agora_observe;
pub mod agora_publish;
pub mod capabilities;
pub mod chronos_task;
pub mod daimon_observe;
pub mod headless;
pub mod hestia;
pub mod hestia_observe;
mod hosted;
pub mod management;
pub mod moira_deliberation;
pub mod native_host;
pub mod profile;
mod provider;
pub mod reconfiguration;
pub mod registry;
pub mod task_agent;
pub mod tellus_sme;
mod terminal;
pub mod vellum_document;

pub use hosted::{run_hosted_terminal, run_hosted_terminal_with_agora};
pub use terminal::{
    AgoraControlConfig, ManagedTerminalOutcome, MoiraAgentWorkerConfig,
    MoiraDecisionPacketComponent, MoiraHumanGateResponseConfig, OperationalPeers,
    RestartProfileRequest, VellumControlConfig, run_multi_peer_terminal, run_profiled_terminal,
    run_terminal, run_terminal_with_all_peers, run_terminal_with_all_peers_managed,
    run_terminal_with_all_peers_managed_configuration, run_terminal_with_moira,
    run_terminal_with_operational_peers,
};
