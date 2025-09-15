use crate::protocol::Request;

pub struct BlinkRequest {
    pub command_id: u8,
    pub padding: [u8; 14],
    pub checksum: u8,
}

impl BlinkRequest {
    pub fn new() -> Self {
        let mut req = Self {
            command_id: 16,
            padding: [0; 14],
            checksum: 16,
        };

        req.checksum = req.update_checksum();

        req
    }
}

impl Request for BlinkRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;

        bytes
    }
}
