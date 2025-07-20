mod ble;
mod errors;
use btleplug::api::Peripheral;

use crate::errors::BleError;

#[tokio::main]
async fn main() {
    println!("Starting Bluetooth device scan...");

    match ble::scan_for_devices().await {
        Ok(devices) => {
            for device in devices {
                let properties = device.properties().await.unwrap();

                if let Some(props) = properties {
                    println!("  Address: {}", props.address);
                    println!("  Local name: {:?}", props.local_name);
                    println!("  Manufacturer data: {:?}", props.manufacturer_data);
                    println!("  RSSI: {:?}", props.rssi);
                    println!();
                }
            }
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
