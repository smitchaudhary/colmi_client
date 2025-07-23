use btleplug::{
    api::{Characteristic, Peripheral, WriteType},
    platform::Peripheral as PlatformPeripheral,
};

use crate::devices::models::Device;
use crate::error::{ConnectionError, ProtocolError};
use crate::protocol::{Request, Response};
use crate::protocol::{NOTIFY_CHARACTERISTICS, SERVICE_UUID, WRITE_CHARACTERISTICS};

pub struct DeviceManager;

impl DeviceManager {
    pub async fn connect(
        device: &Device,
    ) -> Result<(Option<Characteristic>, Option<Characteristic>), ConnectionError> {
        match device.peripheral.connect().await {
            Ok(_) => {
                let mut write_characteristics = None;
                let mut notify_characteristics = None;

                for service in device.peripheral.services() {
                    if service.uuid.to_string() != SERVICE_UUID {
                        continue;
                    }

                    for char in service.characteristics {
                        if char.uuid.to_string() == NOTIFY_CHARACTERISTICS {
                            notify_characteristics = Some(char);
                        } else if char.uuid.to_string() == WRITE_CHARACTERISTICS {
                            write_characteristics = Some(char);
                        }
                    }
                }

                if notify_characteristics.is_none() || write_characteristics.is_none() {
                    Err(ConnectionError::CharacteristicsNotFound)
                } else {
                    Ok((write_characteristics, notify_characteristics))
                }
            }
            Err(_) => Err(ConnectionError::ConnectionFailed),
        }
    }

    pub async fn write_request(
        device: &PlatformPeripheral,
        write_char: &Characteristic,
        request: impl Request,
    ) {
        device
            .write(write_char, &request.as_bytes(), WriteType::WithoutResponse)
            .await
            .unwrap();
    }

    pub async fn read_response<R: Response>(
        device: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) -> Result<R, ProtocolError> {
        let result = device.read(notify_char).await.unwrap();
        R::from_bytes(result)
    }

    pub async fn subscribe_to_notifications(
        device: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) {
        device.subscribe(notify_char).await.unwrap();
    }
}
