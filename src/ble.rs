use crate::device::Device;
use crate::errors::ScanError;
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

pub mod battery;

pub async fn scan_for_devices() -> Result<Vec<Device>, Box<dyn std::error::Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        return Err(Box::new(ScanError::NoAdapters));
    }

    let mut devices: Vec<_> = Vec::new();

    for adapter in adapters {
        adapter.start_scan(ScanFilter::default()).await?;
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await?;

        for peripheral in peripherals {
            let device = Device::new(peripheral).await;
            devices.push(device);
        }
    }

    if devices.is_empty() {
        return Err(Box::new(ScanError::NoDevices));
    }

    Ok(devices)
}
