use crate::errors::BleError;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

pub async fn scan_for_devices() -> Result<Vec<impl Peripheral>, Box<dyn std::error::Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        println!("No Bluetooth adapters found");
        return Err(Box::new(BleError::NoAdapters));
    }

    let mut devices: Vec<_> = Vec::new();

    for adapter in adapters {
        println!(
            "Starting scan on adapter: {:?}",
            adapter.adapter_info().await?
        );

        adapter.start_scan(ScanFilter::default()).await?;

        time::sleep(Duration::from_secs(10)).await;

        let peripherals = adapter.peripherals().await?;

        for peripheral in peripherals {
            if let Ok(Some(props)) = peripheral.properties().await {
                if props.manufacturer_data.contains_key(&4660) {
                    devices.push(peripheral);
                }
            }
        }
    }

    if devices.is_empty() {
        return Err(Box::new(BleError::NoDevices));
    }

    Ok(devices)
}
