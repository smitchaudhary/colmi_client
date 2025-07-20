mod ble;

#[tokio::main]
async fn main() {
    println!("Starting Bluetooth device scan...");
    
    match ble::scan_for_devices().await {
        Ok(_) => println!("Scan completed successfully"),
        Err(e) => eprintln!("Error during scan: {}", e),
    }
}
