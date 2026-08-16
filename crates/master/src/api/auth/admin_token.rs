//! Выпуск и проверка admin-токенов.
//!
//! Токен состоит из распознаваемого префикса и 32 случайных байт. Префикс
//! нужен не красоте: по нему секрет находят сканеры репозиториев и логов, а
//! человек с первого взгляда понимает, что именно он нашёл.
//!
//! Хранится токен в двух видах, и это не дублирование:
//!
//! * `token_lookup` — SHA-256, селектор. Argon2 солёный, искать по нему строку
//!   нельзя, а перебирать все токены по одной argon2-проверке на каждый — это
//!   ровно та стоимость, ради которой argon2 и выбирают.
//! * `token_hash` — argon2, верификатор. Именно он решает, владеет ли клиент
//!   секретом.
//!
//! Разделение убирает то, чем плоха была старая схема: содержимое таблицы
//! больше не является рабочим токеном.

use anyhow::{anyhow, Result};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::Rng;
use sha2::{Digest, Sha256};

/// `at` — admin token. Отличает его от кодов и сессий, которые тоже `noro_`.
pub const PREFIX: &str = "noro_at_";

/// Выпустить новый секрет. Показывается один раз и больше нигде не хранится.
pub fn generate() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    format!("{PREFIX}{}", hex::encode(bytes))
}

/// Селектор для поиска строки.
pub fn lookup(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

/// Верификатор в формате PHC.
pub fn hash(secret: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow!("не захешировать admin-токен: {e}"))
}

/// Владеет ли клиент секретом.
pub fn verify(secret: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(secret.as_bytes(), &parsed)
            .is_ok(),
        Err(e) => {
            // Строка в БД испорчена: молча отвечать «не подошло» нельзя, иначе
            // токен просто перестаёт работать без единого следа.
            tracing::error!(error = %e, "admin-токен: не разобрать argon2-хеш из БД");
            false
        }
    }
}

#[cfg(test)]
#[path = "admin_token_tests.rs"]
mod tests;
