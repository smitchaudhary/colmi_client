use std::time::Duration;

use btleplug::{
    api::{Characteristic, Peripheral, WriteType},
    platform::Peripheral as PlatformPeripheral,
};
use futures_util::stream::StreamExt;
use tokio::time::timeout;

use crate::{
    config::manager::save_device_to_config,
    protocol::{
        NOTIFY_CHARACTERISTICS, Request, Response, SERVICE_UUID, WRITE_CHARACTERISTICS,
        blink::BlinkRequest, find::FindRequest, reboot::RebootRequest, reset::ResetRequest,
    },
};
use crate::{devices::models::Device, protocol::features::FeatureResponse};
use crate::{
    error::{ConnectionError, DeviceError},
    protocol::features::FeatureRequest,
};

pub struct DeviceManager;

impl DeviceManager {
    pub async fn connect_and_setup(
        device: &Device,
    ) -> Result<(Characteristic, Characteristic), DeviceError> {
        let (write_char, notify_char) = Self::connect(device).await?;

        let write_char = write_char.expect("Write characteristic not found");
        let notify_char = notify_char.expect("Notify characteristic not found");

        let peripheral = device.peripheral();

        Self::subscribe_to_notifications(peripheral, &notify_char).await?;

        let request = FeatureRequest::new();

        Self::write_request(peripheral, &write_char, request).await?;
        let features =
            Self::read_response_stream::<FeatureResponse>(peripheral, &notify_char, 1, 1000)
                .await?;

        save_device_to_config(device.clone(), features);

        Ok((write_char, notify_char))
    }

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
        peripheral: &PlatformPeripheral,
        write_char: &Characteristic,
        request: impl Request,
    ) -> Result<(), ConnectionError> {
        peripheral
            .write(write_char, &request.as_bytes(), WriteType::WithoutResponse)
            .await
            .map_err(|_| ConnectionError::WriteFailed)?;
        Ok(())
    }

    pub async fn read_response<R: Response>(
        peripheral: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) -> Result<R, DeviceError> {
        let reading = peripheral
            .read(notify_char)
            .await
            .map_err(|_| ConnectionError::ReadFailed)?;
        let result = R::from_bytes(reading)?;
        Ok(result)
    }

    pub async fn read_response_stream<R: Response>(
        peripheral: &PlatformPeripheral,
        notify_char: &Characteristic,
        expected_command_id: u8,
        timeout_ms: u64,
    ) -> Result<R, DeviceError> {
        let mut notifications = peripheral
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
                        } else {
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    return Err(DeviceError::StreamEnded);
                }
                Err(err) => {
                    return Err(DeviceError::Timeout(err));
                }
            }
        }
    }

    pub async fn subscribe_to_notifications(
        peripheral: &PlatformPeripheral,
        notify_char: &Characteristic,
    ) -> Result<(), ConnectionError> {
        peripheral
            .subscribe(notify_char)
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;
        Ok(())
    }
}

impl DeviceManager {
    pub async fn blink(device: &Device) -> Result<(), DeviceError> {
        Self::execute_device_control_command(device, BlinkRequest::new()).await
    }

    pub async fn reboot(device: &Device) -> Result<(), DeviceError> {
        Self::execute_device_control_command(device, RebootRequest::new()).await
    }

    pub async fn find(device: &Device) -> Result<(), DeviceError> {
        Self::execute_device_control_command(device, FindRequest::new()).await
    }

    pub async fn reset(device: &Device) -> Result<(), DeviceError> {
        Self::execute_device_control_command(device, ResetRequest::new()).await
    }

    async fn execute_device_control_command(
        device: &Device,
        request: impl Request,
    ) -> Result<(), DeviceError> {
        let (write_char, _notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();
        Self::write_request(peripheral, &write_char, request).await?;
        Ok(())
    }
}
