//! Extractors авторизации: AuthUser (Bearer-токен пользователя) и AdminAuth
//! (пользователь с admin-правом ИЛИ admin-токен из CLI/CI).

use crate::error::AppError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use schema::UserProfile;
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn bearer(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.trim().to_string())
}

/// Аутентифицированный пользователь.
pub struct AuthUser {
    pub user_id: Uuid,
    pub profile: UserProfile,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer(parts).ok_or_else(|| AppError::Unauthorized("нет Bearer-токена".into()))?;
        let token_uuid =
            Uuid::parse_str(&token).map_err(|_| AppError::Unauthorized("неверный токен".into()))?;
        let row = crate::db::user_by_access_token(&state.db, token_uuid)
            .await?
            .ok_or_else(|| AppError::Unauthorized("сессия не найдена или истекла".into()))?;
        let user_id = row.id;
        let profile = crate::db::profile_from_row(&state.db, row).await?;
        if profile.banned {
            return Err(AppError::Forbidden("аккаунт заблокирован".into()));
        }
        Ok(AuthUser { user_id, profile })
    }
}

/// Доступ к админ-API. Любое из:
///  - пользователь с правом, покрывающим запрошенное (например `noro.admin.users`);
///  - admin-токен с таким правом.
pub struct AdminAuth {
    /// Кто именно действует — для журнала аудита.
    pub actor: crate::audit::Actor,
    /// Эффективные права субъекта.
    pub permissions: Vec<String>,
}

impl AdminAuth {
    /// id пользователя, если авторизация по пользовательскому токену.
    pub fn user_id(&self) -> Option<Uuid> {
        self.actor.id()
    }

    pub fn require(&self, perm: &str) -> Result<(), AppError> {
        let has = self
            .permissions
            .iter()
            .any(|p| schema::permission_matches(p, perm));
        if has {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("нужно право {perm}")))
        }
    }
}

impl FromRequestParts<AppState> for AdminAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer(parts).ok_or_else(|| AppError::Unauthorized("нет Bearer-токена".into()))?;

        // Сначала пробуем как admin-токен (hash совпадает).
        let hash = hex::encode(Sha256::digest(token.as_bytes()));
        if let Some(t) = crate::db::admin_token_by_hash(&state.db, &hash).await? {
            return Ok(AdminAuth {
                actor: crate::audit::Actor::Token { name: t.name },
                permissions: t.permissions,
            });
        }

        // Иначе — пользовательский токен.
        if let Ok(token_uuid) = Uuid::parse_str(&token) {
            if let Some(row) = crate::db::user_by_access_token(&state.db, token_uuid).await? {
                let user_id = row.id;
                let profile = crate::db::profile_from_row(&state.db, row).await?;
                let permissions: Vec<String> =
                    profile.all_permissions().map(String::from).collect();
                return Ok(AdminAuth {
                    actor: crate::audit::Actor::User {
                        id: user_id,
                        username: profile.username.clone(),
                    },
                    permissions,
                });
            }
        }

        Err(AppError::Unauthorized("неверный токен".into()))
    }
}

/// Хешировать секрет admin-токена для хранения/поиска.
pub fn hash_admin_token(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}
