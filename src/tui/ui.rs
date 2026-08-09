use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    protocol::{hr::HeartRateResult, steps::StepsResult},
    tui::app::{App, ConnectedTab, Screen},
};

pub fn render_app(f: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, main_layout[0]);
    render_main_content(f, main_layout[1], app);
    render_footer(f, main_layout[2], app);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![Span::styled(
        "Colmi TUI",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let header = Paragraph::new(title).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue)),
    );

    f.render_widget(header, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_screen {
        Screen::Idle => render_idle_screen(f, area, app),
        Screen::Scanning => render_scanning_screen(f, area, app),
        Screen::DeviceList => render_device_list_screen(f, area, app),
        Screen::Error => render_error_screen(f, area, app),
        Screen::Connecting => render_connecting_screen(f, area, app),
        Screen::Connected => render_connected_screen(f, area, app),
        Screen::ConfirmReset => render_confirm_reset_screen(f, area, app),
    }
}

fn render_idle_screen(f: &mut Frame, area: Rect, app: &App) {
    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Device Scanner",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from("Press [s] to start scanning for Colmi devices"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::White)),
            Span::raw(&app.status_message),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Scanner"),
        );

    f.render_widget(paragraph, area);
}

fn render_scanning_screen(f: &mut Frame, area: Rect, app: &App) {
    let elapsed = app
        .scan_start_time
        .map(|start_time| start_time.elapsed().as_secs())
        .unwrap_or(0);

    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Scanning... ({})s", elapsed),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Searching for Colmi devices..."),
        Line::from(""),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled("Scanning in progress", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ESC to stop scanning",
            Style::default().fg(Color::Red),
        )]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Scanning"),
    );

    f.render_widget(paragraph, area);
}

fn render_device_list_screen(f: &mut Frame, area: Rect, app: &App) {
    let devices: Vec<ListItem> = app
        .devices
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let style = if Some(i) == app.selected_device {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(device.display_name().to_string()).style(style)
        })
        .collect();

    let list = List::new(devices)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("Found {} Devices", app.devices.len())),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(app.selected_device);

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_error_screen(f: &mut Frame, area: Rect, app: &App) {
    let error_msg = app.error_message.as_deref().unwrap_or("Unknown Error");
    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Error!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::raw(error_msg)]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled("Press ESC to return", Style::default())]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Error")
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(paragraph, area);
}

fn render_connecting_screen(f: &mut Frame, area: Rect, app: &App) {
    let device_name = app
        .connecting_device_name
        .as_deref()
        .unwrap_or("Unknown Device");

    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Connecting...",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!("Connecting to {}...", device_name)),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ESC to cancel",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Connecting"),
    );

    f.render_widget(paragraph, area);
}

fn create_battery_bar(charge_pct: u8) -> String {
    let filled_blocks = (charge_pct as f32 / 100.0 * 10.0).round() as usize;
    let empty_blocks = 10 - filled_blocks;
    format!(
        "[{}{}]",
        "▪".repeat(filled_blocks),
        "░".repeat(empty_blocks)
    )
}

fn get_battery_color(charge_pct: u8) -> Style {
    Style::default().fg(if charge_pct >= 70 {
        Color::Green
    } else if charge_pct >= 30 {
        Color::Yellow
    } else {
        Color::Red
    })
}

fn render_sparkline(readings: &[crate::protocol::realtime::RealtimeReading]) -> String {
    let window: Vec<u8> = readings.iter().rev().take(30).map(|r| r.value).collect();
    let max = window.iter().copied().max().unwrap_or(1).max(1) as f32;
    let mut line = String::new();
    for &value in window.iter().rev() {
        let blocks = ((value as f32 / max) * 8.0).round() as usize;
        line.push(match blocks.clamp(1, 8) {
            1 => '▁',
            2 => '▂',
            3 => '▃',
            4 => '▄',
            5 => '▅',
            6 => '▆',
            7 => '▇',
            _ => '█',
        });
    }
    line
}

fn tab_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    }
}

