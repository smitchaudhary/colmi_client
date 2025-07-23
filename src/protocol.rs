pub mod battery;
pub mod features;

pub const SERVICE_UUID: &str = "6e40fff0-b5a3-f393-e0a9-e50e24dcca9e";
pub const WRITE_CHARACTERISTICS: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const NOTIFY_CHARACTERISTICS: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

pub trait Request {
    fn as_bytes(&self) -> [u8; 16];

    fn update_checksum(&self) -> u8 {
        calculate_checksum(&self.as_bytes())
    }
}

pub trait Response {
    fn from_bytes(bytes: Vec<u8>) -> Self;

    fn verify_checksum(bytes: &[u8]) -> bool {
        bytes.len() == 16 && bytes[15] == calculate_checksum(&bytes[..15])
    }
}

pub fn calculate_checksum(bytes: &[u8]) -> u8 {
    let sum: u32 = bytes[0..15].iter().map(|&b| b as u32).sum();
    (sum & 255) as u8
}
