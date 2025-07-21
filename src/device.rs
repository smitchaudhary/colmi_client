pub mod colmi {
    use std::collections::HashMap;

    const COLMI_MANUFACTURER_ID: u16 = 4660;

    pub fn is_colmi_device(manufacturer_data: &HashMap<u16, Vec<u8>>) -> bool {
        manufacturer_data.contains_key(&COLMI_MANUFACTURER_ID)
    }

    pub fn format_device_info(name: Option<String>, address: String) -> String {
        let device_name = name.unwrap_or_else(|| "Unknown Device".to_string());
        format!("{}, ({})", device_name, address)
    }
}