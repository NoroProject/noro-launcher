//! Extractors авторизации: AuthUser (Bearer-токен пользователя) и AdminAuth
//! (пользователь с admin-правом ИЛИ admin-токен из CLI/CI).

use super::admin_token;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use schema::UserProfile;
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

        // Сначала пробуем как admin-токен.
        if let Some(t) = admin_token_auth(state, &token).await? {
            return Ok(t);
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

/// Проверить предъявленный admin-токен.
///
/// `None` означает «это не admin-токен» — вызывающий пробует его как
/// пользовательский. Отказ в правах здесь не различается с «не найден»
/// намеренно: подсказывать, что токен существует, но не подошёл, незачем.
async fn admin_token_auth(state: &AppState, token: &str) -> Result<Option<AdminAuth>, AppError> {
    let lookup = admin_token::lookup(token);
    let Some(row) = crate::db::admin_token_by_lookup(&state.db, &lookup).await? else {
        return Ok(None);
    };

    match &row.token_hash {
        Some(phc) => {
            if !admin_token::verify(token, phc) {
                // Селектор сошёлся, а argon2 — нет. SHA-256-коллизии не бывает,
                // значит запись испорчена: об этом надо знать.
                tracing::error!(token = %row.name, "admin-токен: селектор сошёлся, хеш нет");
                return Ok(None);
            }
        }
        // Токен из старой схемы: доказательством был сам SHA-256, и он совпал.
        // Секрет у нас на руках ровно сейчас — досчитываем argon2 и больше к
        // старой схеме не возвращаемся.
        None => match admin_token::hash(token) {
            Ok(phc) => {
                crate::db::upgrade_admin_token_hash(&state.db, row.id, &phc).await?;
                tracing::info!(token = %row.name, "admin-токен переведён на argon2");
            }
            Err(e) => tracing::error!(error = %e, token = %row.name, "не пересчитать хеш токена"),
        },
    }

    crate::db::touch_admin_token(&state.db, row.id).await?;
    Ok(Some(AdminAuth {
        actor: crate::audit::Actor::Token { name: row.name },
        permissions: row.permissions,
    }))
}
