pub mod app;
pub mod ui;

use std::{
    io::stdout,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    error::TuiError,
    tui::{app::App, ui::render_app},
};

use crate::devices::models::Device;
use inquire::Select;

pub fn select_device(devices: Vec<Device>) -> Option<Device> {
    Select::new("Choose the device to connect to:", devices)
        .prompt()
        .ok()
}

pub async fn run_tui() -> Result<(), TuiError> {
    enable_raw_mode().map_err(|e| TuiError::TerminalInit(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| TuiError::TerminalInit(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| TuiError::TerminalInit(e.to_string()))?;

    let mut app = App::new();

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal
            .draw(|f| render_app(f, &app))
            .map_err(|e| TuiError::Rendering(e.to_string()))?;

        if event::poll(tick_rate).map_err(|e| TuiError::EventHandling(e.to_string()))? {
            if let Event::Key(key) =
                event::read().map_err(|e| TuiError::EventHandling(e.to_string()))?
            {
                app.handle_key_event(key);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update_operations().await;
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    cleanup_terminal();

    Ok(())
}

pub fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}
