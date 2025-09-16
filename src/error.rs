use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("No Bluetooth adapters found! Please ensure Bluetooth is turned on.")]
    NoAdapters,

    #[error("No Bluetooth devices found! Please ensure devices are turned on and in range.")]
    NoDevices,

    #[error("No Colmi devices found! Try `colmi_client scan --all` to see all devices.")]
    NoColmiDevices,

    #[error("Bluetooth operation failed: {0}")]
    BluetoothOperationFailed(#[from] btleplug::Error),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection to selected device failed!")]
    ConnectionFailed,

    #[error("No matching characteristics found on selected device!")]
    CharacteristicsNotFound,

    #[error("Failed to write data to selected device!")]
    WriteFailed,

    #[error("Failed to read data from selected device!")]
    ReadFailed,

    #[error("Failed to subscribe to notifications from selected device!")]
    SubscribeFailed,
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid checksum. Calculated: {}, Actual: {}", calculated, actual)]
    InvalidChecksum { calculated: u8, actual: u8 },

    #[error("Invalid packet length")]
    InvalidPacketLength,
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error(transparent)]
    Connection(#[from] ConnectionError),

    #[error(transparent)]
    Protocol(#[from] ProtocolError),

    #[error("Operation timed out")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum TuiError {
    #[error(transparent)]
    Scan(#[from] ScanError),

    #[error(transparent)]
    Device(#[from] DeviceError),

    #[error("Terminal initialization failed: {0}")]
    TerminalInit(String),

    #[error("Terminal rendering failed: {0}")]
    Rendering(String),

    #[error("Event handling failed: {0}")]
    EventHandling(String),
}
