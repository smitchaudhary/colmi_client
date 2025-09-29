use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tui::app::{App, Screen};

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
    let device_name = app.connecting_device_name.as_deref().unwrap_or("Unknown Device");

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
        Line::from(vec![Span::styled("Press ESC to cancel", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Connecting"),
    );

    f.render_widget(paragraph, area);
}

fn render_connected_screen(f: &mut Frame, area: Rect, app: &App) {
    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Connected!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Connected to device"),
        Line::from(""),
        Line::from(vec![Span::raw("Status: "), Span::styled("Connected", Style::default().fg(Color::White))]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled("Press ESC to disconnect", Style::default().fg(Color::Red))]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Connected"),
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
        Screen::Connected => "[q] Quit | Connected",
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
