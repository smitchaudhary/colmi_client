use crate::error::ProtocolError;
use crate::protocol::Request;

pub const CMD_GET_ACTIVITY_DATA: u8 = 0x43;

pub struct StepsRequest {
    pub command_id: u8,
    pub day_offset: i8,
    pub constants: [u8; 4],
    pub padding: [u8; 9],
    pub checksum: u8,
}

impl StepsRequest {
    pub fn new(day_offset: i8) -> Self {
        let mut req = Self {
            command_id: CMD_GET_ACTIVITY_DATA,
            day_offset,
            constants: [0x0F, 0x00, 0x5F, 0x01],
            padding: [0; 9],
            checksum: 0,
        };
        req.checksum = req.update_checksum();
        req
    }
}

impl Request for StepsRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.day_offset as u8;
        bytes[2..6].copy_from_slice(&self.constants);
        bytes[6..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;
        bytes
    }
}

#[derive(Clone, Debug)]
pub struct ActivityDetail {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    /// 15-minute slot index within the day (0-95).
    pub time_index: u8,
    pub calories: u32,
    pub steps: u16,
    /// Distance in meters.
    pub distance: u16,
}

#[derive(Debug)]
pub enum StepsResult {
    Details(Vec<ActivityDetail>),
    NoData,
}

pub struct ActivityDetailParser {
    new_calorie_protocol: bool,
    first_packet: bool,
    details: Vec<ActivityDetail>,
}

fn bcd_to_decimal(bcd: u8) -> u8 {
    ((bcd >> 4) & 0x0F) * 10 + (bcd & 0x0F)
}

impl ActivityDetailParser {
    pub fn new() -> Self {
        Self {
            new_calorie_protocol: false,
            first_packet: true,
            details: Vec::new(),
        }
    }

    pub fn feed(&mut self, packet: &[u8]) -> Result<Option<StepsResult>, ProtocolError> {
        if packet.len() != 16 {
            return Err(ProtocolError::PacketLength);
        }

        if self.first_packet && packet[1] == 0xFF {
            return Ok(Some(StepsResult::NoData));
        }

        if self.first_packet && packet[1] == 0xF0 {
            self.new_calorie_protocol = packet[3] == 0x01;
            self.first_packet = false;
            return Ok(None);
        }

        self.first_packet = false;

        let mut calories = u16::from_le_bytes([packet[7], packet[8]]) as u32;
        if self.new_calorie_protocol {
            calories *= 10;
        }
        let steps = u16::from_le_bytes([packet[9], packet[10]]);
        let distance = u16::from_le_bytes([packet[11], packet[12]]);

        self.details.push(ActivityDetail {
            year: 2000 + bcd_to_decimal(packet[1]) as u16,
            month: bcd_to_decimal(packet[2]),
            day: bcd_to_decimal(packet[3]),
            time_index: packet[4],
            calories,
            steps,
            distance,
        });

        if packet[5] == packet[6].saturating_sub(1) {
            let details = std::mem::take(&mut self.details);
            *self = Self::new();
            return Ok(Some(StepsResult::Details(details)));
        }

        Ok(None)
    }
}
