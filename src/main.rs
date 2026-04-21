use clap::Parser;
use rustforge::cli::{Cli, handle_command};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle command
    if let Err(e) = handle_command(cli.command).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
