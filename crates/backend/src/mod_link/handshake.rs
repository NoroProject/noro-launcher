//! Файл рукопожатия в каталоге инстанса: порт и одноразовый ключ.
//!
//! Мод знает только свой `gameDir`, поэтому файл лежит там, а не в конфиге
//! лаунчера. Ключ обязателен: сокет открыт наружу процесса, и чужая страница
//! может постучаться на `ws://127.0.0.1:port` — CORS на WebSocket не
//! распространяется. Прочитать файл в каталоге игры она при этом не может.

use anyhow::Result;
use mod_link::{Handshake, HANDSHAKE_FILE, PROTOCOL};
use std::path::{Path, PathBuf};

/// Одноразовый ключ. Два v4 подряд — 256 бит из системного генератора; заводить
/// ради этого `rand` в лаунчер незачем, `uuid` уже здесь.
pub fn new_key() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

pub fn path(instance_dir: &Path) -> PathBuf {
    instance_dir.join(HANDSHAKE_FILE)
}

pub async fn write(instance_dir: &Path, port: u16, key: &str) -> Result<()> {
    let body = serde_json::to_vec_pretty(&Handshake {
        port,
        key: key.to_string(),
        protocol: PROTOCOL,
    })?;
    tokio::fs::write(path(instance_dir), body).await?;
    Ok(())
}

/// Убрать файл. Игра закрылась — ключ протух, и оставлять его лежать значит
/// обещать доступ, которого больше нет.
pub async fn remove(instance_dir: &Path) {
    let _ = tokio::fs::remove_file(path(instance_dir)).await;
}
