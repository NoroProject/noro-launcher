//! Проверка подписи core-бинарника.
//!
//! Ключ зашивается на компиляции и в bootstrapper'е это **настоящий якорь
//! доверия**: этот бинарь по замыслу никогда не обновляется, значит подменить
//! ключ негде. Отсюда и правило ниже — sha256 из ответа мастера подтверждает
//! только целостность закачки: кто подменит канал, подставит и файл, и его хеш.
//! Отличает подделку от подлинника единственно подпись.

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::path::Path;

/// Имя файла с подписью рядом с core-бинарником.
pub const SIG_SUFFIX: &str = ".sig";

use crate::embedded_config;

/// Возвращает hex публичного ключа из вшитого конфига или env.
pub fn raw_signing_pubkey() -> String {
    if let Some(cfg) = embedded_config::get_embedded_config() {
        if !cfg.pubkey.is_empty() && !cfg.pubkey.contains("__NORO_PUBKEY_PLACEHOLDER__") {
            return cfg.pubkey;
        }
    }
    option_env!("NORO_SIGNING_PUBKEY").unwrap_or_default().to_string()
}

fn verifying_key() -> Result<VerifyingKey> {
    let hex_str = raw_signing_pubkey();
    if hex_str.is_empty() {
        // В dev ключ выводится из общего seed — того же, что у мастера.
        let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        return Ok(sk.verifying_key());
    }
    let bytes = hex::decode(&hex_str).map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: невалидный hex"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: нужно 32 байта"))?;
    VerifyingKey::from_bytes(&arr).map_err(|_| anyhow!("NORO_SIGNING_PUBKEY: невалидный ключ"))
}

/// Адрес мастера. Читается из впечатанного конфига, либо из env / localhost.
pub fn master_url() -> String {
    if let Some(cfg) = embedded_config::get_embedded_config() {
        if !cfg.master_url.is_empty() {
            return cfg.master_url;
        }
    }
    option_env!("NORO_MASTER_URL")
        .unwrap_or("http://localhost:8080")
        .to_string()
}

/// Проверяет ed25519-подпись байтов. `signature_b64` — как отдаёт мастер.
pub fn verify_bytes(data: &[u8], signature_b64: &str) -> Result<()> {
    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(signature_b64.trim())
        .map_err(|_| anyhow!("подпись не base64"))?;
    let sig: [u8; 64] = raw.try_into().map_err(|_| anyhow!("подпись не 64 байта"))?;
    verifying_key()?
        .verify(data, &Signature::from_bytes(&sig))
        .map_err(|_| anyhow!("подпись не совпала с зашитым ключом"))
}

/// Кладёт подпись рядом с бинарником, чтобы проверять её и без сети.
pub fn store(core_path: &Path, signature_b64: &str) -> Result<()> {
    let path = sig_path(core_path);
    std::fs::write(&path, signature_b64.trim().as_bytes())
        .map_err(|e| anyhow!("не записать {}: {e}", path.display()))
}

/// Перепроверяет уже установленный core на каждом запуске.
pub fn verify_installed(core_path: &Path) -> Result<()> {
    let sig_file = sig_path(core_path);
    let signature = std::fs::read_to_string(&sig_file)
        .map_err(|_| anyhow!("нет подписи рядом с {}", core_path.display()))?;
    let bytes = std::fs::read(core_path)
        .map_err(|e| anyhow!("не прочитать {}: {e}", core_path.display()))?;
    verify_bytes(&bytes, &signature)
}

/// Убирает битый core вместе с подписью, чтобы следующий запуск скачал заново.
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

    /// Контракт с мастером: тот подписывает байты тем же ключом и кодирует
    /// подпись в base64 (`launcher_builder::run_build`). Тест ловит расхождение
    /// на сборке, а не на машине игрока.
    #[test]
    fn accepts_signature_made_like_master() {
        use base64::Engine;
        use ed25519_dalek::Signer;

        let sk = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        let payload = b"noro-launcher-core payload";
        let sig = base64::engine::general_purpose::STANDARD.encode(sk.sign(payload).to_bytes());

        verify_bytes(payload, &sig).expect("подпись мастера должна проходить");
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
        verify_installed(&core).expect("свежеустановленный core проходит проверку");

        // Подменённый на диске бинарник обязан быть отвергнут.
        std::fs::write(&core, b"evil").unwrap();
        verify_installed(&core).unwrap_err();

        discard(&core);
        assert!(!core.exists() && !sig_path(&core).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
