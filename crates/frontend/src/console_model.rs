//! Labels the console puts on a line.

use bridge::GameLogLevel;
use chrono::{DateTime, Local, TimeZone};

pub fn level_label(level: GameLogLevel) -> &'static str {
    match level {
        GameLogLevel::Error => "ERROR",
        GameLogLevel::Warn => "WARN",
        GameLogLevel::Info => "INFO",
    }
}

pub fn time_label(timestamp: i64) -> String {
    Local
        .timestamp_millis_opt(timestamp)
        .single()
        .map(|t: DateTime<Local>| t.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "00:00:00".to_string())
}
