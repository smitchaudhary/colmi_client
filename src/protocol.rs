use crate::error::ProtocolError;

pub mod battery;
pub mod bigdata;
pub mod blink;
pub mod features;
pub mod find;
pub mod hr;
pub mod realtime;
pub mod reboot;
pub mod reset;
pub mod steps;

pub const SERVICE_UUID: &str = "6e40fff0-b5a3-f393-e0a9-e50e24dcca9e";
pub const WRITE_CHARACTERISTICS: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const NOTIFY_CHARACTERISTICS: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

pub const DATA_SERVICE_UUID: &str = "de5bf728-d711-4e47-af26-65e3012a5dc7";
pub const DATA_WRITE_CHARACTERISTICS: &str = "de5bf72a-d711-4e47-af26-65e3012a5dc7";
pub const DATA_NOTIFY_CHARACTERISTICS: &str = "de5bf729-d711-4e47-af26-65e3012a5dc7";

pub trait Request {
    fn as_bytes(&self) -> [u8; 16];

    fn update_checksum(&self) -> u8 {
        calculate_checksum(&self.as_bytes())
    }
}

pub trait Response {
    const EXPECTED_COMMAND_ID: u8;

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ProtocolError>
    where
        Self: Sized;

    fn verify_checksum(bytes: &[u8]) -> Result<(), ProtocolError> {
        if bytes.len() != 16 {
            return Err(ProtocolError::PacketLength);
        }
        let calculated = calculate_checksum(&bytes[..15]);
        let actual = bytes[15];

        if calculated != actual {
            return Err(ProtocolError::Checksum { calculated, actual });
        }

        Ok(())
    }

    fn validate_command_id(bytes: &[u8]) -> Result<(), ProtocolError> {
        if has_error_flag(bytes) {
            return Err(ProtocolError::ErrorFlag {
                command_id: bytes[0],
            });
        }
        if bytes[0] != Self::EXPECTED_COMMAND_ID {
            return Err(ProtocolError::CommandId {
                expected: Self::EXPECTED_COMMAND_ID,
                actual: bytes[0],
            });
        }
        Ok(())
    }
}

pub fn calculate_checksum(bytes: &[u8]) -> u8 {
    let sum: u32 = bytes[0..15].iter().map(|&b| b as u32).sum();
    (sum & 255) as u8
}

pub fn has_error_flag(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(|&b| b & 0x80 != 0)
}

pub fn to_bcd(value: u8) -> u8 {
    let tens = value / 10;
    let ones = value % 10;
    (tens << 4) | ones
}
