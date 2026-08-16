//! RSA-подпись профилей Yggdrasil.
//!
//! Без неё игроки видят только свой скин. Свой клиент забирает у мастера сам, по
//! TLS, и доверяет ему по транспорту. А чужие профили приезжают клиенту от
//! игрового сервера — то есть от третьей стороны, — и там уже нужна подпись:
//! иначе любой сервер навязывал бы игрокам произвольные скины. Спецификация
//! authlib-injector прямо требует подпись в ответе `hasJoined`.
//!
//! Алгоритм задан не нами: Yggdrasil исторически SHA1withRSA (PKCS#1 v1.5), и
//! клиент проверяет подпись ключом, который мы объявили в `signaturePublicKey`.
//! Ed25519 из `signing.rs` сюда не подходит — он подписывает манифесты сборок.

use anyhow::{Context, Result};
use base64::Engine;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::signature::{SignatureEncoding, Signer};
use rsa::RsaPrivateKey;
use sha1::Sha1;
use std::path::Path;

/// 2048 бит: Mojang использует 4096, но проверку это не меняет — клиент берёт
/// параметры из нашего же публичного ключа, а 2048 быстрее на каждый запрос.
const BITS: usize = 2048;
const FILE: &str = "yggdrasil_rsa.pem";

pub struct ProfileSigner {
    signing: SigningKey<Sha1>,
    public_pem: String,
}

impl ProfileSigner {
    /// Читает ключ из `{data_dir}/yggdrasil_rsa.pem`, а если его нет — создаёт.
    ///
    /// Ротация безопасна: подпись считается на каждый запрос и нигде не хранится.
    /// Потому ключ и не вынесен в переменную окружения — терять его нечем.
    pub fn load_or_create(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join(FILE);
        let key = match std::fs::read_to_string(&path) {
            Ok(pem) => RsaPrivateKey::from_pkcs8_pem(&pem)
                .with_context(|| format!("{} не разобрать как ключ", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(path = %path.display(), "создаю ключ подписи профилей Yggdrasil");
                let key = RsaPrivateKey::new(&mut rand::rngs::OsRng, BITS)
                    .context("генерация RSA-ключа")?;
                let pem = key.to_pkcs8_pem(LineEnding::LF).context("сериализация ключа")?;
                std::fs::write(&path, pem.as_bytes())
                    .with_context(|| format!("запись {}", path.display()))?;
                restrict(&path)?;
                key
            }
            Err(e) => {
                return Err(e).with_context(|| format!("чтение {}", path.display()));
            }
        };

        let public_pem = key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .context("сериализация публичного ключа")?;
        Ok(Self {
            signing: SigningKey::<Sha1>::new(key),
            public_pem,
        })
    }

    /// PEM публичного ключа — уходит в `signaturePublicKey` корня ALI.
    pub fn public_key_pem(&self) -> &str {
        &self.public_pem
    }

    /// Подпись значения свойства. Подписывается именно base64-строка, как её
    /// видит клиент, а не исходный JSON.
    pub fn sign_property(&self, value: &str) -> String {
        let sig = self.signing.sign(value.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(sig.to_bytes())
    }
}

#[cfg(unix)]
fn restrict(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("права на {}", path.display()))
}

#[cfg(not(unix))]
fn restrict(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1v15::VerifyingKey;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::signature::Verifier;
    use rsa::RsaPublicKey;

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("noro-ygg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Клиент проверяет подпись ключом из `signaturePublicKey`. Если эта пара
    /// разойдётся, игроки снова увидят вместо чужих скинов дефолтные — и без
    /// единой ошибки в логах.
    #[test]
    fn the_published_key_verifies_the_signature_we_produce() {
        let dir = tempdir();
        let signer = ProfileSigner::load_or_create(&dir).unwrap();

        let value = "eyJ0ZXh0dXJlcyI6e319";
        let sig = signer.sign_property(value);

        let public = RsaPublicKey::from_public_key_pem(signer.public_key_pem()).unwrap();
        let verifying = VerifyingKey::<Sha1>::new(public);
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&sig)
            .unwrap();
        verifying
            .verify(value.as_bytes(), &bytes.as_slice().try_into().unwrap())
            .expect("подпись должна проверяться опубликованным ключом");
    }

    /// Ключ обязан пережить перезапуск: клиенты держат метаданные ALI до
    /// перезапуска игры, и смена ключа на ходу ломает уже выданные подписи.
    #[test]
    fn the_key_is_reused_across_restarts() {
        let dir = tempdir();
        let first = ProfileSigner::load_or_create(&dir).unwrap();
        let second = ProfileSigner::load_or_create(&dir).unwrap();
        assert_eq!(first.public_key_pem(), second.public_key_pem());
    }
}
