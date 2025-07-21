mod ble;
mod cli;
mod config;
mod device;
mod errors;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { all } => cli::commands::scan(!all).await,
        Commands::Connect { all } => cli::commands::connect(!all).await,
    }
}
