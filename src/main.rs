mod ble;
mod errors;
use btleplug::api::Peripheral;
use inquire::Select;

use crate::errors::BleError;

#[tokio::main]
async fn main() {
    println!("Starting Bluetooth device scan...");

    match ble::scan_for_devices().await {
        Ok(devices) => {
            let mut device_info = Vec::new();
            for device in devices {
                if let Ok(Some(props)) = device.properties().await {
                    let name = props
                        .local_name
                        .unwrap_or_else(|| "Unknown Device".to_string());
                    device_info.push((name, device));
                }
            }

            let names = device_info
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<String>>();

            let device = Select::new("Select the device you want to connect to!", names)
                .prompt()
                .unwrap();
        }
        Err(err) => {
            if let Some(err) = err.downcast_ref::<BleError>() {
                match err {
                    BleError::NoAdapters => {
                        println!("No Bluetooth adapters found on this device!");
                        println!("Please ensure bluetooth is turned on bluetooth on this device.")
                    }
                    BleError::NoDevices => {
                        println!("Did not find any compatible device!");
                        println!("Please ensure the Colmi device is turned on and in range.");
                    }
                }
            } else {
                println!("An error occurred: {:?}", err);
            }
        }
    }
}
