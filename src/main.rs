mod ble;
mod errors;
use btleplug::api::Peripheral;
use clap::{Parser, Subcommand};
use inquire::Select;

use crate::errors::BleError;

#[derive(Parser)]
#[command(name = "colmi_client")]
#[command(about = "A CLI Client to interact with Colmi Bluetooth devices")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(long)]
        all: bool,
    },

    Connect {
        #[arg(long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { all } => match scan_devices(!all).await {
            Ok(devices) => {
                println!("Found {} device(s):", devices.len());
                for (i, (name, _)) in devices.iter().enumerate() {
                    println!("  {}. {}", i + 1, name);
                }
            }
            Err(err) => err.display(all),
        },
        Commands::Connect { all } => match scan_devices(!all).await {
            Ok(devices) => {
                //
            }
            Err(err) => err.display(all),
        },
    }
}

async fn scan_devices(filter_colmi: bool) -> Result<Vec<(String, impl Peripheral)>, BleError> {
    let devices = ble::scan_for_devices().await.map_err(|e| {
        if let Some(error) = e.downcast_ref::<BleError>() {
            *error
        } else {
            BleError::OperationFailed
        }
    })?;

    let mut device_info = Vec::new();
    for device in devices {
        if let Ok(Some(props)) = device.properties().await {
            let is_colmi = props.manufacturer_data.contains_key(&4660);
            if !filter_colmi || is_colmi {
                let name = props
                    .local_name
                    .unwrap_or_else(|| "Unknown Device".to_string());
                device_info.push((name, device));
            }
        }
    }

    if device_info.is_empty() {
        return Err(BleError::NoDevices);
    }

    Ok(device_info)
}
