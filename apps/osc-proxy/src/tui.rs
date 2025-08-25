use std::collections::HashMap;
use std::io;
use std::time::{SystemTime, Duration, Instant};

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use tokio::sync::mpsc;
use tracing::debug;

/// Connection status for display
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Online,
    Offline,
    Reconnecting,
    Failed,
}

impl ConnectionStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            ConnectionStatus::Online => "🟢",
            ConnectionStatus::Offline => "🔴",
            ConnectionStatus::Reconnecting => "🟡",
            ConnectionStatus::Failed => "🔴",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            ConnectionStatus::Online => Color::Green,
            ConnectionStatus::Offline => Color::Red,
            ConnectionStatus::Reconnecting => Color::Yellow,
            ConnectionStatus::Failed => Color::Red,
        }
    }
    
    pub fn text(&self) -> &'static str {
        match self {
            ConnectionStatus::Online => "Online",
            ConnectionStatus::Offline => "Offline",
            ConnectionStatus::Reconnecting => "Reconnecting...",
            ConnectionStatus::Failed => "Failed",
        }
    }
}

/// Target information for display
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub name: String,
    pub address: String,
    pub transport: String,
    pub status: ConnectionStatus,
    pub last_packet_time: Option<SystemTime>,
    pub packet_count: u64,
    pub next_attempt_at: Option<Instant>,
}

/// Log entry for display
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// UI event types
#[derive(Debug)]
pub enum UiEvent {
    TargetInfo {
        name: String,
        address: String,
        transport: String,
        status: ConnectionStatus,
        next_attempt_at: Option<Instant>,
    },
    LogEntry(LogEntry),
    PacketReceived {
        target: String,
        packet: String,
    },
}

/// Application state
pub struct App {
    /// Target information
    targets: HashMap<String, TargetInfo>,
    /// Log entries (latest first)
    logs: Vec<LogEntry>,
    /// Maximum number of log entries to keep
    max_logs: usize,
    /// Current log scroll position
    log_scroll: usize,
    /// Should quit
    should_quit: bool,
    /// Help dialog state
    show_help: bool,
    /// UI event receiver
    ui_rx: mpsc::UnboundedReceiver<UiEvent>,
}

impl App {
    pub fn new(
        targets: HashMap<String, TargetInfo>,
        ui_rx: mpsc::UnboundedReceiver<UiEvent>,
    ) -> Self {
        Self {
            targets,
            logs: Vec::new(),
            max_logs: 1000,
            log_scroll: 0,
            should_quit: false,
            show_help: false,
            ui_rx,
        }
    }
    
