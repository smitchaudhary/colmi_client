pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "colmi_client")]
#[command(about = "A CLI Client to interact with Colmi Bluetooth devices")]
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
}