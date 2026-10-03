// Over 150 lines: the checks are under a hundred; the rest is tests that tamper
// with a signed manifest.
//! ed25519 signature checks on the build manifest.
//!
//! The public key comes from the bootstrapper (`NORO_SIGNING_PUBKEY` in the
//! environment, or `bootstrap.json`), or is baked in at compile time. Debug
//! builds without one derive it from the shared [`schema::DEV_SIGNING_SEED`].
//! Release builds never do: that seed is in the sources, so trusting it would
//! accept anything anyone signs.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use once_cell::sync::Lazy;

fn resolve_signing_pubkey_hex() -> Option<String> {
    if let Ok(val) = std::env::var("NORO_SIGNING_PUBKEY") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let boot_path = crate::directories::LauncherDirectories::new().bootstrap_file();
    if let Ok(raw) = std::fs::read_to_string(&boot_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(key) = v.get("signing_pubkey").and_then(|k| k.as_str()) {
                let trimmed = key.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    option_env!("NORO_SIGNING_PUBKEY").map(String::from)
}

fn parse_key(hex_str: &str) -> Result<VerifyingKey, &'static str> {
    let bytes = hex::decode(hex_str).map_err(|_| "not valid hex")?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| "must be 32 bytes")?;
    VerifyingKey::from_bytes(&arr).map_err(|_| "not a valid key")
}

/// `None` fails every check. A broken key used to panic on the first manifest
/// and take the launcher down with it; failing closed leaves it running and
/// says why in the log.
static VERIFYING_KEY: Lazy<Option<VerifyingKey>> =
    Lazy::new(|| match resolve_signing_pubkey_hex() {
        Some(hex_str) => match parse_key(&hex_str) {
            Ok(key) => Some(key),
            Err(e) => {
                tracing::error!("NORO_SIGNING_PUBKEY: {e}; every signature check will fail");
                None
            }
        },
        // Dev builds derive the key from the same seed the master uses.
        None if cfg!(debug_assertions) => {
            Some(ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED).verifying_key())
        }
        None => {
            tracing::error!("no signing key is configured; every signature check will fail");
            None
        }
    });

fn verify(msg: &[u8], signature: &Signature) -> bool {
    VERIFYING_KEY
        .as_ref()
        .is_some_and(|key| key.verify(msg, signature).is_ok())
}

pub fn verify_manifest(manifest: &schema::BuildManifest) -> bool {
    if manifest.signature.len() != 64 {
        return false;
    }
    let sig_bytes: [u8; 64] = match manifest.signature.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig_bytes);
    verify(&manifest.signing_bytes(), &signature)
}

/// Signature over raw bytes, used for the launcher's own update binary.
pub fn verify_bytes(data: &[u8], signature_b64: &str) -> bool {
    use base64::Engine;
    let Ok(sig_raw) = base64::engine::general_purpose::STANDARD.decode(signature_b64) else {
        return false;
    };
    let Ok(sig_bytes): Result<[u8; 64], _> = sig_raw.try_into() else {
        return false;
    };
    verify(data, &Signature::from_bytes(&sig_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer as _, SigningKey};
    use schema::{BuildManifest, FileEntry, FileSide, Modloader};

    /// Tests run without `NORO_SIGNING_PUBKEY`, so both sides fall back to the
    /// dev seed and the signatures line up.
    fn dev_key() -> SigningKey {
        SigningKey::from_bytes(&schema::DEV_SIGNING_SEED)
    }

    fn manifest() -> BuildManifest {
        BuildManifest {
            build_id: uuid::Uuid::nil(),
            server_id: uuid::Uuid::nil(),
            version: "1.0.0".into(),
            mc_version: "1.21.1".into(),
            modloader: Modloader::Fabric,
            modloader_version: Some("0.16.0".into()),
            main_class: "net.fabricmc.loader.impl.launch.knot.KnotClient".into(),
            jvm_args: Vec::new(),
            game_args: Vec::new(),
            assets_index_name: "1.21".into(),
            verified_files: vec![FileEntry {
                path: "mods/jei.jar".into(),
                sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709".into(),
                size: 1024,
                url: "http://localhost/files/da39a3ee".into(),
                side: FileSide::Client,
                executable: false,
                platform: None,
            }],
            artifact_kinds: Default::default(),
            unmanaged_paths: Vec::new(),
            path_rules: Vec::new(),
            blocked_files: Vec::new(),
            user_managed_paths: Vec::new(),
            optional_mods: Vec::new(),
            allow_optional_mod_suggestions: true,
            allow_personal_content: false,
            recommended_client_settings: Default::default(),
            personal_content: Vec::new(),
            signature: Vec::new(),
        }
    }

    fn signed() -> BuildManifest {
        let mut m = manifest();
        m.signature = dev_key().sign(&m.signing_bytes()).to_bytes().to_vec();
        m
    }

    #[test]
    fn accepts_manifest_signed_by_the_master_key() {
        assert!(verify_manifest(&signed()));
    }

    /// The property everything else rests on: swap a hash and the signature
    /// stops matching. Otherwise any jar can be handed to a player.
    #[test]
    fn rejects_manifest_with_a_swapped_file_hash() {
        let mut m = signed();
        m.verified_files[0].sha1 = "0000000000000000000000000000000000000000".into();
        assert!(!verify_manifest(&m));
    }

    #[test]
    fn rejects_manifest_with_an_extra_file() {
        let mut m = signed();
        let mut extra = m.verified_files[0].clone();
        extra.path = "mods/backdoor.jar".into();
        m.verified_files.push(extra);
        assert!(!verify_manifest(&m));
    }

    #[test]
    fn rejects_unsigned_and_malformed_signatures() {
        let mut m = manifest();
        assert!(!verify_manifest(&m), "empty signature");

        m.signature = vec![0u8; 64];
        assert!(!verify_manifest(&m), "all-zero signature");

        m.signature = vec![1u8; 63];
        assert!(!verify_manifest(&m), "wrong length");
    }

    /// Otherwise anyone running their own master could sign a build for someone
    /// else's launcher.
    #[test]
    fn rejects_manifest_signed_by_a_foreign_key() {
        let foreign = SigningKey::from_bytes(&[7u8; 32]);
        let mut m = manifest();
        m.signature = foreign.sign(&m.signing_bytes()).to_bytes().to_vec();
        assert!(!verify_manifest(&m));
    }

    #[test]
    fn a_malformed_key_is_an_error_not_a_panic() {
        assert!(parse_key("zz").is_err());
        assert!(parse_key("abcd").is_err());
        assert!(parse_key(&hex::encode(dev_key().verifying_key().to_bytes())).is_ok());
    }

    #[test]
    fn verifies_launcher_binary_signature() {
        use base64::Engine;
        let binary = b"launcher binary bytes";
        let sig =
            base64::engine::general_purpose::STANDARD.encode(dev_key().sign(binary).to_bytes());

        assert!(verify_bytes(binary, &sig));
        assert!(!verify_bytes(b"tampered binary bytes", &sig));
        assert!(!verify_bytes(binary, "not base64 at all !!!"));
    }
}
