//! Авторизация агента игрового сервера по собственному секрету.
//!
//! Админ-токен для этого не годится: он живёт в конфиге на игровой машине, а
//! к ней есть доступ у тех, кто сервером управляет. Секрет привязан к одному
//! game_server — по нему же мастер понимает, кто именно спрашивает, поэтому
//! агент не может подставить чужой server_id.

use crate::db::game_servers::GameServerRow;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sha2::{Digest, Sha256};

/// Секрет агента: префикс отличает его от админского в логах и конфигах.
pub fn generate_agent_secret() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    format!("noroagent_{}", hex::encode(bytes))
}

pub fn hash_agent_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

/// Агент, опознанный по секрету. Содержит сервер, за который он отвечает.
pub struct AgentAuth {
    pub game_server: GameServerRow,
}

impl FromRequestParts<AppState> for AgentAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let secret = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .ok_or_else(|| AppError::Unauthorized("нет Bearer-токена".into()))?;

        let hash = hash_agent_secret(secret);
        let game_server = crate::db::game_server_by_token_hash(&state.db, &hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("неизвестный секрет агента".into()))?;
        Ok(AgentAuth { game_server })
    }
}
