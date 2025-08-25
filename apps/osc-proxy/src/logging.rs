use anyhow::Result;
use chrono::Local;
use std::fmt;
use tokio::sync::mpsc;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    Layer,
};

use crate::tui::{LogEntry, UiEvent};

/// Custom tracing layer that sends log entries to the TUI
pub struct TuiLayer {
    sender: mpsc::UnboundedSender<UiEvent>,
}

impl TuiLayer {
    pub fn new(sender: mpsc::UnboundedSender<UiEvent>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for TuiLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        
        // Extract the message from the event
        let mut visitor = LogVisitor::new();
        event.record(&mut visitor);
        
        let log_entry = LogEntry {
            timestamp: Local::now(),
            level: level_to_string(metadata.level()),
            target: metadata.target().to_string(),
            message: visitor.message,
        };
        
        // Send to TUI (ignore if channel is closed)
        let _ = self.sender.send(UiEvent::LogEntry(log_entry));
    }
}

struct LogVisitor {
    message: String,
}

impl LogVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(&format!("{}={:?}", field.name(), value));
        }
    }
    
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(&format!("{}={}", field.name(), value));
        }
    }
}

fn level_to_string(level: &Level) -> String {
    match *level {
        Level::ERROR => "ERROR".to_string(),
        Level::WARN => "WARN".to_string(),
        Level::INFO => "INFO".to_string(),
        Level::DEBUG => "DEBUG".to_string(),
        Level::TRACE => "TRACE".to_string(),
    }
}

/// Setup logging with TUI output only (no console output to avoid conflicts)
pub fn setup_logging(ui_sender: mpsc::UnboundedSender<UiEvent>) -> Result<()> {
    let tui_layer = TuiLayer::new(ui_sender);

    let subscriber = tracing_subscriber::registry()
        .with(tui_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set global subscriber: {}", e))?;

    Ok(())
}