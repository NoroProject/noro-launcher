//! Signature check for the core binary. The key is compiled in or stamped.
//! The signature separates a forgery from the real binary.

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::path::Path;

pub const SIG_SUFFIX: &str = ".sig";

use crate::embedded_config;

/// Hex from the stamped config, falling back to env var, bootstrap.json, or build-time.
pub fn raw_signing_pubkey() -> String {
    if let Some(cfg) = embedded_config::get_embedded_config() {
        if !cfg.pubkey.is_empty() && !cfg.pubkey.contains("__NORO_PUBKEY_PLACEHOLDER__") {
            return cfg.pubkey;
        }
    }
    if let Ok(val) = std::env::var("NORO_SIGNING_PUBKEY") {
        if !val.trim().is_empty() {
            return val.trim().to_string();
        }
    }
    from_bootstrap("signing_pubkey").unwrap_or_else(|| {
        option_env!("NORO_SIGNING_PUBKEY").unwrap_or_default().to_string()
    })
}

fn from_bootstrap(key: &str) -> Option<String> {
    let path = dirs::data_dir()?.join(schema::launcher_dir_name()).join("bootstrap.json");
    let content = std::fs::read_to_string(path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    val.get(key)?.as_str().filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string())
}

fn verifying_key() -> Result<VerifyingKey> {
    let hex_str = raw_signing_pubkey();
    if hex_str.is_empty() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        return Ok(sk.verifying_key());
    }
    let bytes = hex::decode(&hex_str).map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: not valid hex"))?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: expected 32 bytes"))?;
    VerifyingKey::from_bytes(&arr).map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: not a valid key"))
}

/// Stamped config first, then env var, then bootstrap.json, then build-time env var.
pub fn master_url() -> String {
    if let Some(cfg) = embedded_config::get_embedded_config() {
        if !cfg.master_url.is_empty() {
            return cfg.master_url;
        }
    }
    if let Ok(val) = std::env::var("NORO_MASTER_URL") {
        if !val.trim().is_empty() {
            return val.trim().to_string();
        }
    }
    from_bootstrap("master_url").unwrap_or_else(|| {
        option_env!("NORO_MASTER_URL").unwrap_or("http://localhost:8080").to_string()
    })
}

/// `signature_b64` in the form the master hands it over.
pub fn verify_bytes(data: &[u8], signature_b64: &str) -> Result<()> {
    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(signature_b64.trim())
        .map_err(|_| anyhow!("signature is not base64"))?;
    let sig: [u8; 64] = raw
        .try_into()
        .map_err(|_| anyhow!("signature is not 64 bytes"))?;
    verifying_key()?
        .verify(data, &Signature::from_bytes(&sig))
        .map_err(|_| anyhow!("signature does not match the built-in key"))
}

/// Kept next to the binary so it can be checked again with no network.
pub fn store(core_path: &Path, signature_b64: &str) -> Result<()> {
    let path = sig_path(core_path);
    std::fs::write(&path, signature_b64.trim().as_bytes())
        .map_err(|e| anyhow!("could not write {}: {e}", path.display()))
}

/// Runs on every launch, not just after a download.
pub fn verify_installed(core_path: &Path) -> Result<()> {
    let sig_file = sig_path(core_path);
    let signature = std::fs::read_to_string(&sig_file)
        .map_err(|_| anyhow!("no signature next to {}", core_path.display()))?;
    let bytes = std::fs::read(core_path)
        .map_err(|e| anyhow!("could not read {}: {e}", core_path.display()))?;
    verify_bytes(&bytes, &signature)
}

/// Drop a core that failed the check, signature included, so the next launch
/// downloads it again.
pub fn discard(core_path: &Path) {
    let _ = std::fs::remove_file(core_path);
    let _ = std::fs::remove_file(sig_path(core_path));
}

fn sig_path(core_path: &Path) -> std::path::PathBuf {
    let mut name = core_path.as_os_str().to_os_string();
    name.push(SIG_SUFFIX);
    std::path::PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The master signs the bytes with this key and base64s the result.
    #[test]
    fn accepts_signature_made_like_master() {
        use base64::Engine;
        use ed25519_dalek::Signer;

        let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        let payload = b"noro-launcher-core payload";
        let sig = base64::engine::general_purpose::STANDARD.encode(sk.sign(payload).to_bytes());

        verify_bytes(payload, &sig).expect("a signature from the master must verify");
        verify_bytes(b"tampered", &sig).unwrap_err();
    }

    #[test]
    fn round_trip_through_disk() {
        use base64::Engine;
        use ed25519_dalek::Signer;

        let dir = std::env::temp_dir().join(format!("noro-verify-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let core = dir.join("core.bin");
        let body = b"binary";
        std::fs::write(&core, body).unwrap();

        let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        let sig = base64::engine::general_purpose::STANDARD.encode(sk.sign(body).to_bytes());
        store(&core, &sig).unwrap();
        verify_installed(&core).expect("a freshly installed core verifies");

        // A binary swapped on disk has to be rejected.
        std::fs::write(&core, b"evil").unwrap();
        verify_installed(&core).unwrap_err();

        discard(&core);
        assert!(!core.exists() && !sig_path(&core).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
