mod bluetooth;
mod cli;
mod config;
mod devices;
mod error;
mod protocol;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { all } => cli::commands::scan(!all).await,
        Commands::Connect { all } => cli::commands::connect(!all).await,
        Commands::Battery => cli::commands::battery().await,
        Commands::Features => cli::commands::features().await,
    }
}
