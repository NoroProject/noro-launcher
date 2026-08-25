//! Ключи провайдеров: откуда берутся и как понять, что провайдер готов.
//!
//! Приоритет тот же, что у остальных настроек, — env > БД. Боевой инстанс,
//! у которого `DISCORD_CLIENT_SECRET` задан в compose, продолжает работать на
//! нём, даже если в БД пусто.

use super::provider::Provider;
use crate::db::auth_methods::AuthMethodRow;
use crate::error::{AppError, AppResult};
use sqlx::PgPool;

pub struct Creds {
    pub client_id: String,
    pub client_secret: String,
}

impl Creds {
    pub fn is_set(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

fn from_env(p: Provider, suffix: &str) -> Option<String> {
    crate::config::env_opt(&format!("{}_{suffix}", p.slug().to_uppercase()))
}

/// Ключи из env, а чего там нет — из строки таблицы.
pub fn resolve(p: Provider, row: Option<&AuthMethodRow>) -> Creds {
    Creds {
        client_id: from_env(p, "CLIENT_ID")
            .unwrap_or_else(|| row.map(|r| r.client_id.clone()).unwrap_or_default()),
        client_secret: from_env(p, "CLIENT_SECRET")
            .unwrap_or_else(|| row.map(|r| r.client_secret.clone()).unwrap_or_default()),
    }
}

/// Ключи провайдера или внятный отказ. Ненастроенный провайдер — не 500:
/// оператор просто ещё не завёл приложение на той стороне.
pub async fn creds(db: &PgPool, p: Provider) -> AppResult<Creds> {
    let row = crate::db::auth_methods::get(db, p.slug())
        .await
        .map_err(AppError::Other)?;

    // Выключенный провайдер не должен пускать внутрь даже по прямой ссылке:
    // кнопку с сайта убрали, а адрес остался у кого-то в закладках.
    if row.as_ref().is_some_and(|r| !r.enabled) {
        return Err(AppError::BadRequest(format!(
            "signing in with {} is turned off",
            p.display_name()
        )));
    }

    let creds = resolve(p, row.as_ref());
    if !creds.is_set() {
        return Err(AppError::BadRequest(format!(
            "{} sign-in is not configured on this instance",
            p.display_name()
        )));
    }
    Ok(creds)
}

/// Провайдеры, которыми реально можно войти прямо сейчас.
pub async fn enabled(db: &PgPool) -> AppResult<Vec<Provider>> {
    let rows = crate::db::auth_methods::all(db)
        .await
        .map_err(AppError::Other)?;
    Ok(Provider::ALL
        .into_iter()
        .filter(|p| {
            let row = rows.iter().find(|r| r.method == p.slug());
            row.is_none_or(|r| r.enabled) && resolve(*p, row).is_set()
        })
        .collect())
}
