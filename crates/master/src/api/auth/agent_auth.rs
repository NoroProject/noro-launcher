//! Game-server agent auth, by a secret of its own.
//!
//! An admin token would be wrong here: it sits in a config file on the game
//! machine, which whoever runs the server can read. This secret is bound to a
//! single game_server, and the master derives the server identity from it, so
//! an agent can't pass someone else's server_id.

use crate::db::game_servers::GameServerRow;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sha2::{Digest, Sha256};

/// The prefix is what tells an agent secret from an admin one in logs and
/// config files.
pub fn generate_agent_secret() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    format!("noroagent_{}", hex::encode(bytes))
}

pub fn hash_agent_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

/// An agent identified by its secret, together with the server it speaks for.
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
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

        let hash = hash_agent_secret(secret);
        let game_server = crate::db::game_server_by_token_hash(&state.db, &hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("unknown agent secret".into()))?;
        Ok(AgentAuth { game_server })
    }
}
