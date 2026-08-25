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
            bearer(parts).ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
        let token_uuid =
            Uuid::parse_str(&token).map_err(|_| AppError::Unauthorized("invalid token".into()))?;
        let (row, scope) = crate::db::session_by_access_token(&state.db, token_uuid)
            .await?
            .ok_or_else(|| AppError::Unauthorized("session not found or expired".into()))?;
        // Токен, выданный стороннему приложению, сюда не проходит. Раньше
        // проходил: scope не проверялся нигде, и любое приложение получало
        // аккаунт целиком — вместе с админкой, если игрок был оператором.
        if !schema::is_internal(&scope) {
            return Err(AppError::Forbidden(
                "this token was issued to an application: use /api/oauth/*".into(),
            ));
        }
        let user_id = row.id;
        let profile = crate::db::profile_from_row(&state.db, row).await?;
        if profile.banned {
            return Err(AppError::Forbidden("account is banned".into()));
        }
        Ok(AuthUser { user_id, profile })
    }
}

/// Аккаунт, открытый стороннему приложению ровно на выданные scope'ы.
///
/// Отдельный extractor, а не флаг в [`AuthUser`]: ручка обязана назвать scope,
/// который ей нужен, и забыть проверку нельзя — без вызова `require` из
/// экстрактора не достать ни профиль, ни идентификатор.
pub struct AppAuth {
    user_id: Uuid,
    profile: UserProfile,
    scopes: Vec<String>,
}

impl AppAuth {
    /// Профиль и идентификатор — только вместе с проверкой scope'а.
    pub fn require(&self, scope: &'static str) -> Result<(Uuid, &UserProfile), AppError> {
        if self.scopes.iter().any(|s| s == scope) {
            return Ok((self.user_id, &self.profile));
        }
        Err(AppError::Forbidden(format!(
            "the player did not grant the {scope} scope to this application"
        )))
    }

    /// Что игрок разрешил приложению — для ручек, отдающих разный объём данных.
    pub fn has(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

impl FromRequestParts<AppState> for AppAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer(parts).ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
        let token_uuid =
            Uuid::parse_str(&token).map_err(|_| AppError::Unauthorized("invalid token".into()))?;
        let (row, scope) = crate::db::session_by_access_token(&state.db, token_uuid)
            .await?
            .ok_or_else(|| AppError::Unauthorized("session not found or expired".into()))?;
        let user_id = row.id;
        let profile = crate::db::profile_from_row(&state.db, row).await?;
        if profile.banned {
            return Err(AppError::Forbidden("account is banned".into()));
        }
        // Наш собственный вход умеет всё, что умеет приложение: лаунчеру и
        // сайту незачёт отдельный токен ради тех же данных.
        let scopes = if schema::is_internal(&scope) {
            schema::ALL_SCOPES
                .iter()
                .map(|s| s.name.to_string())
                .collect()
        } else {
            schema::parse_scopes(&scope)
        };
        Ok(AppAuth {
            user_id,
            profile,
            scopes,
        })
    }
}

/// Необязательный пользователь (если токен передан и валиден — AuthUser, иначе None).
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl std::ops::Deref for OptionalAuthUser {
    type Target = Option<AuthUser>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(
            AuthUser::from_request_parts(parts, state).await.ok(),
        ))
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
    /// Токен сессии, если вход по пользовательскому токену. Нужен, чтобы
    /// отличить сессию, открытую под impersonation, от обычной.
    pub session_token: Option<Uuid>,
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
            Err(AppError::Forbidden(format!(
                "the {perm} permission is required"
            )))
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
            bearer(parts).ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

        // Сначала пробуем как admin-токен.
        if let Some(t) = admin_token_auth(state, &token).await? {
            return Ok(t);
        }

        // Иначе — пользовательский токен. Токен приложения админкой не считается
        // никогда: игрок разрешал ему свои скины, а не выдачу банов от своего
        // имени — и права оператора в этот момент были бы правами приложения.
        if let Ok(token_uuid) = Uuid::parse_str(&token) {
            if let Some((row, scope)) =
                crate::db::session_by_access_token(&state.db, token_uuid).await?
            {
                if !schema::is_internal(&scope) {
                    return Err(AppError::Forbidden(
                        "this token was issued to an application".into(),
                    ));
                }
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
                    session_token: Some(token_uuid),
                });
            }
        }

        Err(AppError::Unauthorized("invalid token".into()))
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
        session_token: None,
    }))
}
