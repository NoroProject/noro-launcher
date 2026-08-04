//! Проверка ed25519-подписи манифеста. Публичный ключ зашит в бинарник:
//! в production — через env `NORO_SIGNING_PUBKEY` (hex) при компиляции,
//! в dev — выводится из общего seed [`schema::DEV_SIGNING_SEED`].

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use once_cell::sync::Lazy;

static VERIFYING_KEY: Lazy<VerifyingKey> = Lazy::new(|| {
    match option_env!("NORO_SIGNING_PUBKEY") {
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
