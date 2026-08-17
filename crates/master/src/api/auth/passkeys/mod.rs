//! Passkey: регистрация в кабинете и вход по WebAuthn.
//!
//! Подпись проверяется библиотекой `webauthn-rs` — вместе с challenge, origin,
//! rpId и счётчиком. До этого достаточно было знать `credential_id`, который
//! браузер отдаёт публично.

mod login;
mod register;

pub use login::{login_options, login_verify, verify as verify_login, LoginVerifyReq};
pub use register::{delete_passkey, list_passkeys, register_options, register_verify};

use crate::error::AppError;

/// Ответ на `/options`: опции для браузера плюс id состояния, которое мастер
/// придержал у себя. Клиент возвращает его в `/verify`.
#[derive(serde::Serialize)]
pub struct ChallengeRes<T> {
    pub state_id: uuid::Uuid,
    #[serde(flatten)]
    pub options: T,
}

/// Провал проверки не должен подсказывать, чего именно не хватило: подробности
/// уезжают в лог, наружу — одна формулировка.
fn reject(err: webauthn_rs::prelude::WebauthnError) -> AppError {
    tracing::warn!(error = %err, "проверка WebAuthn не пройдена");
    AppError::Unauthorized("That key did not match".into())
}
