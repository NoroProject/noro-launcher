//! Способы входа: включён ли и с какими ключами приложения.

use anyhow::Result;
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthMethodRow {
    pub method: String,
    pub client_id: String,
    pub client_secret: String,
    pub enabled: bool,
}

const COLUMNS: &str = "method, client_id, client_secret, enabled";

pub async fn all(pool: &PgPool) -> Result<Vec<AuthMethodRow>> {
    let rows = sqlx::query_as::<_, AuthMethodRow>(&format!(
        "SELECT {COLUMNS} FROM auth_methods ORDER BY method"
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get(pool: &PgPool, method: &str) -> Result<Option<AuthMethodRow>> {
    let row = sqlx::query_as::<_, AuthMethodRow>(&format!(
        "SELECT {COLUMNS} FROM auth_methods WHERE method = $1"
    ))
    .bind(method)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Способ, о котором в таблице ничего не сказано, считается включённым: строки
/// появляются миграцией, и отсутствие записи — это старый инстанс, а не запрет.
pub async fn is_enabled(pool: &PgPool, method: &str) -> Result<bool> {
    Ok(get(pool, method).await?.is_none_or(|r| r.enabled))
}

/// Сохранить настройку. `client_secret = None` — «не трогать»: админка не
/// показывает текущий секрет, и пустое поле формы означает «оставить как
/// было», а не «стереть».
pub async fn save(
    pool: &PgPool,
    method: &str,
    client_id: &str,
    client_secret: Option<&str>,
    enabled: bool,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO auth_methods (method, client_id, client_secret, enabled, updated_at)
         VALUES ($1, $2, COALESCE($3, ''), $4, NOW())
         ON CONFLICT (method) DO UPDATE SET
             client_id = EXCLUDED.client_id,
             client_secret = COALESCE($3, auth_methods.client_secret),
             enabled = EXCLUDED.enabled,
             updated_at = NOW()",
    )
    .bind(method)
    .bind(client_id)
    .bind(client_secret)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}
