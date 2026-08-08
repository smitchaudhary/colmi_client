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
    Info,
    Blink,
    Reset,
    Reboot,
    Find,
    Hr {
        #[arg(long, default_value_t = 1)]
        days: u32,
    },
    Steps {
        #[arg(long, default_value_t = 1)]
        days: u32,
    },
    Sleep,
    Spo2,
    /// Stream live readings for a few seconds.
    Realtime {
        /// Reading type: hr, spo2 or hrv.
        #[arg(long, default_value = "hr")]
        r#type: String,
        /// Stream duration in seconds (values appear after ~30s warm-up).
        #[arg(long, default_value_t = 60)]
        seconds: u64,
    },
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    },
    Tui,
}

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Heart-rate logging settings.
    Hr {
        #[arg(long)]
        enable: bool,
        #[arg(long)]
        disable: bool,
        #[arg(long)]
        interval: Option<u8>,
    },
}
