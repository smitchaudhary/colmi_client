const COLMI_MANUFACTURER_ID: u16 = 4660;

use btleplug::{api::Peripheral, platform::Peripheral as PlatformPeripheral};
use std::fmt::Display;

#[derive(Clone)]
pub struct Device {
    peripheral: PlatformPeripheral,
    name: String,
    id: String,
    is_colmi_device: bool,
}

impl Device {
    pub async fn new(peripheral: PlatformPeripheral) -> Self {
        let props = peripheral
            .properties()
            .await
            .unwrap()
            .expect("Failed to retrieve device properties");

        let name = props.local_name.unwrap_or("Unknown Device".to_string());
        let id = peripheral.id().to_string();
        let is_colmi_device = props.manufacturer_data.contains_key(&COLMI_MANUFACTURER_ID);

        Self {
            peripheral,
            name,
            id,
            is_colmi_device,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}, ({})", self.name, self.id)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_colmi_device(&self) -> bool {
        self.is_colmi_device
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
