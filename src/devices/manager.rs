use std::time::Duration;

use btleplug::{
    api::{Characteristic, Peripheral, WriteType},
    platform::Peripheral as PlatformPeripheral,
};
use futures_util::stream::StreamExt;
use tokio::time::timeout;

use crate::devices::models::Device;
use crate::error::{ConnectionError, DeviceError};
use crate::protocol::{
    NOTIFY_CHARACTERISTICS, Request, Response, SERVICE_UUID, WRITE_CHARACTERISTICS,
};

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
    ) -> Result<(), ConnectionError> {
        device
            .write(write_char, &request.as_bytes(), WriteType::WithoutResponse)
            .await
            .map_err(|_| ConnectionError::WriteFailed)?;
        Ok(())
    }

    pub async fn read_response<R: Response>(
        device: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) -> Result<R, DeviceError> {
        let reading = device
            .read(notify_char)
            .await
            .map_err(|_| ConnectionError::ReadFailed)?;
        let result = R::from_bytes(reading)?;
        Ok(result)
    }

    pub async fn read_response_stream<R: Response>(
        device: &PlatformPeripheral,
        notify_char: &Characteristic,
        expected_command_id: u8,
        timeout_ms: u64,
    ) -> Result<R, DeviceError> {
        let mut notifications = device
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let timeout_duration = Duration::from_millis(timeout_ms);

        loop {
            match timeout(timeout_duration, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == notify_char.uuid {
                        let packet = &notification.value;

                        if packet[0] == expected_command_id {
                            let response = R::from_bytes(packet.clone())?;
                            return Ok(response);
                        } else if packet[0] == 0x2f {
                            println!("Got the notification packet!");
                            continue;
                        } else {
                            println!("Got an unexpected packet!");
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    return Err(DeviceError::Timeout);
                }
                Err(_) => {
                    return Err(DeviceError::Timeout);
                }
            }
        }
    }

    pub async fn subscribe_to_notifications(
        device: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) -> Result<(), ConnectionError> {
        device
            .subscribe(notify_char)
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;
        Ok(())
    }
}
