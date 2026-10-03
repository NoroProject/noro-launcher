// Over 150 lines: the buffer and its filters, then tests. The code alone is
// under two hundred.
use crate::state::LogEntry;
use bridge::GameLogLevel;
use chrono::{DateTime, Local, TimeZone};
use std::collections::VecDeque;

/// Lines kept per server. A modded start prints thousands of lines before the
/// window even opens, and the error that matters is usually among the first.
pub const MAX_LOG_LINES: usize = 5000;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filters {
    pub info: bool,
    pub warn: bool,
    pub error: bool,
    /// Lowercased already; empty means no text filter.
    pub query: String,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            info: true,
            warn: true,
            error: true,
            query: String::new(),
        }
    }
}

impl Filters {
    fn admits(&self, entry: &LogEntry) -> bool {
        let level = match entry.level {
            GameLogLevel::Info => self.info,
            GameLogLevel::Warn => self.warn,
            GameLogLevel::Error => self.error,
        };
        level && (self.query.is_empty() || entry.text.to_lowercase().contains(&self.query))
    }
}

/// What an append did to the visible rows, for `ListState::splice`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Change {
    /// Visible rows dropped from the front (old lines pushed out).
    pub removed_front: usize,
    /// Visible rows added at the end.
    pub added_back: usize,
}

/// A server's log as the console shows it: a bounded buffer, plus which of its
/// lines pass the filters. Visible rows are kept by line number, so adding a
/// batch costs the batch rather than a pass over the whole log — the console
/// used to refilter and copy all of it for every batch and every frame.
#[derive(Default)]
pub struct ConsoleBuffer {
    lines: VecDeque<LogEntry>,
    /// Number of the line at `lines[0]`; grows as old lines are dropped.
    first: u64,
    visible: VecDeque<u64>,
    filters: Filters,
}

impl ConsoleBuffer {
    pub fn from_lines(lines: impl IntoIterator<Item = LogEntry>, filters: Filters) -> Self {
        let mut buffer = ConsoleBuffer {
            filters,
            ..Default::default()
        };
        buffer.append(lines);
        buffer
    }

    pub fn filters(&self) -> &Filters {
        &self.filters
    }

    pub fn total(&self) -> usize {
        self.lines.len()
    }

    pub fn visible_len(&self) -> usize {
        self.visible.len()
    }

    pub fn visible(&self, row: usize) -> Option<&LogEntry> {
        let n = *self.visible.get(row)?;
        self.lines.get((n - self.first) as usize)
    }

    pub fn visible_entries(&self) -> impl Iterator<Item = &LogEntry> {
        (0..self.visible.len()).filter_map(|row| self.visible(row))
    }

    pub fn append(&mut self, batch: impl IntoIterator<Item = LogEntry>) -> Change {
        let mut change = Change::default();
        // Rows dropped from the front are old ones while any are left; past
        // that, a batch bigger than the buffer pushes out its own first lines,
        // which were never shown.
        let mut old_visible = self.visible.len();
        for entry in batch {
            let n = self.first + self.lines.len() as u64;
            if self.filters.admits(&entry) {
                self.visible.push_back(n);
                change.added_back += 1;
            }
            self.lines.push_back(entry);
        }
        while self.lines.len() > MAX_LOG_LINES {
            self.lines.pop_front();
            if self.visible.front() == Some(&self.first) {
                self.visible.pop_front();
                if old_visible > 0 {
                    old_visible -= 1;
                    change.removed_front += 1;
                } else {
                    change.added_back -= 1;
                }
            }
            self.first += 1;
        }
        change
    }

    /// Recompute which lines show. The caller resets the list afterwards.
    pub fn set_filters(&mut self, filters: Filters) {
        self.filters = filters;
        self.visible = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, e)| self.filters.admits(e))
            .map(|(i, _)| self.first + i as u64)
            .collect();
    }

    pub fn clear(&mut self) {
        self.first += self.lines.len() as u64;
        self.lines.clear();
        self.visible.clear();
    }

    /// The visible lines as text, for the clipboard.
    pub fn copy_text(&self) -> String {
        self.visible_entries()
            .map(entry_line)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, level: GameLogLevel) -> LogEntry {
        LogEntry {
            timestamp: 0,
            level,
            text: text.to_string(),
        }
    }

    #[test]
    fn appends_report_only_what_changed() {
        let mut b = ConsoleBuffer::default();
        let change = b.append([
            line("a", GameLogLevel::Info),
            line("b", GameLogLevel::Error),
        ]);
        assert_eq!(
            change,
            Change {
                removed_front: 0,
                added_back: 2
            }
        );
        assert_eq!(b.visible(1).unwrap().text, "b");
    }

    #[test]
    fn filters_hide_and_show_without_losing_lines() {
        let mut b = ConsoleBuffer::default();
        b.append([
            line("loading mods", GameLogLevel::Info),
            line("Missing texture", GameLogLevel::Warn),
            line("Crash in render", GameLogLevel::Error),
        ]);
        b.set_filters(Filters {
            info: false,
            ..Filters::default()
        });
        assert_eq!(b.visible_len(), 2);
        b.set_filters(Filters {
            query: "texture".into(),
            ..Filters::default()
        });
        assert_eq!(b.visible_len(), 1);
        assert_eq!(b.visible(0).unwrap().text, "Missing texture");
        b.set_filters(Filters::default());
        assert_eq!(b.visible_len(), 3);
    }

    #[test]
    fn old_lines_fall_off_the_front() {
        let mut b = ConsoleBuffer::default();
        b.append((0..MAX_LOG_LINES).map(|i| line(&i.to_string(), GameLogLevel::Info)));
        let change = b.append([line("new", GameLogLevel::Info)]);
        assert_eq!(
            change,
            Change {
                removed_front: 1,
                added_back: 1
            }
        );
        assert_eq!(b.total(), MAX_LOG_LINES);
        assert_eq!(b.visible(0).unwrap().text, "1");
        assert_eq!(b.visible(MAX_LOG_LINES - 1).unwrap().text, "new");
    }

    #[test]
    fn a_batch_larger_than_the_buffer_keeps_its_tail() {
        let mut b = ConsoleBuffer::default();
        b.append([line("old", GameLogLevel::Info)]);
        let change =
            b.append((0..MAX_LOG_LINES + 10).map(|i| line(&i.to_string(), GameLogLevel::Info)));
        assert_eq!(change.removed_front, 1);
        assert_eq!(change.added_back, MAX_LOG_LINES);
        assert_eq!(b.visible(0).unwrap().text, "10");
    }

    #[test]
    fn copy_takes_only_visible_lines() {
        let mut b = ConsoleBuffer::default();
        b.append([
            line("keep", GameLogLevel::Error),
            line("drop", GameLogLevel::Info),
        ]);
        b.set_filters(Filters {
            info: false,
            ..Filters::default()
        });
        let text = b.copy_text();
        assert!(text.contains("keep") && !text.contains("drop"), "{text}");
    }
}
