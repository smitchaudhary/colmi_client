use crate::error::ProtocolError;
use crate::protocol::Request;

pub const CMD_READ_HEART_RATE: u8 = 0x15;
pub const HEART_RATE_POINTS_PER_DAY: usize = 288;

pub struct HeartRateRequest {
    pub command_id: u8,
    pub timestamp: u32,
    pub padding: [u8; 10],
    pub checksum: u8,
}

impl HeartRateRequest {
    pub fn new(timestamp: u32) -> Self {
        let mut req = Self {
            command_id: CMD_READ_HEART_RATE,
            timestamp,
            padding: [0; 10],
            checksum: 0,
        };
        req.checksum = req.update_checksum();
        req
    }
}

impl Request for HeartRateRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1..5].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[5..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;
        bytes
    }
}

#[derive(Clone, Debug)]
pub struct HeartRateLog {
    pub heart_rates: Vec<u8>,
    pub range: u8,
}

#[derive(Debug)]
pub enum HeartRateResult {
    Log(HeartRateLog),
    NoData,
}

pub struct HeartRateLogParser {
    size: usize,
    range: u8,
    raw: Vec<u8>,
    next_subtype: u8,
    started: bool,
}

impl HeartRateLogParser {
    pub fn new() -> Self {
        Self {
            size: 0,
            range: 0,
            raw: Vec::new(),
            next_subtype: 1,
            started: false,
        }
    }

    pub fn feed(&mut self, packet: &[u8]) -> Result<Option<HeartRateResult>, ProtocolError> {
        if packet.len() != 16 {
            return Err(ProtocolError::PacketLength);
        }

        let subtype = packet[1];

        if subtype == 0xFF {
            return Ok(Some(HeartRateResult::NoData));
        }

        if subtype == 0 && !self.started {
            self.size = packet[2] as usize;
            self.range = packet[3];
            self.raw = vec![0; self.size * 13];
            self.started = true;
            return Ok(None);
        }

        if !self.started {
            return Err(ProtocolError::MalformedSplitArray);
        }

        if subtype == 1 {
            self.raw[0..9].copy_from_slice(&packet[6..15]);
            self.next_subtype = 2;
        } else if subtype == self.next_subtype && self.next_subtype > 1 {
            // Packet 1 carries 9 bytes, all later packets 13.
            let offset = (subtype as usize - 1) * 13 - 4;
            self.raw[offset..offset + 13].copy_from_slice(&packet[2..15]);
            self.next_subtype += 1;
        } else {
            return Err(ProtocolError::MalformedSplitArray);
        }

        if subtype as usize == self.size - 1 {
            let mut heart_rates = self.raw.clone();
            heart_rates.truncate(HEART_RATE_POINTS_PER_DAY);
            heart_rates.resize(HEART_RATE_POINTS_PER_DAY, 0);

            let result = HeartRateLog {
                heart_rates,
                range: self.range,
            };
            *self = Self::new();
            return Ok(Some(HeartRateResult::Log(result)));
        }

        Ok(None)
    }
}
