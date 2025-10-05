use crate::{
    bluetooth::scanner,
    devices::{manager::DeviceManager, models::Device},
    error::{DeviceError, ScanError},
    protocol::battery::BatteryResponse,
};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;
use tokio::task;

#[derive(PartialEq, Debug)]
pub enum Screen {
    Idle,
    Scanning,
    DeviceList,
    Connecting,
    Connected,
    Error,
}

pub struct App {
    pub current_screen: Screen,
    pub should_quit: bool,

    pub devices: Vec<Device>,
    pub selected_device: Option<usize>,
    pub is_scanning: bool,
    pub scan_start_time: Option<Instant>,

    pub status_message: String,
    pub error_message: Option<String>,

    pub scan_task: Option<task::JoinHandle<Result<Vec<Device>, ScanError>>>,

    pub connecting_device_name: Option<String>,

    pub connected_device: Option<Device>,
    pub is_operation_in_progress: bool,
    pub connection_task: Option<task::JoinHandle<Result<(), DeviceError>>>,
    pub operation_task: Option<task::JoinHandle<Result<(), DeviceError>>>,
    pub battery_task: Option<task::JoinHandle<Result<BatteryResponse, DeviceError>>>,
    pub battery_level: Option<BatteryResponse>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Idle,
            should_quit: false,
            devices: Vec::new(),
            selected_device: None,
            is_scanning: false,
            scan_start_time: None,
            status_message: "Ready to scan".to_string(),
            error_message: None,
            scan_task: None,
            connecting_device_name: None,
            connected_device: None,
            is_operation_in_progress: false,
            connection_task: None,
            operation_task: None,
            battery_task: None,
            battery_level: None,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Char('s') => self.start_scanning(),
            KeyCode::Char('b') => self.fetch_battery(),
            KeyCode::Up => self.handle_up(),
            KeyCode::Down => self.handle_down(),
            KeyCode::Enter => self.handle_enter(),
            _ => {}
        }
    }

    fn handle_escape(&mut self) {
        match self.current_screen {
            Screen::Scanning => {
                self.stop_scanning();
                self.current_screen = Screen::Idle;
                self.status_message = "Scanning cancelled".to_string();
            }
            Screen::DeviceList => {
                self.current_screen = Screen::Idle;
                self.devices.clear();
                self.selected_device = None;
                self.status_message = "Ready to scan".to_string();
            }
            Screen::Error => {
                self.current_screen = Screen::Idle;
                self.error_message = None;
            }
            Screen::Connecting => {
                self.cancel_connection();
                self.current_screen = Screen::Idle;
                self.status_message = "Connection cancelled".to_string();
            }
            Screen::Connected => {}
            Screen::Idle => {}
        }
    }

    fn cancel_connection(&mut self) {
        if let Some(task) = &mut self.connection_task {
            task.abort();
            self.connection_task = None;
        }
        self.connecting_device_name = None;
        self.is_operation_in_progress = false;
        self.status_message = "Connection cancelled".to_string();
    }

    pub fn start_scanning(&mut self) {
        if self.current_screen == Screen::Idle
            || self.current_screen == Screen::DeviceList && !self.is_scanning
        {
            self.current_screen = Screen::Scanning;
            self.is_scanning = true;
            self.scan_start_time = Some(Instant::now());
            self.devices.clear();
            self.status_message = "Scanning devices...".to_string();

            self.scan_task = Some(tokio::spawn(async move {
                match scanner::scan_for_devices().await {
                    Ok(all_devices) => {
                        let colmi_devices: Vec<Device> = all_devices
                            .into_iter()
                            .filter(|d| d.is_colmi_device())
                            .collect();
                        Ok(colmi_devices)
                    }
                    Err(err) => Err(err),
                }
            }));
        }
    }

    pub fn stop_scanning(&mut self) {
        self.is_scanning = false;
        self.scan_task = None;
        self.status_message = "Scanning stopped".to_string();
    }

    pub async fn update_operations(&mut self) {
        if let Some(task) = &mut self.scan_task {
            if task.is_finished() {
                match task.await {
                    Ok(Ok(devices)) => {
                        self.devices = devices;
                        self.current_screen = Screen::DeviceList;
                        self.selected_device = if !self.devices.is_empty() {
                            Some(0)
                        } else {
                            None
                        };
                        self.status_message = format!("Found {} devices", self.devices.len());
                    }
                    Ok(Err(error)) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some(format!("Scan failed: {}", error));
                    }
                    Err(_) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some("Scan task panicked".to_string());
                    }
                }
                self.scan_task = None;
                self.is_scanning = false;
            }
        }

        if let Some(task) = &mut self.connection_task {
            if task.is_finished() {
                match task.await {
                    Ok(Ok(_)) => {
                        if let Some(selected) = self.selected_device {
                            self.connected_device = Some(self.devices[selected].clone());
                            self.current_screen = Screen::Connected;
                            self.status_message = format!(
                                "Connected to {}",
                                self.connected_device.as_ref().unwrap().display_name()
                            );
                        }
                    }
                    Ok(Err(err)) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some(format!("Connection failed: {}", err));
                    }
                    Err(_) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some("Connection task panicked".to_string());
                    }
                }
                self.connection_task = None;
                self.connecting_device_name = None;
                self.is_operation_in_progress = false;
            }
        }

        if let Some(task) = &mut self.operation_task {
            if task.is_finished() {
                match task.await {
                    Ok(Ok(_)) => {
                        self.status_message = "Operation completed successfully".to_string();
                    }
                    Ok(Err(err)) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some(format!("Operation failed: {}", err));
                    }
                    Err(_) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some("Operation task panicked".to_string());
                    }
                }
                self.operation_task = None;
            }
        }

        if let Some(task) = &mut self.battery_task {
            if task.is_finished() {
                match task.await {
                    Ok(Ok(battery_response)) => {
                        self.status_message = format!(
                            "Battery: {}% | Charging: {}",
                            battery_response.charge_pct, battery_response.is_charging
                        );
                        self.battery_level = Some(battery_response);
                    }
                    Ok(Err(err)) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some(format!("Battery fetch failed: {}", err));
                    }
                    Err(_) => {
                        self.current_screen = Screen::Error;
                        self.error_message = Some("Battery fetch task panicked".to_string());
                    }
                }
                self.battery_task = None;
            }
        }
    }

    fn handle_up(&mut self) {
        if self.current_screen == Screen::DeviceList && !self.devices.is_empty() {
            if let Some(selected) = self.selected_device {
                self.selected_device = Some(selected.saturating_sub(1))
            }
        }
    }

    fn handle_down(&mut self) {
        if self.current_screen == Screen::DeviceList && !self.devices.is_empty() {
            if let Some(selected) = self.selected_device {
                if selected < self.devices.len() - 1 {
                    self.selected_device = Some(selected + 1)
                }
            } else {
                self.selected_device = Some(0)
            }
        }
    }

    fn handle_enter(&mut self) {
        if self.current_screen == Screen::DeviceList {
            if let Some(selected_device) = self.selected_device {
                let device = self.devices[selected_device].clone();
                self.status_message = format!("Selected: {}", device.display_name());
                self.current_screen = Screen::Connecting;
                self.is_operation_in_progress = true;
                self.connecting_device_name = Some(device.display_name().to_string());
                self.connection_task = Some(tokio::spawn(async move {
                    match DeviceManager::connect_and_setup(&device).await {
                        Ok(_) => Ok(()),
                        Err(err) => Err(err),
                    }
                }));
            }
        }
    }

    fn fetch_battery(&mut self) {
        if self.current_screen == Screen::Connected {
            if let Some(ref device) = self.connected_device {
                self.status_message = "Fetching battery level...".to_string();
                let device = device.clone();
                self.battery_task = Some(tokio::spawn(async move {
                    DeviceManager::get_battery_level(&device).await
                }));
            }
        }
    }
}
