use thiserror::Error;

#[derive(Copy, Clone, Error, Debug)]
pub enum ScanError {
    #[error("No Bluetooth adapters found!")]
    NoAdapters,

    #[error("No external devices found!")]
    NoDevices,

    #[error("Operation failed!")]
    OperationFailed,
}

impl ScanError {
    pub fn display(&self, all: bool) {
        match self {
            ScanError::NoAdapters => {
                println!("No Bluetooth adapters found!");
                println!("Please ensure Bluetooth is turned on.");
            }
            ScanError::NoDevices => {
                if !all {
                    println!("No Colmi devices found!");
                    println!("Try `colmi_client scan --all` to see all devices.");
                } else {
                    println!("No Bluetooth devices found!");
                    println!("Please ensure devices are turned on and in range.");
                }
            }
            ScanError::OperationFailed => {
                println!("Scan operation failed!");
            }
        }
    }
}

#[derive(Copy, Clone, Error, Debug)]
pub enum ConnectionError {
    #[error("Connection failed!")]
    ConnectionFailed,
}

impl ConnectionError {
    pub fn display(&self) {
        match self {
            ConnectionError::ConnectionFailed => println!("Connection to selected device failed!"),
        }
    }
}
