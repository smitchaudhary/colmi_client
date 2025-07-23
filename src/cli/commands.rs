use crate::bluetooth::scanner;
use crate::config::manager::save_device_to_config;
use crate::devices::manager::DeviceManager;
use crate::devices::models::Device;
use crate::error::ScanError;
use crate::protocol::battery::{BatteryRequest, BatteryResponse};
use crate::protocol::features::{FeatureRequest, FeatureResponse};
use crate::tui;

pub async fn scan(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());
            for (i, device) in devices.iter().enumerate() {
                println!("  {}. {}", i + 1, device.display_name());
            }
        }
        Err(err) => err.display(!filter_colmi),
    }
}

pub async fn connect(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                let (write_char, notify_char) = match DeviceManager::connect(&selected_device).await
                {
                    Ok(chars) => chars,
                    Err(err) => {
                        err.display();
                        return;
                    }
                };

                let write_char = write_char.expect("Write characteristic not found");
                let notify_char = notify_char.expect("Notify characteristic not found");

                println!("Connected to device {}", selected_device);

                let peripheral = selected_device.peripheral();

                match DeviceManager::subscribe_to_notifications(&peripheral, &notify_char).await {
                    Ok(_) => (),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }

                match DeviceManager::write_request(&peripheral, &write_char, FeatureRequest::new())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }

                match DeviceManager::read_response::<FeatureResponse>(&peripheral, &notify_char)
                    .await
                {
                    Ok(features) => save_device_to_config(selected_device, features),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }
            }
        }
        Err(err) => err.display(filter_colmi),
    }
}

pub async fn battery() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                let (write_char, notify_char) = match DeviceManager::connect(&selected_device).await
                {
                    Ok(chars) => chars,
                    Err(err) => {
                        err.display();
                        return;
                    }
                };

                let write_char = write_char.expect("Write characteristic not found");
                let notify_char = notify_char.expect("Notify characteristic not found");

                println!("Connected to device {}", selected_device);

                let peripheral = selected_device.peripheral();

                match DeviceManager::subscribe_to_notifications(&peripheral, &notify_char).await {
                    Ok(_) => (),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }

                match DeviceManager::write_request(&peripheral, &write_char, BatteryRequest::new())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }

                match DeviceManager::read_response::<BatteryResponse>(&peripheral, &notify_char)
                    .await
                {
                    Ok(response) => println!("{}", response),
                    Err(err) => {
                        err.display();
                        return;
                    }
                }
            }
        }
        Err(err) => err.display(true),
    }
}

async fn filter_devices(filter_colmi: bool) -> Result<Vec<Device>, ScanError> {
    let devices = scanner::scan_for_devices()
        .await
        .map_err(|_| ScanError::OperationFailed)?;

    let filtered_devices = if filter_colmi {
        devices
            .into_iter()
            .filter(|d| d.is_colmi_device())
            .collect::<Vec<Device>>()
    } else {
        devices
    };

    if filtered_devices.is_empty() {
        return Err(ScanError::NoDevices);
    }

    Ok(filtered_devices)
}