fn render_connected_screen(f: &mut Frame, area: Rect, app: &App) {
    let tabs = Line::from(vec![
        Span::styled(" Live ", tab_style(app.connected_tab == ConnectedTab::Live)),
        Span::styled(
            " Today ",
            tab_style(app.connected_tab == ConnectedTab::Today),
        ),
        Span::styled(
            " Controls ",
            tab_style(app.connected_tab == ConnectedTab::Controls),
        ),
    ]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    f.render_widget(Paragraph::new(tabs).alignment(Alignment::Center), layout[0]);

    match app.connected_tab {
        ConnectedTab::Live => render_live_tab(f, layout[1], app),
        ConnectedTab::Today => render_today_tab(f, layout[1], app),
        ConnectedTab::Controls => render_controls_tab(f, layout[1], app),
    }
}

fn render_status_and_footer(content: &mut Vec<Line<'static>>, app: &App) {
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::raw("Status: "),
        Span::styled(
            app.status_message.clone(),
            Style::default().fg(Color::White),
        ),
    ]));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("TAB ", Style::default().fg(Color::Yellow)),
        Span::raw("switch tab | "),
        Span::styled("ESC ", Style::default().fg(Color::Yellow)),
        Span::raw("disconnect"),
    ]));
}

fn render_live_tab(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Connected!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    if let Some(device) = &app.connected_device {
        content.push(Line::from(format!("Device: {}", device.name())));
    } else {
        content.push(Line::from("Connected to device"));
    }

    if let Some((firmware, hardware, manufacturer)) = &app.device_info {
        content.push(Line::from(format!(
            "{} | firmware {} | hw {}",
            manufacturer, firmware, hardware
        )));
    }

    content.push(Line::from(""));

    if let Some(battery_level) = &app.battery_level {
        let battery_bar = create_battery_bar(battery_level.charge_pct);
        let charging_text = if battery_level.is_charging {
            "⚡ Charging"
        } else {
            ""
        };

        content.push(Line::from(vec![
            Span::raw("🔋 Battery: "),
            Span::styled(
                format!("{}%", battery_level.charge_pct),
                get_battery_color(battery_level.charge_pct),
            ),
            Span::raw(" "),
            Span::styled(battery_bar, get_battery_color(battery_level.charge_pct)),
            Span::raw(charging_text.to_string()),
        ]));
    } else {
        content.push(Line::from("🔋 Battery: Press [b] to check"));
    }

    content.push(Line::from(""));
    content.push(Line::from("Live:"));

    if app.is_monitoring {
        let elapsed = app
            .monitor_started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        content.push(Line::from(format!(
            "  ● LIVE  {}s elapsed  |  {} readings  |  [l] stop",
            elapsed,
            app.live_readings.len()
        )));
        if let Some(latest) = app.live_readings.last() {
            content.push(Line::from(format!(
                "  Latest: {} {}",
                latest.value,
                latest.reading_type.unit()
            )));
            content.push(Line::from(format!(
                "  {}",
                render_sparkline(&app.live_readings)
            )));
        } else {
            content.push(Line::from("  Warming up (~30s)..."));
        }
    } else {
        content.push(Line::from(
            "  Press [l] to start live heart-rate monitoring",
        ));
    }

    render_status_and_footer(&mut content, app);

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Live"),
    );

    f.render_widget(paragraph, area);
}

