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

    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                let mut lines = Vec::new();
                // Walked by offset and trimmed once at the end: draining line by
                // line moved the rest of the buffer every time.
                let mut start = 0;
                while let Some(len) = buffer[start..].iter().position(|&b| b == b'\n') {
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

                    lines.push(GameLogLine {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        level,
                        text: clean_text.into_owned(),
                    });
                }
                buffer.drain(..start);
                if !lines.is_empty() {
                    frontend.send(MessageToFrontend::GameLog { server_id, lines });
                }
            }
            Err(_) => break,
        }
    }
}

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
