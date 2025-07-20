use thiserror::Error;

#[derive(Error, Debug)]
pub enum BleError {
    #[error("No Bluetooth adapters found!")]
    NoAdapters,

    #[error("No external devices found!")]
    NoDevices,
}