fn render_today_tab(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Today",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    if let Some((heart_rate, steps, sleep, oxygen)) = &app.history {
        match heart_rate {
            HeartRateResult::Log(log) => {
                let readings: Vec<u8> =
                    log.heart_rates.iter().copied().filter(|&r| r > 0).collect();
                if readings.is_empty() {
                    content.push(Line::from("  🫀  Heart rate: no readings"));
                } else {
                    let avg =
                        readings.iter().map(|&r| r as u32).sum::<u32>() / readings.len() as u32;
                    let min = readings.iter().min().unwrap();
                    let max = readings.iter().max().unwrap();
                    content.push(Line::from(format!(
                        "  🫀  Heart rate: {} readings, avg {} bpm ({} - {})",
                        readings.len(),
                        avg,
                        min,
                        max
                    )));
                }
            }
            HeartRateResult::NoData => content.push(Line::from("  🫀  Heart rate: no data")),
        }

        match steps {
            StepsResult::Details(details) => {
                let total_steps: u32 = details.iter().map(|d| d.steps as u32).sum();
                let total_calories: f64 = details.iter().map(|d| d.calories).sum();
                let total_distance: u32 = details.iter().map(|d| d.distance as u32).sum();
                content.push(Line::from(format!(
                    "  👟  Steps: {} | {:.0} kcal | {} m",
                    total_steps, total_calories, total_distance
                )));
            }
            StepsResult::NoData => content.push(Line::from("  👟  Steps: no data")),
        }

        if let Some(day) = sleep.days.first() {
            let total: u16 = day.phases.iter().map(|p| p.minutes as u16).sum();
            content.push(Line::from(format!(
                "  😴  Sleep: {}h {:02}m ({}:{:02} → {}:{:02})",
                total / 60,
                total % 60,
                day.start_minutes / 60,
                day.start_minutes % 60,
                day.end_minutes / 60,
                day.end_minutes % 60
            )));
        } else {
            content.push(Line::from("  😴  Sleep: no data"));
        }

        if let Some(day) = oxygen.days.first() {
            let valid: Vec<_> = day
                .samples
                .iter()
                .filter(|s| s.min > 0 || s.max > 0)
                .collect();
            if valid.is_empty() {
                content.push(Line::from("  🫁  SpO2: no samples"));
            } else {
                let min_avg: u32 =
                    valid.iter().map(|s| s.min as u32).sum::<u32>() / valid.len() as u32;
                let max_avg: u32 =
                    valid.iter().map(|s| s.max as u32).sum::<u32>() / valid.len() as u32;
                content.push(Line::from(format!(
                    "  🫁  SpO2: {} samples, avg range {}–{}%",
                    valid.len(),
                    min_avg,
                    max_avg
                )));
            }
        } else {
            content.push(Line::from("  🫁  SpO2: no data"));
        }
    } else {
        content.push(Line::from("  Press [h] to fetch today's data"));
    }

    render_status_and_footer(&mut content, app);

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Today"),
    );

    f.render_widget(paragraph, area);
}

fn render_controls_tab(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Device Controls",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("[1] Blink  [2] Find  [3] Reboot  [4] Reset"),
        Line::from(""),
        Line::from("[l] Live heart-rate monitoring"),
        Line::from("[h] Refresh today's data"),
        Line::from("[b] Refresh battery"),
    ];

    render_status_and_footer(&mut content, app);

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Controls"),
    );

    f.render_widget(paragraph, area);
}

fn render_confirm_reset_screen(f: &mut Frame, area: Rect, app: &App) {
    let device_name = app
        .connected_device
        .as_ref()
        .map(|d| d.name())
        .unwrap_or("device");

    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Confirm Reset",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!("Reset {}?", device_name)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[4]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Confirm Reset"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[ESC]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Reset Confirmation")
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.current_screen {
        Screen::Idle => "[s] Scan | [q] Quit",
        Screen::Scanning => "[ESC] Stop | Found: 0 devices",
        Screen::DeviceList => "[↑/↓] Select | [ENTER] Choose | [ESC] Back | [s] Rescan",
        Screen::Error => "[ESC] Back",
        Screen::Connecting => "[ESC] Cancel | Connecting...",
        Screen::Connected => "[l] Live HR | [h] Today | [b] Battery | [q] Quit",
        Screen::ConfirmReset => "[4] Confirm Reset | [ESC] Cancel",
    };

    let device_count = format!("Found: {} devices", app.devices.len());

    let footer_content = Line::from(vec![
        Span::styled(
            format!("Screen: {:?} | ", app.current_screen),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(help_text, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" | {}", device_count),
            Style::default().fg(Color::Green),
        ),
    ]);

    let footer = Paragraph::new(footer_content)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    f.render_widget(footer, area);
}
