// Over 150 lines: reading the pipes and classifying the lines, then tests for
// both.
//! Reading and classifying Minecraft's log output (based on PandoraLauncher).
//!
//! Handles the log4j XML format as well as plain lines. Redaction lives in
//! [`schema::redact`], not here — log files need the same rules as the live
//! stream.

use bridge::{GameLogLevel, GameLogLine, MessageToFrontend};
use schema::redact;
use std::borrow::Cow;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

/// The game never reports what screen it's on, so Rich Presence is driven off
/// whatever the log happens to say.
pub struct RpcLogContext {
    pub rpc: crate::discord_rpc::DiscordRpc,
    pub server_name: String,
    pub start_timestamp: u64,
    pub online_current: Option<u32>,
    pub online_max: Option<u32>,
}

pub async fn spawn_log_reader<R>(
    mut reader: R,
    server_id: Uuid,
    frontend: bridge::FrontendHandle,
    is_stderr: bool,
    rpc_info: Option<RpcLogContext>,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut buffer = Vec::new();
    // Big enough that a burst comes out in a few reads, and every read goes to
    // the window as one message however many lines it holds.
    let mut chunk = vec![0u8; 64 * 1024];
    // A stack trace takes the level of the line that started it: its frames
    // carry none of their own.
    let mut last_level = GameLogLevel::Info;

    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                // A "line" this long with no break in it — progress output that
                // only ever returns the carriage — would grow the buffer without
                // bound. Cut it here and show what there is.
                if buffer.len() > MAX_LINE && !buffer.iter().any(|&b| b == b'\n' || b == b'\r') {
                    buffer.push(b'\n');
                }
                let mut lines = Vec::new();
                // Walked by offset and trimmed once at the end: draining line by
                // line moved the rest of the buffer every time.
                let mut start = 0;
                // `\r` ends a line too: progress bars redraw with it alone, and
                // `\r\n` just leaves an empty line, which is skipped.
                while let Some(len) = buffer[start..]
                    .iter()
                    .position(|&b| b == b'\n' || b == b'\r')
                {
                    let line_bytes = &buffer[start..start + len];
                    start += len + 1;
                    let line = String::from_utf8_lossy(line_bytes);
                    let line = line.trim_end();
                    if line.is_empty() {
                        continue;
                    }

                    let redacted = redact(line);
                    let (level, clean_text) = classify_log(&redacted, is_stderr);

                    if let Some(ref ctx) = rpc_info {
                        let lower = clean_text.to_lowercase();
                        if lower.contains("connecting to ") || lower.contains("joining world") {
                            ctx.rpc
                                .update(crate::discord_rpc::DiscordRpcState::GamePlaying {
                                    server_name: ctx.server_name.clone(),
                                    online_current: ctx.online_current,
                                    online_max: ctx.online_max,
                                    start_timestamp: ctx.start_timestamp,
                                });
                        } else if lower.contains("titlescreen")
                            || lower.contains("disconnecting from")
                        {
                            ctx.rpc
                                .update(crate::discord_rpc::DiscordRpcState::GameMenu {
                                    server_name: ctx.server_name.clone(),
                                    start_timestamp: ctx.start_timestamp,
                                });
                        }
                    }

                    let parsed = crate::log_line::parse(&clean_text);
                    let continuation = crate::log_line::continues(&clean_text, parsed.body);
                    let level = if parsed.thrown {
                        GameLogLevel::Error
                    } else if continuation {
                        last_level
                    } else {
                        level
                    };
                    if !continuation {
                        last_level = level;
                    }
                    lines.push(GameLogLine {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        level,
                        thread: parsed.thread.map(str::to_string),
                        logger: parsed.logger.map(str::to_string),
                        body: parsed.body.to_string(),
                        continuation,
                        text: clean_text.into_owned(),
                    });
                }
                buffer.drain(..start);
                if !lines.is_empty() {
                    frontend.send(MessageToFrontend::GameLog { server_id, lines });
                }
            }
            Err(e) => {
                // Keep draining even though nothing more can be shown. A pipe
                // nobody reads fills up, and the game then blocks on its next
                // write to stdout and freezes.
                tracing::warn!(error = %format!("{e:#}"), "game output stopped being readable");
                let _ = tokio::io::copy(&mut reader, &mut tokio::io::sink()).await;
                break;
            }
        }
    }
}

