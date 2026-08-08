use crate::error::ProtocolError;
use crate::protocol::{Request, Response};

pub const CMD_HEART_RATE_LOG_SETTINGS: u8 = 0x16;

pub struct SettingsRequest {
    pub command_id: u8,
    pub action: u8,
    pub payload: [u8; 13],
    pub checksum: u8,
}

impl SettingsRequest {
    pub fn read() -> Self {
        let mut req = Self {
            command_id: CMD_HEART_RATE_LOG_SETTINGS,
            action: 0x01,
            payload: [0; 13],
            checksum: 0,
        };
        req.checksum = req.update_checksum();
        req
    }

    pub fn write_heart_rate(enabled: bool, interval_minutes: u8) -> Self {
        let mut req = Self {
            command_id: CMD_HEART_RATE_LOG_SETTINGS,
            action: 0x02,
            payload: [0; 13],
            checksum: 0,
        };
        req.payload[0] = if enabled { 1 } else { 2 };
        req.payload[1] = interval_minutes;
        req.checksum = req.update_checksum();
        req
    }
}

impl Request for SettingsRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.action;
        bytes[2..15].copy_from_slice(&self.payload);
        bytes[15] = self.checksum;
        bytes
    }
}

#[derive(Clone, Debug)]
pub struct HeartRateLogSettings {
    pub enabled: bool,
    /// Logging interval in minutes.
    pub interval: u8,
}

impl Response for HeartRateLogSettings {
    const EXPECTED_COMMAND_ID: u8 = CMD_HEART_RATE_LOG_SETTINGS;

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        Self::validate_command_id(&bytes)?;
        Self::verify_checksum(&bytes)?;

        Ok(Self {
            enabled: bytes[2] == 1,
            interval: bytes[3],
        })
    }
}
