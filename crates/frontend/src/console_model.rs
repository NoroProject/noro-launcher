use crate::state::LogEntry;
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

pub fn entry_line(entry: &LogEntry) -> String {
    format!(
        "{} {:>5} {}",
        time_label(entry.timestamp),
        level_label(entry.level),
        entry.text
    )
}

pub fn filtered_logs(
    logs: &[LogEntry],
    show_info: bool,
    show_warn: bool,
    show_error: bool,
    query: &str,
) -> Vec<LogEntry> {
    let query = query.trim().to_ascii_lowercase();
    logs.iter()
        .filter(|entry| match entry.level {
            GameLogLevel::Info => show_info,
            GameLogLevel::Warn => show_warn,
            GameLogLevel::Error => show_error,
        })
        .filter(|entry| query.is_empty() || entry.text.to_ascii_lowercase().contains(&query))
        .cloned()
        .collect()
}

pub fn joined_lines(logs: &[LogEntry]) -> String {
    logs.iter().map(entry_line).collect::<Vec<_>>().join("\n")
}
