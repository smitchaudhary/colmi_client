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
        Commands::Info => cli::commands::info().await,
        Commands::Blink => cli::commands::blink().await,
        Commands::Reset => cli::commands::reset().await,
        Commands::Reboot => cli::commands::reboot().await,
        Commands::Find => cli::commands::find().await,
        Commands::Hr { days } => cli::commands::hr(days).await,
        Commands::Steps { days } => cli::commands::steps(days).await,
        Commands::Sleep => cli::commands::sleep().await,
        Commands::Spo2 => cli::commands::spo2().await,
        Commands::Realtime { r#type, seconds } => {
            cli::commands::realtime(&r#type, seconds).await
        }
        Commands::Settings { command } => match command {
            cli::SettingsCommands::Hr {
                enable,
                disable,
                interval,
            } => cli::commands::settings_hr(enable, disable, interval).await,
        },
        Commands::Tui => {
            if let Err(err) = tui::run_tui().await {
                eprintln!("TUI Error: {}", err);
            }
        }
    }
}
