use std::fs;

use serde::{Deserialize, Serialize};

use crate::device::Device;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    title: String,
    device_config: DeviceConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DeviceConfig {
    name: Option<String>,
    address: Option<String>,
}

pub fn save_device_address(device: Device) {
    let config = Config {
        title: "Config for Colmi Client".to_string(),
        device_config: DeviceConfig {
            name: Some(device.name().to_string()),
            address: Some(device.address().to_string()),
        },
    };

    let toml_string = toml::to_string(&config).unwrap();
    fs::write("config.toml", toml_string).unwrap();
}
