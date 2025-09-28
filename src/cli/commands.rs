use inquire::Confirm;

use crate::bluetooth::scanner;
use crate::devices::manager::DeviceManager;
use crate::devices::models::Device;
use crate::error::ScanError;
use crate::tui;

pub async fn scan(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());
            for (i, device) in devices.iter().enumerate() {
                println!("  {}. {}", i + 1, device.display_name());
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn connect(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::connect_and_setup(&selected_device).await {
                    Ok(_) => println!("Connected and configured device: {}", selected_device),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn battery() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::get_battery_level(&selected_device).await {
                    Ok(response) => println!("{}", response),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn blink() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::blink(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn reset() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match Confirm::new("This will reset the device. Continue?")
                    .with_default(false)
                    .prompt()
                {
                    Ok(true) => match DeviceManager::reset(&selected_device).await {
                        Ok(_) => (),
                        Err(err) => {
                            println!("{}", err);
                        }
                    },
                    Ok(false) => {
                        println!("Reset cancelled.");
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn reboot() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::reboot(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn find() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::find(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

async fn filter_devices(filter_colmi: bool) -> Result<Vec<Device>, ScanError> {
    let devices = scanner::scan_for_devices().await?;

    let filtered_devices = if filter_colmi {
        devices
            .into_iter()
            .filter(|d| d.is_colmi_device())
            .collect::<Vec<Device>>()
    } else {
        devices
    };

    if filtered_devices.is_empty() {
        return Err(if filter_colmi {
            ScanError::NoColmiDevices
        } else {
            ScanError::NoDevices
        });
    }

    Ok(filtered_devices)
}
