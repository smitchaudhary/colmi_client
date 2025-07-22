use crate::bluetooth::scanner;
use crate::config::manager::save_device_to_config;
use crate::devices::manager::DeviceManager;
use crate::devices::models::Device;
use crate::error::ScanError;
use crate::protocol::battery::BatteryRequest;
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

                println!("Connected to device {}", selected_device);
                save_device_to_config(selected_device);
            }
        }
        Err(err) => err.display(!filter_colmi),
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

                DeviceManager::subscribe_to_notifications(&peripheral, &notify_char).await;

                DeviceManager::write_battery_request(
                    &peripheral,
                    &write_char,
                    BatteryRequest::new(),
                )
                .await;

                let response =
                    DeviceManager::read_battery_response(&peripheral, &notify_char).await;

                println!("{}", response);
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
