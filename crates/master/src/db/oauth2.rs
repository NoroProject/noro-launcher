//! OAuth2 Provider database models and queries.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct OAuthApp {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret_hash: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub redirect_uris: String,
    pub is_trusted: bool,
    pub created_at: DateTime<Utc>,
}

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

pub async fn get_oauth_app_by_client_id(
    db: &PgPool,
    client_id: &str,
) -> Result<Option<OAuthApp>> {
    let row = sqlx::query_as::<_, OAuthApp>(
        "SELECT id, client_id, client_secret_hash, name, icon_url, description, redirect_uris, is_trusted, created_at FROM oauth_applications WHERE client_id = $1"
    )
    .bind(client_id)
    .fetch_optional(db)
    .await?;
    Ok(row)
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
        "#
    )
    .bind(user_id)
    .bind(app_id)
    .bind(scopes)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn revoke_user_authorized_app(
    db: &PgPool,
    user_id: Uuid,
    app_id: Uuid,
) -> Result<bool> {
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
) -> Result<Uuid> {
    let code = Uuid::new_v4();
    let expires = Utc::now() + chrono::Duration::minutes(10);
    sqlx::query(
        "INSERT INTO oauth_codes (code, user_id, app_id, redirect_uri, scopes, expires_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(code)
    .bind(user_id)
    .bind(app_id)
    .bind(redirect_uri)
    .bind(scopes)
    .bind(expires)
    .execute(db)
    .await?;
    Ok(code)
}

pub async fn take_oauth_code(
    db: &PgPool,
    code: Uuid,
) -> Result<Option<(Uuid, Uuid, String, String)>> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        "DELETE FROM oauth_codes WHERE code = $1 AND used = FALSE AND expires_at > NOW() RETURNING user_id, app_id, redirect_uri, scopes"
    )
    .bind(code)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

pub async fn ensure_default_launcher_app(db: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS oauth_applications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id VARCHAR(64) NOT NULL UNIQUE,
            client_secret_hash VARCHAR(255) NOT NULL,
            name VARCHAR(100) NOT NULL,
            icon_url TEXT,
            description TEXT,
            redirect_uris TEXT NOT NULL,
            is_trusted BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_authorized_apps (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            app_id UUID NOT NULL REFERENCES oauth_applications(id) ON DELETE CASCADE,
            scopes TEXT NOT NULL DEFAULT 'profile',
            authorized_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, app_id)
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS oauth_codes (
            code UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            app_id UUID NOT NULL REFERENCES oauth_applications(id) ON DELETE CASCADE,
            redirect_uri TEXT NOT NULL,
            scopes TEXT NOT NULL DEFAULT 'profile',
            expires_at TIMESTAMPTZ NOT NULL,
            used BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO oauth_applications (client_id, client_secret_hash, name, description, redirect_uris, is_trusted)
        VALUES ('noro_launcher', 'public', 'Noro Launcher', 'Официальный лаунчер Noro Network', '["http://127.0.0.1"]', TRUE)
        ON CONFLICT (client_id) DO UPDATE SET is_trusted = TRUE
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}
