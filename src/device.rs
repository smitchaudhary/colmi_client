const COLMI_MANUFACTURER_ID: u16 = 4660;

use btleplug::{
    api::{Characteristic, Peripheral, WriteType},
    platform::Peripheral as PlatformPeripheral,
};
use std::fmt::Display;

use crate::ble::battery::{
    BatteryRequest, BatteryResponse, NOTIFY_CHARACTERISTICS, SERVICE_UUID, WRITE_CHARACTERISTICS,
};
use crate::errors::ConnectionError;

#[derive(Clone)]
pub struct Device {
    peripheral: PlatformPeripheral,
    name: String,
    id: String,
    is_colmi_device: bool,
    write_characteristics: Option<Characteristic>,
    notify_characteristics: Option<Characteristic>,
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
            write_characteristics: None,
            notify_characteristics: None,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}, ({})", self.name, self.id)
    }

    pub fn peripheral(self) -> PlatformPeripheral {
        self.peripheral
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

    fn get_write_characteristics(&self) -> &Characteristic {
        &self.write_characteristics.as_ref().unwrap()
    }

    fn get_notify_characteristics(&self) -> &Characteristic {
        &self.notify_characteristics.as_ref().unwrap()
    }

    pub async fn connect(&mut self) -> Result<(), ConnectionError> {
        match self.peripheral.connect().await {
            Ok(_) => {
                for service in self.peripheral.services() {
                    if service.uuid.to_string() != SERVICE_UUID {
                        continue;
                    }

                    for char in service.characteristics {
                        if char.uuid.to_string() == NOTIFY_CHARACTERISTICS {
                            self.notify_characteristics = Some(char);
                        } else if char.uuid.to_string() == WRITE_CHARACTERISTICS {
                            self.write_characteristics = Some(char);
                        }
                    }
                }
                if self.notify_characteristics.is_none() || self.write_characteristics.is_none() {
                    Err(ConnectionError::CharacteristicsNotFound)
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(ConnectionError::ConnectionFailed),
        }
    }

    pub async fn write(&self, request: BatteryRequest) {
        self.peripheral
            .write(
                self.get_write_characteristics(),
                &request.as_bytes(),
                WriteType::WithoutResponse,
            )
            .await
            .unwrap();
    }

    pub async fn read(&self) -> BatteryResponse {
        let result = self
            .peripheral
            .read(self.get_notify_characteristics())
            .await
            .unwrap();
        BatteryResponse::from_bytes(result)
    }

    pub async fn subscribe(&self) {
        self.peripheral
            .subscribe(self.get_notify_characteristics())
            .await
            .unwrap();
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