/// Longest stretch of output kept waiting for a line break.
const MAX_LINE: usize = 64 * 1024;

fn classify_log(line: &str, is_stderr: bool) -> (GameLogLevel, Cow<'_, str>) {
    // Substring matching rather than parsing: a line can be a fragment of the
    // XML, and a half-parsed event is worse than a guessed level.
    if line.contains("<log4j:Event") {
        if line.contains("level=\"FATAL\"") || line.contains("level=\"ERROR\"") {
            return (GameLogLevel::Error, Cow::Borrowed(line));
        }
        if line.contains("level=\"WARN\"") {
            return (GameLogLevel::Warn, Cow::Borrowed(line));
        }
        return (GameLogLevel::Info, Cow::Borrowed(line));
    }

    let upper = line.to_uppercase();
    // log4j names the level next to the thread: `[Client thread/WARN]`. That
    // one is authoritative — a message that merely says "severe" or
    // "exception" is not an error, and a WARN line is not INFO.
    for (tag, level) in [
        ("/FATAL]", GameLogLevel::Error),
        ("/ERROR]", GameLogLevel::Error),
        ("/WARN]", GameLogLevel::Warn),
        ("/INFO]", GameLogLevel::Info),
        ("/DEBUG]", GameLogLevel::Info),
        ("/TRACE]", GameLogLevel::Info),
    ] {
        if upper.contains(tag) {
            return (level, Cow::Borrowed(line));
        }
    }
    if upper.contains("[ERROR]")
        || upper.contains("[FATAL]")
        || upper.contains("SEVERE")
        || upper.contains("EXCEPTION")
    {
        return (GameLogLevel::Error, Cow::Borrowed(line));
    }
    if upper.contains("[WARN]") || upper.contains("[WARNING]") {
        return (GameLogLevel::Warn, Cow::Borrowed(line));
    }
    if upper.contains("[INFO]") {
        return (GameLogLevel::Info, Cow::Borrowed(line));
    }

    // No marker at all — fall back to which stream it came from.
    if is_stderr {
        if upper.contains("ERROR") {
            (GameLogLevel::Error, Cow::Borrowed(line))
        } else if upper.contains("WARN") {
            (GameLogLevel::Warn, Cow::Borrowed(line))
        } else {
            // authlib-injector and friends write ordinary progress to stderr,
            // so stderr on its own doesn't mean anything went wrong.
            (GameLogLevel::Info, Cow::Borrowed(line))
        }
    } else {
        (GameLogLevel::Info, Cow::Borrowed(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_thread_level_decides() {
        let warn = "[06:18:18] [Client thread/WARN] [AssetDirector]: Ignoring mod etfuturum";
        assert_eq!(classify_log(warn, false).0, GameLogLevel::Warn);
        // "severe" in the text of an INFO line used to make it an error.
        let info = "[06:18:08] [Client thread/INFO] [FML]: could severe stability issues";
        assert_eq!(classify_log(info, false).0, GameLogLevel::Info);
        let error = "[06:18:09] [Client thread/ERROR] [IC2]: expecting signature";
        assert_eq!(classify_log(error, false).0, GameLogLevel::Error);
    }
}

#[cfg(test)]
mod reader_tests {
    use super::*;

    async fn read_all(input: &'static [u8]) -> Vec<String> {
        let (handle, mut recv) = {
            let (_brx, _bh, frx, fh) = bridge::create_pair();
            (fh, frx)
        };
        spawn_log_reader(input, Uuid::nil(), handle, false, None).await;
        let mut out = Vec::new();
        while let Some(MessageToFrontend::GameLog { lines, .. }) = recv.try_recv() {
            out.extend(lines.into_iter().map(|l| l.text));
        }
        out
    }

    #[tokio::test]
    async fn carriage_returns_split_lines_too() {
        let lines = read_all(b"10%\r20%\r30%\r\ndone\n").await;
        assert_eq!(lines, ["10%", "20%", "30%", "done"]);
    }
}
