use crate::error::ProtocolError;
use crate::protocol::{Request, Response};
use chrono::{Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};

pub struct FeatureRequest {
    pub command_id: u8,
    pub year: u8,
    pub month: u8,
    pub day_of_month: u8,
    pub hour: u8,
    pub minute: u8,
    pub seconds: u8,
    pub padding: [u8; 8],
    pub checksum: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeatureResponse {
    pub command_id: u8,
    pub supports_temperature: bool,
    pub supports_plate: bool,
    pub supports_menstruation: bool,
    pub supports_custom_wallpaper: bool,
    pub supports_blood_oxygen: bool,
    pub supports_blood_pressure: bool,
    pub supports_unknown_feature: bool,
    pub supports_one_key_check: bool,
    pub supports_weather: bool,
    pub supports_wechat: bool,
    pub supports_avatar: bool,
    pub width: u16,
    pub height: u16,
    pub use_new_sleep_protocol: bool,
    pub max_watch_faces: u8,
    pub supports_contacts: bool,
    pub supports_lyrics: bool,
    pub supports_album: bool,
    pub supports_gps: bool,
    pub supports_jeilei_music: bool,
    pub supports_manual_heart_rate: bool,
    pub supports_ecard: bool,
    pub supports_location: bool,
    pub supports_music: bool,
    pub supports_ebook: bool,
    pub supports_blood_sugar: bool,
    pub max_contacts: u16,
    pub supports_blood_pressure_settings: bool,
    pub supports_4g: bool,
    pub supports_nav_picture: bool,
    pub supports_pressure: bool,
    pub supports_hrv: bool,
    pub checksum: u8,
}

impl FeatureRequest {
    pub fn new() -> Self {
        let now = Local::now();

        let mut req = Self {
            command_id: 1,
            year: (now.year() % 2000) as u8,
            month: now.month() as u8,
            day_of_month: now.day() as u8,
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            seconds: now.second() as u8,
            padding: [0; 8],
            checksum: 1,
        };

        req.checksum = req.update_checksum();
        req
    }
}

impl Request for FeatureRequest {
    fn as_bytes(&self) -> [u8; 16] {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0] = self.command_id;
        bytes[1] = self.year;
        bytes[2] = self.month;
        bytes[3] = self.day_of_month;
        bytes[4] = self.hour;
        bytes[5] = self.minute;
        bytes[6] = self.seconds;
        bytes[7..15].copy_from_slice(&self.padding);
        bytes[15] = self.checksum;

        bytes
    }
}

impl Response for FeatureResponse {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        Self::verify_checksum(&bytes)?;

        Ok(Self {
            command_id: bytes[0],
            supports_temperature: bytes[1] != 0,
            supports_plate: bytes[2] != 0,
            supports_menstruation: bytes[3] != 0,
            supports_custom_wallpaper: (bytes[4] & 1 << 0) != 0,
            supports_blood_oxygen: (bytes[4] & 1 << 1) != 0,
            supports_blood_pressure: (bytes[4] & 1 << 2) != 0,
            supports_unknown_feature: (bytes[4] & 1 << 3) != 0,
            supports_one_key_check: (bytes[4] & 1 << 4) != 0,
            supports_weather: (bytes[4] & 1 << 5) != 0,
            supports_wechat: (bytes[4] & 1 << 6) != 0,
            supports_avatar: (bytes[4] & 1 << 7) != 0,
            width: u16::from_le_bytes([bytes[5], bytes[6]]),
            height: u16::from_le_bytes([bytes[7], bytes[8]]),
            use_new_sleep_protocol: bytes[9] != 0,
            max_watch_faces: bytes[10],
            supports_contacts: (bytes[11] & 1 << 0) != 0,
            supports_lyrics: (bytes[11] & 1 << 1) != 0,
            supports_album: (bytes[11] & 1 << 2) != 0,
            supports_gps: (bytes[11] & 1 << 3) != 0,
            supports_jeilei_music: (bytes[11] & 1 << 4) != 0,
            supports_manual_heart_rate: (bytes[12] & 1 << 0) != 0,
            supports_ecard: (bytes[12] & 1 << 1) != 0,
            supports_location: (bytes[12] & 1 << 2) != 0,
            supports_music: (bytes[12] & 1 << 4) != 0,
            supports_ebook: (bytes[12] & 1 << 6) != 0,
            supports_blood_sugar: (bytes[12] & 1 << 7) != 0,
            max_contacts: { if bytes[13] == 0 { 20 } else { bytes[13] * 10 } } as u16,
            supports_blood_pressure_settings: (bytes[14] & 1 << 0) != 0,
            supports_4g: (bytes[14] & 1 << 2) != 0,
            supports_nav_picture: (bytes[14] & 1 << 3) != 0,
            supports_pressure: (bytes[14] & 1 << 4) != 0,
            supports_hrv: (bytes[14] & 1 << 5) != 0,
            checksum: bytes[15],
        })
    }
}
