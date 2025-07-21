use crate::ble;
use crate::config::save_device_to_config;
use crate::device::Device;
use crate::errors::ScanError;
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
                match selected_device.connect().await {
                    Ok(_) => {
                        println!("Connected to device {}", selected_device);
                        save_device_to_config(selected_device);
                    }
                    Err(err) => err.display(),
                }
            }
        }
        Err(err) => err.display(!filter_colmi),
    }
}

async fn filter_devices(filter_colmi: bool) -> Result<Vec<Device>, ScanError> {
    let devices = ble::scan_for_devices().await.map_err(|e| {
        if let Some(error) = e.downcast_ref::<ScanError>() {
            *error
        } else {
            ScanError::OperationFailed
        }
    })?;

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
