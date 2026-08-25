//! OAuth2 Provider database models and queries.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AuthorizedAppInfo {
    pub id: Uuid,
    pub app_id: Uuid,
    pub client_id: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub scopes: String,
    pub authorized_at: DateTime<Utc>,
}

pub async fn list_user_authorized_apps(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AuthorizedAppInfo>> {
    let rows = sqlx::query_as::<_, AuthorizedAppInfo>(
        r#"
        SELECT
            ua.id,
            ua.app_id,
            app.client_id,
            app.name,
            app.icon_url,
            app.description,
            ua.scopes,
            ua.authorized_at
        FROM user_authorized_apps ua
        JOIN oauth_applications app ON app.id = ua.app_id
        WHERE ua.user_id = $1
        ORDER BY ua.authorized_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn authorize_app_for_user(
    db: &PgPool,
    user_id: Uuid,
    app_id: Uuid,
    scopes: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_authorized_apps (user_id, app_id, scopes)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, app_id) DO UPDATE SET scopes = EXCLUDED.scopes, authorized_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(app_id)
    .bind(scopes)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn revoke_user_authorized_app(db: &PgPool, user_id: Uuid, app_id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM user_authorized_apps WHERE user_id = $1 AND app_id = $2")
        .bind(user_id)
        .bind(app_id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn create_oauth_code(
    db: &PgPool,
    user_id: Uuid,
    app_id: Uuid,
    redirect_uri: &str,
    scopes: &str,
    code_challenge: Option<&str>,
) -> Result<Uuid> {
    let code = Uuid::new_v4();
    let expires = Utc::now() + chrono::Duration::minutes(10);
    sqlx::query(
        "INSERT INTO oauth_codes
             (code, user_id, app_id, redirect_uri, scopes, expires_at,
              code_challenge, code_challenge_method)
         VALUES ($1, $2, $3, $4, $5, $6, $7, CASE WHEN $7::text IS NULL THEN NULL ELSE 'S256' END)",
    )
    .bind(code)
    .bind(user_id)
    .bind(app_id)
    .bind(redirect_uri)
    .bind(scopes)
    .bind(expires)
    .bind(code_challenge)
    .execute(db)
    .await?;
    Ok(code)
}

/// Выданный код. Забирается ровно один раз: строка удаляется тем же запросом,
/// поэтому повторный обмен ничего не находит.
#[derive(Debug, FromRow)]
pub struct TakenCode {
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub redirect_uri: String,
    pub scopes: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

pub async fn take_oauth_code(db: &PgPool, code: Uuid) -> Result<Option<TakenCode>> {
    let row = sqlx::query_as::<_, TakenCode>(
        "DELETE FROM oauth_codes WHERE code = $1 AND used = FALSE AND expires_at > NOW()
         RETURNING user_id, app_id, redirect_uri, scopes, code_challenge, code_challenge_method",
    )
    .bind(code)
    .fetch_optional(db)
    .await?;
    Ok(row)
}
