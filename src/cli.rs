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
    Features,
}
