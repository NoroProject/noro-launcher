//! Passkeys: registration in the cabinet and WebAuthn sign-in.
//!
//! `webauthn-rs` does the verification — signature, challenge, origin, rpId and
//! the counter. Nothing here re-checks any of that by hand.

mod login;
mod register;

pub use login::{login_options, login_verify, verify as verify_login, LoginVerifyReq};
pub use register::{delete_passkey, list_passkeys, register_options, register_verify};

use crate::error::AppError;

/// Reply to `/options`: browser options plus the id of the state the master
/// kept for itself. The client hands that id back to `/verify`.
#[derive(serde::Serialize)]
pub struct ChallengeRes<T> {
    pub state_id: uuid::Uuid,
    #[serde(flatten)]
    pub options: T,
}

/// A failure must not hint at what was missing: details go to the log, the
/// caller always gets the same sentence.
fn reject(err: webauthn_rs::prelude::WebauthnError) -> AppError {
    tracing::warn!(error = %err, "WebAuthn verification failed");
    AppError::Unauthorized("That key did not match".into())
}
