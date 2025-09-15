use crate::protocol::Request;

pub struct FindRequest {
    pub command_id: u8,
    pub data_1: u8,
    pub data_2: u8,
    pub padding: [u8; 12],
    pub checksum: u8,
}

impl FindRequest {
    pub fn new() -> Self {
        let mut req = Self {
            command_id: 80,
            data_1: 85,
            data_2: 170,
            padding: [0; 12],
            checksum: 0,
        };

        req.checksum = req.update_checksum();

        req
    }
}

impl Request for FindRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.data_1;
        bytes[2] = self.data_2;
        bytes[3..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;

        bytes
    }
}
