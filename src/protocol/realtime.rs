use crate::error::ProtocolError;
use crate::protocol::Request;

pub const CMD_START_REAL_TIME: u8 = 0x69;
pub const CMD_STOP_REAL_TIME: u8 = 0x6A;
pub const ACTION_START: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingType {
    HeartRateBatch = 0x01,
    BloodOxygen = 0x03,
    Hrv = 0x0A,
}

impl ReadingType {
    pub fn from_byte(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x01 => Ok(Self::HeartRateBatch),
            0x03 => Ok(Self::BloodOxygen),
            0x0A => Ok(Self::Hrv),
            other => Err(ProtocolError::UnknownReadingType(other)),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::HeartRateBatch => "heart rate",
            Self::BloodOxygen => "blood oxygen",
            Self::Hrv => "HRV",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            Self::HeartRateBatch => "bpm",
            Self::BloodOxygen => "%",
            Self::Hrv => "ms",
        }
    }
}

pub struct RealtimeStartRequest {
    pub command_id: u8,
    pub reading_type: ReadingType,
    pub padding: [u8; 13],
    pub checksum: u8,
}

impl RealtimeStartRequest {
    pub fn new(reading_type: ReadingType) -> Self {
        let mut req = Self {
            command_id: CMD_START_REAL_TIME,
            reading_type,
            padding: [0; 13],
            checksum: 0,
        };
        req.checksum = req.update_checksum();
        req
    }
}

impl Request for RealtimeStartRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.reading_type as u8;
        bytes[2] = ACTION_START;
        bytes[3..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;
        bytes
    }
}

pub struct RealtimeStopRequest {
    pub command_id: u8,
    pub reading_type: ReadingType,
    pub padding: [u8; 13],
    pub checksum: u8,
}

impl RealtimeStopRequest {
    pub fn new(reading_type: ReadingType) -> Self {
        let mut req = Self {
            command_id: CMD_STOP_REAL_TIME,
            reading_type,
            padding: [0; 13],
            checksum: 0,
        };
        req.checksum = req.update_checksum();
        req
    }
}

impl Request for RealtimeStopRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.reading_type as u8;
        bytes[2..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;
        bytes
    }
}

#[derive(Clone, Debug)]
pub struct RealtimeReading {
    pub reading_type: ReadingType,
    pub value: u8,
}

impl RealtimeReading {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 16 {
            return Err(ProtocolError::PacketLength);
        }
        if bytes[0] != CMD_START_REAL_TIME {
            return Err(ProtocolError::CommandId {
                expected: CMD_START_REAL_TIME,
                actual: bytes[0],
            });
        }

        let reading_type = ReadingType::from_byte(bytes[1])?;

        let error_code = bytes[2];
        if error_code != 0 {
            return Err(ProtocolError::ReadingError {
                reading_type: bytes[1],
                code: error_code,
            });
        }

        Ok(Self {
            reading_type,
            value: bytes[3],
        })
    }
}
