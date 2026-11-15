//! Workbench application lifecycle for interactive and headless composition.
#![forbid(unsafe_code)]

mod hosted;
pub mod profile;
pub mod registry;
mod terminal;

pub use hosted::{run_hosted_terminal, run_hosted_terminal_with_agora};
pub use terminal::{run_multi_peer_terminal, run_profiled_terminal, run_terminal};
