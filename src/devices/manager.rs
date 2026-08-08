use std::time::Duration;

use btleplug::{
    api::{Characteristic, Peripheral, WriteType},
    platform::Peripheral as PlatformPeripheral,
};
use futures_util::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::{
    config::manager::save_device_to_config,
    protocol::{
        DATA_NOTIFY_CHARACTERISTICS, DATA_SERVICE_UUID, DATA_WRITE_CHARACTERISTICS,
        NOTIFY_CHARACTERISTICS, Request, Response, SERVICE_UUID, WRITE_CHARACTERISTICS,
        battery::{BatteryRequest, BatteryResponse},
        bigdata::{
            OxygenData, SleepData, make_data_request, parse_big_data_header, parse_oxygen_data,
            parse_sleep_data, DATA_REQUEST_ID_OXYGEN, DATA_REQUEST_ID_SLEEP,
        },
        blink::BlinkRequest,
        find::FindRequest,
        hr::{HeartRateLogParser, HeartRateRequest, HeartRateResult},
        reboot::RebootRequest,
        realtime::{
            RealtimeReading, RealtimeStartRequest, RealtimeStopRequest, ReadingType,
        },
        reset::ResetRequest,
        steps::{ActivityDetailParser, StepsRequest, StepsResult},
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

        let write_char = write_char.ok_or(ConnectionError::CharacteristicsNotFound)?;
        let notify_char = notify_char.ok_or(ConnectionError::CharacteristicsNotFound)?;

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

                        if crate::protocol::has_error_flag(packet) {
                            return Err(DeviceError::Protocol(
                                crate::error::ProtocolError::ErrorFlag {
                                    command_id: packet[0],
                                },
                            ));
                        }

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
    pub async fn get_battery_level(device: &Device) -> Result<BatteryResponse, DeviceError> {
        let (write_char, notify_char) = Self::connect_and_setup(device).await?;

        let peripheral = device.peripheral();

        let request = BatteryRequest::new();

        Self::write_request(peripheral, &write_char, request).await?;
        let response = Self::read_response::<BatteryResponse>(peripheral, &notify_char).await?;

        Ok(response)
    }

    pub async fn get_heart_rate_log(
        device: &Device,
        timestamp: u32,
    ) -> Result<HeartRateResult, DeviceError> {
        let (write_char, notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();

        Self::write_request(peripheral, &write_char, HeartRateRequest::new(timestamp)).await?;

        let mut parser = HeartRateLogParser::new();
        let result =
            Self::read_split_array(peripheral, &notify_char, |packet| parser.feed(packet)).await?;
        Ok(result)
    }

    pub async fn get_steps(
        device: &Device,
        day_offset: i8,
    ) -> Result<StepsResult, DeviceError> {
        let (write_char, notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();

        Self::write_request(peripheral, &write_char, StepsRequest::new(day_offset)).await?;

        let mut parser = ActivityDetailParser::new();
        let result =
            Self::read_split_array(peripheral, &notify_char, |packet| parser.feed(packet)).await?;
        Ok(result)
    }

    pub async fn get_sleep(device: &Device) -> Result<SleepData, DeviceError> {
        let (_write_char, _notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();

        let (data_write_char, data_notify_char) =
            Self::find_data_characteristics(peripheral).await?;

        let buffer = Self::read_big_data(
            peripheral,
            &data_write_char,
            &data_notify_char,
            DATA_REQUEST_ID_SLEEP,
        )
        .await?;

        Ok(parse_sleep_data(&buffer)?)
    }

    pub async fn get_oxygen(device: &Device) -> Result<OxygenData, DeviceError> {
        let (_write_char, _notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();

        let (data_write_char, data_notify_char) =
            Self::find_data_characteristics(peripheral).await?;

        let buffer = Self::read_big_data(
            peripheral,
            &data_write_char,
            &data_notify_char,
            DATA_REQUEST_ID_OXYGEN,
        )
        .await?;

        Ok(parse_oxygen_data(&buffer)?)
    }

    async fn find_data_characteristics(
        peripheral: &PlatformPeripheral,
    ) -> Result<(Characteristic, Characteristic), ConnectionError> {
        for service in peripheral.services() {
            if service.uuid.to_string() != DATA_SERVICE_UUID {
                continue;
            }

            let mut write_char = None;
            let mut notify_char = None;
            for char in service.characteristics {
                if char.uuid.to_string() == DATA_WRITE_CHARACTERISTICS {
                    write_char = Some(char);
                } else if char.uuid.to_string() == DATA_NOTIFY_CHARACTERISTICS {
                    notify_char = Some(char);
                }
            }

            if let (Some(write_char), Some(notify_char)) = (write_char, notify_char) {
                return Ok((write_char, notify_char));
            }
        }

        Err(ConnectionError::CharacteristicsNotFound)
    }

    async fn read_big_data(
        peripheral: &PlatformPeripheral,
        data_write_char: &Characteristic,
        data_notify_char: &Characteristic,
        data_id: u8,
    ) -> Result<Vec<u8>, DeviceError> {
        peripheral
            .subscribe(data_notify_char)
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        peripheral
            .write(
                data_write_char,
                &make_data_request(data_id),
                WriteType::WithoutResponse,
            )
            .await
            .map_err(|_| ConnectionError::WriteFailed)?;

        let mut notifications = peripheral
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let mut buffer: Vec<u8> = Vec::new();
        let mut first_packet = true;

        loop {
            let timeout_duration = if first_packet {
                Duration::from_millis(3000)
            } else {
                Duration::from_millis(400)
            };

            match timeout(timeout_duration, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == data_notify_char.uuid {
                        first_packet = false;
                        buffer.extend_from_slice(&notification.value);
                    }
                }
                Ok(None) => {
                    return Err(DeviceError::StreamEnded);
                }
                Err(_) => {
                    if buffer.is_empty() {
                        return Err(DeviceError::BigDataTimeout);
                    }
                    let (id, _) = parse_big_data_header(&buffer)?;
                    if id != data_id {
                        return Err(DeviceError::Protocol(
                            crate::error::ProtocolError::CommandId {
                                expected: data_id,
                                actual: id,
                            },
                        ));
                    }
                    return Ok(buffer);
                }
            }
        }
    }

    async fn read_split_array<T>(
        peripheral: &PlatformPeripheral,
        notify_char: &Characteristic,
        mut feed: impl FnMut(&[u8]) -> Result<Option<T>, crate::error::ProtocolError>,
    ) -> Result<T, DeviceError> {
        let mut notifications = peripheral
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let timeout_duration = Duration::from_millis(3000);

        loop {
            match timeout(timeout_duration, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == notify_char.uuid {
                        let packet = &notification.value;

                        if crate::protocol::has_error_flag(packet) {
                            return Err(DeviceError::Protocol(
                                crate::error::ProtocolError::ErrorFlag {
                                    command_id: packet[0],
                                },
                            ));
                        }

                        if let Some(result) = feed(packet)? {
                            return Ok(result);
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
        let (write_char, _notify_char) = Self::connect(device).await?;
        let write_char = write_char.ok_or(ConnectionError::CharacteristicsNotFound)?;
        let peripheral = device.peripheral();
        Self::write_request(peripheral, &write_char, request).await?;
        Ok(())
    }

    pub async fn stream_realtime(
        device: &Device,
        reading_type: ReadingType,
        duration: Duration,
        tx: mpsc::Sender<RealtimeReading>,
    ) -> Result<(), DeviceError> {
        let (write_char, notify_char) = Self::connect_and_setup(device).await?;
        let peripheral = device.peripheral();

        peripheral
            .write(
                &write_char,
                &make_phone_info_packet(),
                WriteType::WithoutResponse,
            )
            .await
            .map_err(|_| ConnectionError::WriteFailed)?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self::write_request(peripheral, &write_char, RealtimeStartRequest::new(reading_type))
            .await?;

        let mut notifications = peripheral
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let start = std::time::Instant::now();
        loop {
            let remaining = duration.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                break;
            }

            match timeout(remaining, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == notify_char.uuid
                        && let Ok(reading) = RealtimeReading::from_bytes(&notification.value)
                        && reading.reading_type == reading_type
                        && reading.value != 0
                        && tx.send(reading).await.is_err()
                    {
                        break;
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        let _ = Self::write_request(peripheral, &write_char, RealtimeStopRequest::new(reading_type))
            .await;

        Ok(())
    }
}

fn make_phone_info_packet() -> [u8; 16] {
    let mut packet = [0u8; 16];
    packet[0] = 0x07;
    packet[1] = 0x02;
    packet[2] = 0x0A;
    packet[3..15].copy_from_slice(b"colmi_client");
    packet[15] = crate::protocol::calculate_checksum(&packet);
    packet
}
