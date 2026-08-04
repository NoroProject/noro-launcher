//! ed25519 подпись манифестов и проверка.

use anyhow::{Context, Result};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

#[derive(Clone)]
pub struct Signer25519 {
    key: SigningKey,
}

impl Signer25519 {
    /// Создать из конфигурации: реальный ключ из hex или dev-ключ из seed.
    pub fn from_config(signing_key_hex: &Option<String>) -> Result<Self> {
        let seed: [u8; 32] = match signing_key_hex {
            Some(hex_str) => {
                let bytes =
                    hex::decode(hex_str.trim()).context("NORO_SIGNING_KEY должен быть hex")?;
                bytes.try_into().map_err(|_| {
                    anyhow::anyhow!("NORO_SIGNING_KEY должен быть 32 байта (64 hex)")
                })?
            }
            None => schema::DEV_SIGNING_SEED,
        };
        Ok(Self {
            key: SigningKey::from_bytes(&seed),
        })
    }

    /// Подписать произвольные байты.
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.key.sign(data).to_bytes().to_vec()
    }

    /// Подписать манифест: записывает подпись в поле `signature`.
    pub fn sign_manifest(&self, manifest: &mut schema::BuildManifest) {
        manifest.signature.clear();
        let bytes = manifest.signing_bytes();
        manifest.signature = self.sign(&bytes);
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.key.verifying_key()
    }

    /// Публичный ключ в hex — выводится в лог, чтобы зашить в лаунчер.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key().to_bytes())
    }
}