    pub fn handle_event(&mut self, event: Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('h') | KeyCode::F(1) => {
                        self.show_help = !self.show_help;
                    }
                    KeyCode::Up => {
                        if self.log_scroll > 0 {
                            self.log_scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.log_scroll < self.logs.len().saturating_sub(1) {
                            self.log_scroll += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        self.log_scroll = self.log_scroll.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        self.log_scroll = (self.log_scroll + 10).min(self.logs.len().saturating_sub(1));
                    }
                    KeyCode::Home => {
                        self.log_scroll = 0;
                    }
                    KeyCode::End => {
                        self.log_scroll = self.logs.len().saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    pub fn handle_ui_events(&mut self) {
        // Process all available UI events
        while let Ok(event) = self.ui_rx.try_recv() {
            match event {
                UiEvent::TargetInfo { name, address, transport, status, next_attempt_at } => {
                    if let Some(target) = self.targets.get_mut(&name) {
                        // Update existing target
                        target.status = status;
                        target.address = address;
                        target.transport = transport;
                        target.next_attempt_at = next_attempt_at;
                        debug!("Updated target {} info", name);
                    } else {
                        // Create new target
                        debug!("Adding new target: {}", name);
                        let target_info = TargetInfo {
                            name: name.clone(),
                            address,
                            transport,
                            status,
                            last_packet_time: None,
                            packet_count: 0,
                            next_attempt_at,
                        };
                        self.targets.insert(name, target_info);
                    }
                }
                UiEvent::LogEntry(log_entry) => {
                    self.logs.insert(0, log_entry);
                    if self.logs.len() > self.max_logs {
                        self.logs.truncate(self.max_logs);
                    }
                    // Reset scroll to show latest logs
                    self.log_scroll = 0;
                }
                UiEvent::PacketReceived { target, packet: _ } => {
                    if let Some(target_info) = self.targets.get_mut(&target) {
                        target_info.packet_count += 1;
                        target_info.last_packet_time = Some(SystemTime::now());
                    }
                }
            }
        }
    }
    
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();
        
        // Main layout: logs (left 2/3) + targets (right 1/3)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)].as_ref())
            .split(size);
        
        // Draw logs panel
        self.draw_logs(frame, main_chunks[0]);
        
        // Draw targets panel  
        self.draw_targets(frame, main_chunks[1]);
        
        // Draw help dialog if shown
        if self.show_help {
            self.draw_help(frame, size);
        }
    }
    
    fn draw_logs(&self, frame: &mut Frame, area: Rect) {
        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        let logs_area = logs_block.inner(area);
        frame.render_widget(logs_block, area);
        
        // Create log items
        let log_items: Vec<ListItem> = self.logs
            .iter()
            .skip(self.log_scroll)
            .take(logs_area.height as usize)
            .map(|log| {
                let timestamp = log.timestamp.format("%H:%M:%S%.3f");
                let level_style = match log.level.as_str() {
                    "ERROR" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "WARN" => Style::default().fg(Color::Yellow),
                    "INFO" => Style::default().fg(Color::Green),
                    "DEBUG" => Style::default().fg(Color::Blue),
                    "TRACE" => Style::default().fg(Color::Magenta),
                    _ => Style::default().fg(Color::White),
                };
                
                let content = vec![Line::from(vec![
                    Span::styled(format!("{} ", timestamp), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:5} ", log.level), level_style),
                    Span::styled(format!("{}: ", log.target), Style::default().fg(Color::Cyan)),
                    Span::raw(&log.message),
                ])];
                
                ListItem::new(Text::from(content))
            })
            .collect();
        
        let logs_list = List::new(log_items)
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(logs_list, logs_area);
        
        // Show scroll indicator
        if self.logs.len() > logs_area.height as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            
            let mut scrollbar_state = ScrollbarState::default()
                .content_length(self.logs.len())
                .position(self.log_scroll);
                
            frame.render_stateful_widget(
                scrollbar,
                area,
                &mut scrollbar_state,
            );
        }
    }
    
    fn draw_targets(&self, frame: &mut Frame, area: Rect) {
        let targets_block = Block::default()
            .title(" Targets ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let targets_area = targets_block.inner(area);
        frame.render_widget(targets_block, area);
        
        // Create target list items
        let target_items: Vec<ListItem> = self.targets
            .values()
            .map(|target| {
                let status_emoji = target.status.emoji();
                let status_color = target.status.color();
                let status_text = target.status.text();
                
                let packet_info = if target.packet_count > 0 {
                    format!(" ({})", target.packet_count)
                } else {
                    String::new()
                };
                
                let timing_info = if let Some(next_attempt_at) = target.next_attempt_at {
                    // Calculate remaining time until next attempt
                    if next_attempt_at > Instant::now() {
                        let remaining = next_attempt_at - Instant::now();
                        let seconds = remaining.as_secs();
                        if seconds > 0 {
                            format!(" (retry in {}s)", seconds)
                        } else {
                            " (retrying...)".to_string()
                        }
                    } else {
                        // Time has passed, should be retrying
                        " (retrying...)".to_string()
                    }
                } else {
                    String::new()
                };
                
                let content = vec![
                    Line::from(vec![
                        Span::raw(format!("{} ", status_emoji)),
                        Span::styled(&target.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(status_text, Style::default().fg(status_color)),
                        Span::styled(packet_info, Style::default().fg(Color::DarkGray)),
                        Span::styled(timing_info, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&target.transport, Style::default().fg(Color::Cyan)),
                        Span::raw(" "),
                        Span::styled(&target.address, Style::default().fg(Color::DarkGray)),
                    ]),
                ];
                
                let mut lines = content;
                lines.push(Line::raw(""));
                
                ListItem::new(Text::from(lines))
            })
            .collect();
        
        let targets_list = List::new(target_items);
        frame.render_widget(targets_list, targets_area);
    }
    
    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        // Calculate popup area (centered, 60% width, 50% height)
        let popup_area = centered_rect(60, 50, area);
        
        // Clear the area
        frame.render_widget(Clear, popup_area);
        
        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let help_text = vec![
            Line::from("OSC Proxy - Keyboard Shortcuts"),
            Line::raw(""),
            Line::from(vec![
                Span::styled("Ctrl+C", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" : Quit application"),
            ]),
            Line::from(vec![
                Span::styled("h, F1", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("     : Toggle this help"),
            ]),
            Line::raw(""),
            Line::from("Log Navigation:"),
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("       : Scroll logs line by line"),
            ]),
            Line::from(vec![
                Span::styled("PgUp/PgDn", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  : Scroll logs page by page"),
            ]),
            Line::from(vec![
                Span::styled("Home/End", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  : Go to top/bottom of logs"),
            ]),
            Line::raw(""),
            Line::from("Target Status:"),
            Line::from("  🟢 Online      🟡 Reconnecting"),
            Line::from("  🔴 Offline     🔴 Failed"),
        ];
        
        let help_paragraph = Paragraph::new(Text::from(help_text))
            .block(help_block)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(help_paragraph, popup_area);
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Run the TUI application
pub async fn run_tui(
    initial_targets: HashMap<String, TargetInfo>,
    ui_rx: mpsc::UnboundedReceiver<UiEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(initial_targets, ui_rx);
    
    // Main loop
    loop {
        // Handle UI events from other tasks
        app.handle_ui_events();
        
        // Draw UI
        terminal.draw(|f| app.draw(f))?;
        
        // Handle input events (non-blocking) - poll every 100ms for smoother countdown updates
        if event::poll(Duration::from_millis(100))? {
            app.handle_event(event::read()?)?;
        }
        
        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}