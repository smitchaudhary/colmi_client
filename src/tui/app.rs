use crate::{
    bluetooth::scanner,
    devices::{manager::Connection, manager::DeviceManager, models::Device},
    error::{DeviceError, ScanError},
    protocol::{
        battery::BatteryResponse,
        bigdata::{OxygenData, SleepData},
        hr::HeartRateResult,
        steps::StepsResult,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use chrono::{Datelike, TimeZone, Utc};
use std::time::Instant;
use tokio::task;

type DeviceInfo = (String, String, String);
type HistoryData = (HeartRateResult, StepsResult, SleepData, OxygenData);

#[derive(PartialEq, Debug)]
pub enum Screen {
    Idle,
    Scanning,
    DeviceList,
    Connecting,
    Connected,
    Error,
    ConfirmReset,
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
    pub connection: Option<Connection>,
    pub is_operation_in_progress: bool,
    pub connection_task: Option<task::JoinHandle<Result<Connection, DeviceError>>>,
    pub operation_task: Option<task::JoinHandle<Result<(), DeviceError>>>,
    pub battery_task: Option<task::JoinHandle<Result<BatteryResponse, DeviceError>>>,
    pub battery_level: Option<BatteryResponse>,
    pub device_info_task: Option<task::JoinHandle<Result<DeviceInfo, DeviceError>>>,
    pub device_info: Option<DeviceInfo>,
    pub history_task: Option<task::JoinHandle<Result<HistoryData, DeviceError>>>,
    pub history: Option<HistoryData>,
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
            connection: None,
            is_operation_in_progress: false,
            connection_task: None,
            operation_task: None,
            battery_task: None,
            battery_level: None,
            device_info_task: None,
            device_info: None,
            history_task: None,
            history: None,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Char('s') => self.start_scanning(),
            KeyCode::Char('b') => self.fetch_battery(),
            KeyCode::Char('h') => self.fetch_history(),
            KeyCode::Char('1') => self.blink_device(),
            KeyCode::Char('2') => self.find_device(),
            KeyCode::Char('3') => self.reboot_device(),
            KeyCode::Char('4') => self.reset_device(),
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
            Screen::ConfirmReset => {
                self.current_screen = Screen::Connected;
                self.status_message = "Device reset cancelled".to_string();
            }
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
        if let Some(task) = &mut self.scan_task
            && task.is_finished()
        {
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

        if let Some(task) = &mut self.connection_task
            && task.is_finished()
        {
            match task.await {
                Ok(Ok(connection)) => {
                    if let Some(selected) = self.selected_device {
                        self.connected_device = Some(self.devices[selected].clone());
                        self.connection = Some(connection);
                        self.current_screen = Screen::Connected;
                        self.status_message = format!(
                            "Connected to {}",
                            self.connected_device.as_ref().unwrap().display_name()
                        );
                        self.fetch_battery();
                        self.fetch_device_info();
                        self.fetch_history();
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

        if let Some(task) = &mut self.operation_task
            && task.is_finished()
        {
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

        if let Some(task) = &mut self.battery_task
            && task.is_finished()
        {
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

        if let Some(task) = &mut self.device_info_task
            && task.is_finished()
        {
            match task.await {
                Ok(Ok(device_info)) => {
                    self.device_info = Some(device_info);
                }
                Ok(Err(err)) => {
                    self.error_message = Some(format!("Device info fetch failed: {}", err));
                }
                Err(_) => {
                    self.error_message = Some("Device info task panicked".to_string());
                }
            }
            self.device_info_task = None;
        }

        if let Some(task) = &mut self.history_task
            && task.is_finished()
        {
            match task.await {
                Ok(Ok(history)) => {
                    self.history = Some(history);
                }
                Ok(Err(err)) => {
                    self.error_message = Some(format!("History fetch failed: {}", err));
                }
                Err(_) => {
                    self.error_message = Some("History task panicked".to_string());
                }
            }
            self.history_task = None;
        }
    }

    fn handle_up(&mut self) {
        if self.current_screen == Screen::DeviceList
            && !self.devices.is_empty()
            && let Some(selected) = self.selected_device
        {
            self.selected_device = Some(selected.saturating_sub(1))
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
        if self.current_screen == Screen::DeviceList
            && let Some(selected_device) = self.selected_device
        {
            let device = self.devices[selected_device].clone();
            self.status_message = format!("Selected: {}", device.display_name());
            self.current_screen = Screen::Connecting;
            self.is_operation_in_progress = true;
            self.connecting_device_name = Some(device.display_name().to_string());
            self.connection_task = Some(tokio::spawn(async move {
                DeviceManager::connect_and_setup(&device).await
            }));
        }
    }

    fn fetch_battery(&mut self) {
        if self.current_screen == Screen::Connected
            && let Some(conn) = &self.connection
        {
            self.status_message = "Fetching battery level...".to_string();
            let conn = conn.clone();
            self.battery_task = Some(tokio::spawn(async move {
                DeviceManager::get_battery_level(&conn).await
            }));
        }
    }

    fn fetch_device_info(&mut self) {
        if self.current_screen == Screen::Connected
            && let Some(conn) = &self.connection
        {
            let conn = conn.clone();
            self.device_info_task = Some(tokio::spawn(async move {
                DeviceManager::get_device_info(&conn).await
            }));
        }
    }

    fn fetch_history(&mut self) {
        if self.current_screen == Screen::Connected
            && self.history_task.is_none()
            && let Some(conn) = &self.connection
        {
            self.status_message = "Fetching today's data...".to_string();
            let conn = conn.clone();
            self.history_task = Some(tokio::spawn(async move {
                let day = Utc::now();
                let midnight = Utc
                    .with_ymd_and_hms(day.year(), day.month(), day.day(), 0, 0, 0)
                    .single()
                    .ok_or(DeviceError::StreamEnded)?;
                let heart_rate =
                    DeviceManager::get_heart_rate_log(&conn, midnight.timestamp() as u32).await?;
                let steps = DeviceManager::get_steps(&conn, 0).await?;
                let sleep = DeviceManager::get_sleep(&conn).await?;
                let oxygen = DeviceManager::get_oxygen(&conn).await?;
                Ok((heart_rate, steps, sleep, oxygen))
            }));
        }
    }

    fn blink_device(&mut self) {
        if self.current_screen == Screen::Connected
            && let Some(conn) = &self.connection
        {
            self.status_message = "Blinking device...".to_string();
            let conn = conn.clone();
            self.operation_task = Some(tokio::spawn(
                async move { DeviceManager::blink(&conn).await },
            ));
        }
    }

    fn find_device(&mut self) {
        if self.current_screen == Screen::Connected
            && let Some(conn) = &self.connection
        {
            self.status_message = "Finding device...".to_string();
            let conn = conn.clone();
            self.operation_task = Some(tokio::spawn(
                async move { DeviceManager::find(&conn).await },
            ));
        }
    }

    fn reboot_device(&mut self) {
        if self.current_screen == Screen::Connected
            && let Some(conn) = &self.connection
        {
            self.status_message = "Rebooting device...".to_string();
            let conn = conn.clone();
            self.operation_task = Some(tokio::spawn(
                async move { DeviceManager::reboot(&conn).await },
            ));
        }
    }

    fn reset_device(&mut self) {
        if self.current_screen == Screen::Connected {
            if self.connection.is_some() {
                self.current_screen = Screen::ConfirmReset;
                self.status_message = "Are you sure you want to reset the device?".to_string();
            }
        } else if self.current_screen == Screen::ConfirmReset
            && let Some(conn) = &self.connection
        {
            self.status_message = "Resetting device...".to_string();
            let conn = conn.clone();
            self.operation_task = Some(tokio::spawn(
                async move { DeviceManager::reset(&conn).await },
            ));

            self.current_screen = Screen::Idle;
        }
    }
}
