use std::fmt::{Display, Formatter};

use crate::error::ProtocolError;
use crate::protocol::{Request, Response};

pub struct BatteryRequest {
    pub command_id: u8,
    pub padding: [u8; 14],
    pub checksum: u8,
}

pub struct BatteryResponse {
    pub command_id: u8,
    pub charge_pct: u8,
    pub is_charging: bool,
    pub padding: [u8; 12],
    pub checksum: u8,
}

impl BatteryRequest {
    pub fn new() -> Self {
        let mut req = Self {
            command_id: 3,
            padding: [0; 14],
            checksum: 3,
        };
        req.checksum = req.update_checksum();

        req
    }
}

impl Request for BatteryRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;

        bytes
    }
}

impl Response for BatteryResponse {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        match Self::verify_checksum(&bytes) {
            Ok(_) => Ok(Self {
                command_id: bytes[0],
                charge_pct: bytes[1],
                is_charging: bytes[2] == 1,
                padding: bytes[3..15].try_into().unwrap(),
                checksum: bytes[15],
            }),
            Err(err) => Err(err),
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
