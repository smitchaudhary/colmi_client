use crate::ble;
use crate::device::colmi;
use crate::errors::BleError;
use crate::tui;
use btleplug::api::Peripheral;

pub async fn scan(filter_colmi: bool) {
    match scan_devices(filter_colmi).await {
        Ok(devices_info) => {
            println!("Found {} device(s):", devices_info.len());
            for (i, (display_name, _)) in devices_info.iter().enumerate() {
                println!("  {}. {}", i + 1, display_name);
            }
        }
        Err(err) => err.display(!filter_colmi),
    }
}

pub async fn connect(filter_colmi: bool) {
    match scan_devices(filter_colmi).await {
        Ok(devices_info) => {
            println!("Found {} device(s):", devices_info.len());

            if let Some(index) = tui::select_device(&devices_info) {
                println!("The user has chosen device number: {}", index);
            }
        }
        Err(err) => err.display(!filter_colmi),
    }
}

async fn scan_devices(
    filter_colmi: bool,
) -> Result<Vec<(String, impl btleplug::api::Peripheral)>, BleError> {
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
            let is_colmi = colmi::is_colmi_device(&props.manufacturer_data);
            if !filter_colmi || is_colmi {
                let display_name =
                    colmi::format_device_info(props.local_name, props.address.to_string());
                device_info.push((display_name, device));
            }
        }
    }

    if device_info.is_empty() {
        return Err(BleError::NoDevices);
    }

    Ok(device_info)
}
