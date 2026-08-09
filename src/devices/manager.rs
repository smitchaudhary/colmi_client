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
        DEVICE_INFO_FIRMWARE_UUID, DEVICE_INFO_HARDWARE_UUID, DEVICE_INFO_MANUFACTURER_UUID,
        DEVICE_INFO_SERVICE_UUID, NOTIFY_CHARACTERISTICS, Request, Response, SERVICE_UUID,
        WRITE_CHARACTERISTICS,
        battery::{BatteryRequest, BatteryResponse},
        bigdata::{
            DATA_REQUEST_ID_OXYGEN, DATA_REQUEST_ID_SLEEP, OxygenData, SleepData,
            make_data_request, parse_big_data_header, parse_oxygen_data, parse_sleep_data,
        },
        blink::BlinkRequest,
        find::FindRequest,
        hr::{HeartRateLogParser, HeartRateRequest, HeartRateResult},
        realtime::{ReadingType, RealtimeReading, RealtimeStartRequest, RealtimeStopRequest},
        reboot::RebootRequest,
        reset::ResetRequest,
        settings::{CMD_HEART_RATE_LOG_SETTINGS, HeartRateLogSettings, SettingsRequest},
        steps::{ActivityDetailParser, StepsRequest, StepsResult},
    },
};
use crate::{devices::models::Device, protocol::features::FeatureResponse};
use crate::{
    error::{ConnectionError, DeviceError},
    protocol::features::FeatureRequest,
};

#[derive(Clone)]
pub struct Connection {
    pub peripheral: PlatformPeripheral,
    pub write_char: Characteristic,
    pub notify_char: Characteristic,
}

pub struct DeviceManager;

impl DeviceManager {
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
    const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

    pub async fn connect_and_setup(device: &Device) -> Result<Connection, DeviceError> {
        let conn = match tokio::time::timeout(Self::CONNECT_TIMEOUT, Self::connect(device)).await {
            Ok(result) => result?,
            Err(err) => return Err(DeviceError::Timeout(err)),
        };

        Self::subscribe_to_notifications(&conn).await?;

        let request = FeatureRequest::new();

        Self::write_request(&conn, request).await?;
        let features = Self::read_response_stream::<FeatureResponse>(&conn, 1, 1000).await?;

        save_device_to_config(device.clone(), features);

        Ok(conn)
    }

    pub async fn connect(device: &Device) -> Result<Connection, ConnectionError> {
        match device.peripheral.connect().await {
            Ok(_) => {
                let mut write_char = None;
                let mut notify_char = None;

                for service in device.peripheral.services() {
                    if service.uuid.to_string() != SERVICE_UUID {
                        continue;
                    }

                    for char in service.characteristics {
                        if char.uuid.to_string() == NOTIFY_CHARACTERISTICS {
                            notify_char = Some(char);
                        } else if char.uuid.to_string() == WRITE_CHARACTERISTICS {
                            write_char = Some(char);
                        }
                    }
                }

                match (write_char, notify_char) {
                    (Some(write_char), Some(notify_char)) => Ok(Connection {
                        peripheral: device.peripheral().clone(),
                        write_char,
                        notify_char,
                    }),
                    _ => Err(ConnectionError::CharacteristicsNotFound),
                }
            }
            Err(_) => Err(ConnectionError::ConnectionFailed),
        }
    }

    pub async fn write_request(
        conn: &Connection,
        request: impl Request,
    ) -> Result<(), ConnectionError> {
        Self::write_with_timeout(&conn.peripheral, &conn.write_char, &request.as_bytes()).await
    }

