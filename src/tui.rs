use crate::devices::models::Device;
use inquire::Select;

pub fn select_device(devices: Vec<Device>) -> Option<Device> {
    Select::new("Choose the device to connect to:", devices)
        .prompt()
        .ok()
}
