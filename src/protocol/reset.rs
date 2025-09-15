use crate::protocol::Request;

pub struct ResetRequest {
    pub command_id: u8,
    pub padding: [u8; 14],
    pub checksum: u8,
}

impl ResetRequest {
    pub fn new() -> Self {
        let mut req = Self {
            command_id: 255,
            padding: [0; 14],
            checksum: 255,
        };

        req.checksum = req.update_checksum();

        req
    }
}

impl Request for ResetRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;

        bytes
    }
}