    async fn write_with_timeout(
        peripheral: &PlatformPeripheral,
        write_char: &Characteristic,
        bytes: &[u8],
    ) -> Result<(), ConnectionError> {
        match tokio::time::timeout(
            Self::WRITE_TIMEOUT,
            peripheral.write(write_char, bytes, WriteType::WithoutResponse),
        )
        .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) | Err(_) => Err(ConnectionError::WriteFailed),
        }
    }

    pub async fn read_response<R: Response>(conn: &Connection) -> Result<R, DeviceError> {
        let reading = conn
            .peripheral
            .read(&conn.notify_char)
            .await
            .map_err(|_| ConnectionError::ReadFailed)?;
        let result = R::from_bytes(reading)?;
        Ok(result)
    }

    pub async fn read_response_stream<R: Response>(
        conn: &Connection,
        expected_command_id: u8,
        timeout_ms: u64,
    ) -> Result<R, DeviceError> {
        let mut notifications = conn
            .peripheral
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let timeout_duration = Duration::from_millis(timeout_ms);

        loop {
            match timeout(timeout_duration, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == conn.notify_char.uuid {
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

    pub async fn subscribe_to_notifications(conn: &Connection) -> Result<(), ConnectionError> {
        conn.peripheral
            .subscribe(&conn.notify_char)
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;
        Ok(())
    }
}

impl DeviceManager {
    pub async fn get_battery_level(conn: &Connection) -> Result<BatteryResponse, DeviceError> {
        Self::write_request(conn, BatteryRequest::new()).await?;
        let response = Self::read_response::<BatteryResponse>(conn).await?;
        Ok(response)
    }

    pub async fn get_heart_rate_log(
        conn: &Connection,
        timestamp: u32,
    ) -> Result<HeartRateResult, DeviceError> {
        Self::write_request(conn, HeartRateRequest::new(timestamp)).await?;

        let mut parser = HeartRateLogParser::new();
        let result = Self::read_split_array(conn, |packet| parser.feed(packet)).await?;
        Ok(result)
    }

    pub async fn get_steps(conn: &Connection, day_offset: i8) -> Result<StepsResult, DeviceError> {
        Self::write_request(conn, StepsRequest::new(day_offset)).await?;

        let mut parser = ActivityDetailParser::new();
        let result = Self::read_split_array(conn, |packet| parser.feed(packet)).await?;
        Ok(result)
    }

    pub async fn get_device_info(
        conn: &Connection,
    ) -> Result<(String, String, String), DeviceError> {
        let firmware =
            Self::read_device_info_string(&conn.peripheral, DEVICE_INFO_FIRMWARE_UUID).await;
        let hardware =
            Self::read_device_info_string(&conn.peripheral, DEVICE_INFO_HARDWARE_UUID).await;
        let manufacturer =
            Self::read_device_info_string(&conn.peripheral, DEVICE_INFO_MANUFACTURER_UUID).await;

        Ok((firmware, hardware, manufacturer))
    }

    async fn read_device_info_string(peripheral: &PlatformPeripheral, char_uuid: &str) -> String {
        for service in peripheral.services() {
            if service.uuid.to_string() != DEVICE_INFO_SERVICE_UUID {
                continue;
            }

            for char in service.characteristics {
                if char.uuid.to_string() == char_uuid
                    && let Ok(value) = peripheral.read(&char).await
                {
                    let text = String::from_utf8_lossy(&value);
                    let trimmed = text.trim_end_matches('\0');
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }

        "unknown".to_string()
    }

    pub async fn get_heart_rate_log_settings(
        conn: &Connection,
    ) -> Result<HeartRateLogSettings, DeviceError> {
        Self::write_request(conn, SettingsRequest::read()).await?;
        let response = Self::read_response_stream::<HeartRateLogSettings>(
            conn,
            CMD_HEART_RATE_LOG_SETTINGS,
            1000,
        )
        .await?;
        Ok(response)
    }

    pub async fn set_heart_rate_log_settings(
        conn: &Connection,
        enabled: bool,
        interval_minutes: u8,
    ) -> Result<(), DeviceError> {
        Self::write_request(
            conn,
            SettingsRequest::write_heart_rate(enabled, interval_minutes),
        )
        .await?;
        let _ = Self::read_response_stream::<HeartRateLogSettings>(
            conn,
            CMD_HEART_RATE_LOG_SETTINGS,
            1000,
        )
        .await?;
        Ok(())
    }

    pub async fn get_sleep(conn: &Connection) -> Result<SleepData, DeviceError> {
        let (data_write_char, data_notify_char) =
            Self::find_data_characteristics(&conn.peripheral).await?;

        let buffer = Self::read_big_data(
            &conn.peripheral,
            &data_write_char,
            &data_notify_char,
            DATA_REQUEST_ID_SLEEP,
        )
        .await?;

        Ok(parse_sleep_data(&buffer)?)
    }

    pub async fn get_oxygen(conn: &Connection) -> Result<OxygenData, DeviceError> {
        let (data_write_char, data_notify_char) =
            Self::find_data_characteristics(&conn.peripheral).await?;

        let buffer = Self::read_big_data(
            &conn.peripheral,
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

        Self::write_with_timeout(peripheral, data_write_char, &make_data_request(data_id)).await?;

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
        conn: &Connection,
        mut feed: impl FnMut(&[u8]) -> Result<Option<T>, crate::error::ProtocolError>,
    ) -> Result<T, DeviceError> {
        let mut notifications = conn
            .peripheral
            .notifications()
            .await
            .map_err(|_| ConnectionError::SubscribeFailed)?;

        let timeout_duration = Duration::from_millis(3000);

        loop {
            match timeout(timeout_duration, notifications.next()).await {
                Ok(Some(notification)) => {
                    if notification.uuid == conn.notify_char.uuid {
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
    pub async fn blink(conn: &Connection) -> Result<(), DeviceError> {
        Self::write_request(conn, BlinkRequest::new()).await?;
        Ok(())
    }

    pub async fn reboot(conn: &Connection) -> Result<(), DeviceError> {
        Self::write_request(conn, RebootRequest::new()).await?;
        Ok(())
    }

    pub async fn find(conn: &Connection) -> Result<(), DeviceError> {
        Self::write_request(conn, FindRequest::new()).await?;
        Ok(())
    }

    pub async fn reset(conn: &Connection) -> Result<(), DeviceError> {
        Self::write_request(conn, ResetRequest::new()).await?;
        Ok(())
    }

    pub async fn stream_realtime(
        conn: &Connection,
        reading_type: ReadingType,
        duration: Duration,
        tx: mpsc::Sender<RealtimeReading>,
    ) -> Result<(), DeviceError> {
        Self::write_with_timeout(
            &conn.peripheral,
            &conn.write_char,
            &make_phone_info_packet(),
        )
        .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self::write_request(conn, RealtimeStartRequest::new(reading_type)).await?;

        let mut notifications = conn
            .peripheral
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
                    if notification.uuid == conn.notify_char.uuid
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

        let _ = Self::write_request(conn, RealtimeStopRequest::new(reading_type)).await;

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
