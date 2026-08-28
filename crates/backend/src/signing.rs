//! Проверка ed25519-подписи манифеста. Публичный ключ зашит в бинарник:
//! в production — через env `NORO_SIGNING_PUBKEY` (hex) при компиляции,
//! в dev — выводится из общего seed [`schema::DEV_SIGNING_SEED`].

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

static VERIFYING_KEY: Lazy<VerifyingKey> = Lazy::new(|| {
    match resolve_signing_pubkey_hex() {
        Some(hex_str) => {
            let bytes = hex::decode(hex_str).expect("NORO_SIGNING_PUBKEY: невалидный hex");
            let arr: [u8; 32] = bytes
                .try_into()
                .expect("NORO_SIGNING_PUBKEY: нужно 32 байта");
            VerifyingKey::from_bytes(&arr).expect("NORO_SIGNING_PUBKEY: невалидный ключ")
        }
        None => {
            // DEV: вывести публичный ключ из того же seed, что использует мастер.
            let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
            sk.verifying_key()
        }
    }
});

/// Проверить подпись манифеста сборки.
pub fn verify_manifest(manifest: &schema::BuildManifest) -> bool {
    if manifest.signature.len() != 64 {
        return false;
    }
    let sig_bytes: [u8; 64] = match manifest.signature.clone().try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig_bytes);
    let msg = manifest.signing_bytes();
    VERIFYING_KEY.verify(&msg, &signature).is_ok()
}

/// Проверить подпись произвольных байтов (для бинарника обновления лаунчера).
pub fn verify_bytes(data: &[u8], signature_b64: &str) -> bool {
    use base64::Engine;
    let Ok(sig_raw) = base64::engine::general_purpose::STANDARD.decode(signature_b64) else {
        return false;
    };
    let Ok(sig_bytes): Result<[u8; 64], _> = sig_raw.try_into() else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_bytes);
    VERIFYING_KEY.verify(data, &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer as _, SigningKey};
    use schema::{BuildManifest, FileEntry, FileSide, Modloader};

    /// Тесты идут без `NORO_SIGNING_PUBKEY`, поэтому проверяющий ключ выводится
    /// из общего dev-seed — тем же способом, что и на мастере.
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
            recommended_client_settings: Default::default(),
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

    /// Главное свойство: подменённый файл в сборке ломает подпись. Без этого
    /// игроку можно подсунуть любой jar.
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
        assert!(!verify_manifest(&m), "пустая подпись");

        m.signature = vec![0u8; 64];
        assert!(!verify_manifest(&m), "нулевая подпись");

        m.signature = vec![1u8; 63];
        assert!(!verify_manifest(&m), "подпись неверной длины");
    }

    /// Подпись, снятая с другого ключа, не должна проходить: иначе любой,
    /// кто поднял свой мастер, подписал бы сборку для чужого лаунчера.
    #[test]
    fn rejects_manifest_signed_by_a_foreign_key() {
        let foreign = SigningKey::from_bytes(&[7u8; 32]);
        let mut m = manifest();
        m.signature = foreign.sign(&m.signing_bytes()).to_bytes().to_vec();
        assert!(!verify_manifest(&m));
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
