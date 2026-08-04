//! Расширенное чтение и парсинг логов Minecraft (на основе PandoraLauncher).
//! Поддерживает log4j XML формат, очистку чувствительных данных (токены, пути) и классификацию уровней.

use bridge::{GameLogLevel, MessageToFrontend};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

static REPLACEMENTS: Lazy<[(Regex, &'static str); 8]> = Lazy::new(|| {
    [
        // ANSI color/control sequences from log4j console output.
        (Regex::new(r#"\x1b\[[0-9;?]*[ -/]*[@-~]"#).unwrap(), ""),
        // Замена токенов доступа
        (
            Regex::new(r#""SignedJWT: [^\s]+""#).unwrap(),
            "SignedJWT: *****",
        ),
        (
            Regex::new(r#""Session ID is [^\s)]+""#).unwrap(),
            "Session ID is *****",
        ),
        (
            Regex::new(r#""--accessToken, [^\s,]+""#).unwrap(),
            "--accessToken, *****",
        ),
        // Замена путей пользователя
        (Regex::new(r#"/home/[^/]+/"#).unwrap(), "/home/*****/"),
        (Regex::new(r#"/Users/[^/]+/"#).unwrap(), "/Users/*****/"),
        (
            Regex::new(r#"\\Users\\[^\\]+\\"#).unwrap(),
            "\\Users\\*****\\",
        ),
        (
            Regex::new(r#"\\\\Users\\\\[^/]+\\\\"#).unwrap(),
            "\\\\Users\\\\*****\\\\",
        ),
    ]
});

/// Очистить строку от чувствительных данных.
pub fn redact(string: &str) -> Cow<'_, str> {
    let mut replaced = Cow::Borrowed(string);
    for (regex, replacement) in &*REPLACEMENTS {
        let new = regex.replace_all(&replaced, *replacement);
        if let Cow::Owned(new_str) = new {
            replaced = Cow::Owned(new_str);
        }
    }
    replaced
}

/// Запустить чтение логов из stdout/stderr процесса.
pub async fn spawn_log_reader<R>(
    mut reader: R,
    server_id: Uuid,
    frontend: bridge::FrontendHandle,
    is_stderr: bool,
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

                    let redacted = redact(&line);
                    let (level, clean_text) = classify_log(&redacted, is_stderr);

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
