//! Расширенное чтение и парсинг логов Minecraft (на основе PandoraLauncher).
//!
//! Поддерживает log4j XML формат и классификацию уровней. Санитизация — в
//! [`crate::redact`]: те же правила нужны и файлам, а не только живому потоку.

use bridge::{GameLogLevel, MessageToFrontend};
use schema::redact;
use std::borrow::Cow;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

/// Данные для обновления Discord Rich Presence по строкам игрового лога.
pub struct RpcLogContext {
    pub rpc: crate::discord_rpc::DiscordRpc,
    pub server_name: String,
    pub start_timestamp: u64,
    pub online_current: Option<u32>,
    pub online_max: Option<u32>,
}

/// Запустить чтение логов из stdout/stderr процесса.
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

    loop {
        let mut chunk = [0u8; 4096];
        match reader.read(&mut chunk).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes = buffer.drain(..pos + 1).collect::<Vec<_>>();
                    let line = String::from_utf8_lossy(&line_bytes);
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

                    frontend.send(MessageToFrontend::GameLog {
                        server_id,
                        line: clean_text.into_owned(),
                        level,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    });
                }
            }
            Err(_) => break,
        }
    }
}

fn classify_log(line: &str, is_stderr: bool) -> (GameLogLevel, Cow<'_, str>) {
    // Если лог в формате log4j XML (упрощенный поиск)
    if line.contains("<log4j:Event") {
        if line.contains("level=\"FATAL\"") || line.contains("level=\"ERROR\"") {
            return (GameLogLevel::Error, Cow::Borrowed(line));
        }
        if line.contains("level=\"WARN\"") {
            return (GameLogLevel::Warn, Cow::Borrowed(line));
        }
        return (GameLogLevel::Info, Cow::Borrowed(line));
    }

    // Обычные текстовые логи (проверка маркеров)
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

    // Если нет четких маркеров, ориентируемся на поток
    if is_stderr {
        if upper.contains("ERROR") {
            (GameLogLevel::Error, Cow::Borrowed(line))
        } else if upper.contains("WARN") {
            (GameLogLevel::Warn, Cow::Borrowed(line))
        } else {
            // Многие инструменты (в т.ч. authlib-injector) пишут инфо в stderr.
            // Если нет слова ERROR/WARN, считаем это Info.
            (GameLogLevel::Info, Cow::Borrowed(line))
        }
    } else {
        (GameLogLevel::Info, Cow::Borrowed(line))
    }
}
