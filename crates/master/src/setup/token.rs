//! Одноразовый токен первичной настройки.
//!
//! Модель Jenkins `initialAdminPassword`: токен пишется в файл в `data_dir` и
//! печатается в лог при старте. Кто имеет доступ к машине — тот и настраивает;
//! другой аутентификации на пустом инстансе взяться неоткуда.

use crate::api::auth::admin_token;
use crate::error::AppError;
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use rand::Rng;
use sqlx::PgPool;
use std::path::Path;

const FILE_NAME: &str = "setup-token.txt";

/// Создать токен, если настройка ещё не завершена.
///
/// Хранится argon2-хеш; сам токен — только в файле и в логе, откуда его и
/// забирает оператор.
///
/// Токен обязан пережить рестарт: визард двухфазный, и между фазами мастер
/// перезапускают. Пересоздание при каждом старте отбирало бы у оператора доступ
/// к его же визарду ровно там, где визард сам просит перезапуститься.
pub async fn ensure_token(pool: &PgPool, data_dir: &Path) -> Result<Option<String>> {
    let state = crate::db::instance_state(pool).await?;
    let path = data_dir.join(FILE_NAME);

    if state.setup_completed {
        // Файл мог остаться от прошлой установки — он больше ничего не открывает,
        // но лежать ему незачем.
        let _ = tokio::fs::remove_file(&path).await;
        return Ok(None);
    }

    // Хеш есть и файл на месте — токен прежний. Печатать его снова незачем,
    // оператор уже держит его в браузере.
    if state.setup_token_hash.is_some() && tokio::fs::try_exists(&path).await.unwrap_or(false) {
        tracing::warn!(
            "инстанс не настроен. Токен установки — в {}",
            path.display()
        );
        return Ok(None);
    }

    let secret: String = {
        let bytes: [u8; 24] = rand::thread_rng().gen();
        hex::encode(bytes)
    };
    let hash = admin_token::hash(&secret)?;
    crate::db::set_setup_token_hash(pool, Some(&hash)).await?;

    tokio::fs::write(&path, &secret)
        .await
        .with_context(|| format!("запись {}", path.display()))?;

    Ok(Some(secret))
}

/// Удалить файл токена: настройка завершена, открывать ему больше нечего.
pub async fn burn_token_file(data_dir: &Path) {
    let _ = tokio::fs::remove_file(data_dir.join(FILE_NAME)).await;
}

/// Проверить предъявленный токен.
pub async fn verify_token(pool: &PgPool, presented: &str) -> Result<bool> {
    let state = crate::db::instance_state(pool).await?;
    if state.setup_completed {
        return Ok(false);
    }
    Ok(match state.setup_token_hash {
        Some(hash) => admin_token::verify(presented, &hash),
        None => false,
    })
}

/// Доступ к `/api/setup/*`. Существует только до завершения настройки.
pub struct SetupAuth;

impl FromRequestParts<AppState> for SetupAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .ok_or_else(|| AppError::Unauthorized("missing setup token".into()))?;

        if verify_token(&state.db, token)
            .await
            .map_err(AppError::Other)?
        {
            Ok(SetupAuth)
        } else {
            Err(AppError::Unauthorized(
                "the setup token did not match".into(),
            ))
        }
    }
}
