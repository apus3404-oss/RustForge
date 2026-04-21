// src/cli/mod.rs
pub mod commands;
pub mod handlers;

pub use commands::{Cli, Commands};
pub use handlers::handle_command;
