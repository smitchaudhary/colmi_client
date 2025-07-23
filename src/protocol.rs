pub mod battery;
pub mod features;

pub const SERVICE_UUID: &str = "6e40fff0-b5a3-f393-e0a9-e50e24dcca9e";
pub const WRITE_CHARACTERISTICS: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const NOTIFY_CHARACTERISTICS: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

pub trait Request {
    fn as_bytes(&self) -> [u8; 16];
}

pub trait Response {
    fn from_bytes(bytes: Vec<u8>) -> Self;
}
