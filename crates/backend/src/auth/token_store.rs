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

/// On Linux the default store is keyutils backed by the Secret Service, so the
/// session survives a reboot. Some desktops have no Secret Service at all — a
/// bare window manager, a keyring that was never set up — and there the write
/// fails as a whole; the kernel keyring alone still keeps the session until the
/// next reboot, which beats asking for a login on every start.
#[cfg(target_os = "linux")]
fn session_only_entry() -> Result<keyring::Entry> {
    let credential = keyring::keyutils::KeyutilsCredential::new_with_target(None, SERVICE, ACCOUNT)
        .context("keyutils entry init failed")?;
    Ok(keyring::Entry::new_with_credential(Box::new(credential)))
}

pub fn save(auth: &StoredAuth) -> Result<()> {
    let json = serde_json::to_string(auth)?;
    let saved = entry()?.set_password(&json);
    #[cfg(target_os = "linux")]
    if let Err(e) = &saved {
        tracing::warn!("keyring: no persistent store ({e}), keeping the session until reboot");
        return session_only_entry()?
            .set_password(&json)
            .context("keyring write failed");
    }
    saved.context("keyring write failed")
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
    // The kernel-only copy a missing Secret Service left behind goes too,
    // or the next start would sign the player back in.
    #[cfg(target_os = "linux")]
    if let Ok(e) = session_only_entry() {
        let _ = e.delete_credential();
    }
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("keyring delete failed"),
    }
}
