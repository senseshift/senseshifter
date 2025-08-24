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

/// Setup logging with both console and TUI output
pub fn setup_logging(
    log_level: &str,
    file_logging: bool,
    log_file: &str,
    ui_sender: mpsc::UnboundedSender<UiEvent>,
) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter, Registry};
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    let registry = Registry::default().with(env_filter);
    
    // Add TUI layer
    let registry = registry.with(TuiLayer::new(ui_sender));
    
    // Add file logging if enabled
    if file_logging {
        use tracing_appender::rolling::{RollingFileAppender, Rotation};
        
        let file_appender = RollingFileAppender::new(Rotation::DAILY, ".", log_file);
        let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);
        
        let registry = registry.with(
            fmt::Layer::default()
                .with_writer(non_blocking_file)
                .with_ansi(false)
        );
        
        tracing::subscriber::set_global_default(registry)?;
        
        // We need to keep the guard alive for the duration of the program
        // In a real application, you'd want to store this somewhere
        std::mem::forget(_guard);
    } else {
        tracing::subscriber::set_global_default(registry)?;
    }
    
    Ok(())
}