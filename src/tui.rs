use btleplug::api::Peripheral;
use inquire::Select;

pub fn select_device<P: Peripheral>(devices_info: &[(String, P)]) -> Option<usize> {
    let display_names: Vec<String> = devices_info.iter().map(|(name, _)| name.clone()).collect();

    Select::new("Choose the device to connect to:", display_names)
        .raw_prompt()
        .ok()
        .map(|result| result.index)
}
