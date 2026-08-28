//! Session tokens in the OS keyring, encrypted by the OS.

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

pub fn save(auth: &StoredAuth) -> Result<()> {
    let json = serde_json::to_string(auth)?;
    entry()?
        .set_password(&json)
        .context("keyring write failed")?;
    Ok(())
}

/// Every failure here is `None` — a keyring the launcher can't reach is
/// indistinguishable from never having logged in, and both mean sign in again.
pub fn load() -> Option<StoredAuth> {
    let e = match entry() {
        Ok(e) => e,
        Err(err) => {
            tracing::error!("keyring: cannot open the entry: {err}");
            return None;
        }
    };
    let json = match e.get_password() {
        Ok(j) => j,
        Err(keyring::Error::NoEntry) => {
            tracing::debug!("keyring: no stored session");
            return None;
        }
        Err(err) => {
            tracing::error!("keyring: cannot read the stored session: {err}");
            return None;
        }
    };
    match serde_json::from_str(&json) {
        Ok(auth) => Some(auth),
        Err(err) => {
            tracing::error!("keyring: stored session is not valid json: {err}");
            None
        }
    }
}

/// Logging out. A missing entry counts as success.
pub fn clear() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("keyring delete failed"),
    }
}
