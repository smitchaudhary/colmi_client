use clap::{Parser, Subcommand};

pub mod commands;

#[derive(Parser)]
#[command(name = "colmi_client")]
#[command(about = "A CLI tool for interacting with Colmi Bluetooth devices")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Scan {
        #[arg(long)]
        all: bool,
    },
    Connect {
        #[arg(long)]
        all: bool,
    },
    Battery,
    Blink,
    Reset,
    Reboot,
    Find,
    Hr {
        #[arg(long, default_value_t = 1)]
        days: u32,
    },
    Tui,
}
