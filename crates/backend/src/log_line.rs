//! One line of the game's output taken apart: the game's own time, the thread,
//! the level, the logger and the message.
//!
//! Minecraft prints through log4j — `[06:18:16] [Client thread/INFO] [FML]:
//! Found 448 ObjectHolder annotations`. Shown whole, every line repeats forty
//! characters (a time the console already has, a thread) before it says
//! anything, and the level, the one part worth colouring, sits in the middle of
//! them. The prefix is read once here, when the line arrives.

use bridge::GameLogLevel;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed<'a> {
    pub thread: Option<&'a str>,
    pub level: Option<GameLogLevel>,
    pub logger: Option<&'a str>,
    pub body: &'a str,
    /// An exception printed with `printStackTrace`. FML routes that through
    /// log4j, which files it under INFO.
    pub thrown: bool,
}

pub fn parse(line: &str) -> Parsed<'_> {
    let mut out = Parsed::default();
    let mut rest = line;
    // Only the first few groups are the prefix: `[FML]: [AppEng] Core Init` —
    // the second bracket is already the message.
    for _ in 0..4 {
        let trimmed = rest.trim_start();
        let close = match trimmed.chars().next() {
            Some('[') => ']',
            // Fabric: `[main/INFO] (FabricLoader) Loading 123 mods`.
            Some('(') if out.level.is_some() && out.logger.is_none() => ')',
            _ => break,
        };
        let Some(end) = trimmed.find(close) else {
            break;
        };
        let inner = &trimmed[1..end];
        if out.thread.is_none() && out.level.is_none() && out.logger.is_none() && is_time(inner) {
            // The game's own clock — the console shows its own.
        } else if let Some((thread, level)) = thread_level(inner) {
            out.thread = Some(thread);
            out.level = Some(level);
        } else if let Some(level) = level_word(inner) {
            out.level = Some(level);
        } else if out.logger.is_none() && !inner.trim().is_empty() && inner.len() <= 80 {
            out.logger = Some(inner.trim_end_matches('/'));
        } else {
            break;
        }
        rest = &trimmed[end + 1..];
        if let Some(after) = rest.strip_prefix(':') {
            rest = after;
            break;
        }
    }
    out.body = rest.trim_start();

    // FML hands System.out and System.err to log4j with the caller in front:
    // `[pcl.opensecurity.util.SoundUnpack:load:32]: Extracting…`. The class
    // says more than `STDOUT`; `Throwable$WrappedPrintStream` means a stack
    // trace is being printed.
    if let Some((caller, message)) = redirected(out.body) {
        if caller.starts_with("java.lang.Throwable") {
            out.thrown = true;
        } else if out
            .logger
            .is_none_or(|l| l.eq_ignore_ascii_case("STDOUT") || l.eq_ignore_ascii_case("STDERR"))
        {
            out.logger = Some(caller.rsplit('.').next().unwrap_or(caller));
        }
        out.body = message;
    }
    out
}

/// A line that only continues the one above it: a frame of a stack trace, its
/// `Caused by`, the `... 12 more` at the end.
pub fn continues(raw: &str, body: &str) -> bool {
    let trimmed = body.trim_start();
    let indented = raw.starts_with([' ', '\t']) || body.starts_with([' ', '\t']);
    (indented && (trimmed.starts_with("at ") || trimmed.starts_with("...")))
        || trimmed.starts_with("Caused by:")
        || trimmed.starts_with("Suppressed:")
}

fn is_time(inner: &str) -> bool {
    inner.len() <= 32 && inner.contains(':') && inner.starts_with(|c: char| c.is_ascii_digit())
}

fn thread_level(inner: &str) -> Option<(&str, GameLogLevel)> {
    let (thread, level) = inner.rsplit_once('/')?;
    Some((thread, level_word(level)?))
}

fn level_word(word: &str) -> Option<GameLogLevel> {
    match word.trim() {
        "ERROR" | "FATAL" | "SEVERE" => Some(GameLogLevel::Error),
        "WARN" | "WARNING" => Some(GameLogLevel::Warn),
        "INFO" | "DEBUG" | "TRACE" | "CONFIG" | "FINE" | "FINER" | "FINEST" => {
            Some(GameLogLevel::Info)
        }
        _ => None,
    }
}

/// `[some.package.Class:method:12]: message` → (`some.package.Class`, `message`).
fn redirected(body: &str) -> Option<(&str, &str)> {
    let inner = body.strip_prefix('[')?;
    let (caller, message) = inner.split_once("]: ")?;
    let class = caller.split(':').next()?;
    (caller.contains(':') && class.contains('.') && !class.contains(' '))
        .then_some((class, message))
}

#[cfg(test)]
#[path = "log_line_tests.rs"]
mod tests;
