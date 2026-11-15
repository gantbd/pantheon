//! Workbench application lifecycle for interactive and headless composition.
#![forbid(unsafe_code)]

pub mod headless;
mod hosted;
pub mod native_host;
pub mod profile;
pub mod registry;
mod terminal;

pub use hosted::{run_hosted_terminal, run_hosted_terminal_with_agora};
pub use terminal::{
    run_multi_peer_terminal, run_profiled_terminal, run_terminal, run_terminal_with_all_peers,
    run_terminal_with_moira, run_terminal_with_operational_peers,
};
