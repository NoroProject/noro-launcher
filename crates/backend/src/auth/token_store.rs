//! Хранение токенов сессии в системном keyring (зашифровано ОС).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SERVICE: &str = "noro-launcher";
const ACCOUNT: &str = "session";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuth {
    pub access_token: String,
    pub refresh_token: String,
}

fn entry() -> Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, ACCOUNT).context("keyring entry init failed")
}

/// Сохранить токены.
pub fn save(auth: &StoredAuth) -> Result<()> {
    let json = serde_json::to_string(auth)?;
    entry()?
        .set_password(&json)
        .context("keyring write failed")?;
    Ok(())
}

/// Загрузить токены, если есть.
pub fn load() -> Option<StoredAuth> {
    let e = match entry() {
        Ok(e) => e,
        Err(err) => {
            tracing::error!("keyring: не удалось получить доступ к записи: {err}");
            return None;
        }
    };
    let json = match e.get_password() {
        Ok(j) => j,
        Err(keyring::Error::NoEntry) => {
            tracing::debug!("keyring: запись сессии не найдена");
            return None;
        }
        Err(err) => {
            tracing::error!("keyring: не удалось прочитать запись сессии: {err}");
            return None;
        }
    };
    match serde_json::from_str(&json) {
        Ok(auth) => Some(auth),
        Err(err) => {
            tracing::error!("keyring: не удалось десериализовать токены: {err}");
            None
        }
    }
}

/// Удалить токены (выход).
pub fn clear() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("keyring delete failed"),
    }
}
