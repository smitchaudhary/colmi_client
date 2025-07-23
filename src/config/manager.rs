use std::fs;

use serde::{Deserialize, Serialize};

use crate::devices::models::Device;
use crate::protocol::features::FeatureResponse;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    title: String,
    device_config: DeviceConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DeviceConfig {
    name: Option<String>,
    address: Option<String>,
    features: Option<FeatureResponse>,
}

pub fn save_device_to_config(device: Device, features: FeatureResponse) {
    let config = Config {
        title: "Config for Colmi Client".to_string(),
        device_config: DeviceConfig {
            name: Some(device.name().to_string()),
            address: Some(device.id().to_string()),
            features: Some(features),
        },
    };

    let toml_string = toml::to_string(&config).unwrap();
    fs::write("config.toml", toml_string).unwrap();
}
