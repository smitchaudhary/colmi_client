use std::fmt::{Display, Formatter};

pub const SERVICE_UUID: &str = "6e40fff0-b5a3-f393-e0a9-e50e24dcca9e";
pub const WRITE_CHARACTERISTICS: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const NOTIFY_CHARACTERISTICS: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

pub struct BatteryRequest {
    pub command_id: u8,
    pub padding: [u8; 14],
    pub crc: u8,
}

pub struct BatteryResponse {
    pub command_id: u8,
    pub charge_pct: u8,
    pub is_charging: bool,
    pub padding: [u8; 12],
    pub crc: u8,
}

impl BatteryRequest {
    pub fn new() -> Self {
        Self {
            command_id: 3,
            padding: [0; 14],
            crc: 3,
        }
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1..15].copy_from_slice(&self.padding);
        bytes[15] = self.crc;

        bytes
    }
}

impl BatteryResponse {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let command_id = bytes[0];
        let charge_pct = bytes[1];
        let is_charging = bytes[2] == 1;
        let padding = bytes[3..15].try_into().unwrap();
        let crc = bytes[15];

        Self {
            command_id,
            charge_pct,
            is_charging,
            padding,
            crc,
        }
    }
}

impl Display for BatteryResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Battery level: {}% | Charging: {}",
            self.charge_pct, self.is_charging
        )
    }
}
