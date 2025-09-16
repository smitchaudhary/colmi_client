use crate::{bluetooth::scanner, devices::models::Device, error::ScanError};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;
use tokio::task;

#[derive(PartialEq, Debug)]
pub enum Screen {
    Idle,
    Scanning,
    DeviceList,
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
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Char('s') => self.start_scanning(),
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
            Screen::Idle => {}
        }
    }

    pub fn start_scanning(&mut self) {
        if self.current_screen == Screen::Idle && !self.is_scanning {
            self.current_screen = Screen::Scanning;
            self.is_scanning = true;
            self.scan_start_time = Some(Instant::now());
            self.devices.clear();
            self.status_message = "Scanning devices...".to_string();

            self.scan_task = Some(tokio::spawn(
                async move { scanner::scan_for_devices().await },
            ));
        }
    }

    pub fn stop_scanning(&mut self) {
        self.is_scanning = false;
        self.scan_task = None;
        self.status_message = "Scanning stopped".to_string();
    }

    pub async fn update_scan_status(&mut self) {
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
            if let Some(selected) = self.selected_device {
                self.status_message = format!("Selected: {}", self.devices[selected].display_name())
            }
        }
    }
}
