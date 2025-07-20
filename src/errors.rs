use thiserror::Error;

#[derive(Copy, Clone, Error, Debug)]
pub enum BleError {
    #[error("No Bluetooth adapters found!")]
    NoAdapters,

    #[error("No external devices found!")]
    NoDevices,

    #[error("Operation failed!")]
    OperationFailed,
}

impl BleError {
    pub fn display(&self, all: bool) {
        match self {
            BleError::NoAdapters => {
                println!("No Bluetooth adapters found!");
                println!("Please ensure Bluetooth is turned on.");
            }
            BleError::NoDevices => {
                if !all {
                    println!("No Colmi devices found!");
                    println!("Try `colmi_client scan --all` to see all devices.");
                } else {
                    println!("No Bluetooth devices found!");
                    println!("Please ensure devices are turned on and in range.");
                }
            }
            BleError::OperationFailed => {
                println!("Scan operation failed!");
            }
        }
    }
}
