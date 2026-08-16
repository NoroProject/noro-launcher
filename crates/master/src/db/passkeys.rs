//! Хранение passkey-ключей и состояния WebAuthn между /options и /verify.

use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Строка ключа. `credential` — учётные данные webauthn-rs целиком: открытый
/// ключ, счётчик, флаги. Наружу (в кабинет) она не отдаётся.
#[derive(Debug, sqlx::FromRow)]
pub struct PasskeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub credential_id: String,
    pub credential: Value,
    pub created_at: chrono::DateTime<Utc>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
}

/// То, что видно в кабинете: без ключевого материала.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasskeyInfo {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
}

/// Состояние регистрации/входа живёт 5 минут — столько же, сколько браузерный
/// таймаут диалога.
pub async fn save_webauthn_state(
    pool: &PgPool,
    kind: &str,
    user_id: Option<Uuid>,
    state: &Value,
) -> Result<Uuid> {
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO webauthn_states (user_id, kind, state, expires_at)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(user_id)
    .bind(kind)
    .bind(state)
    .bind(Utc::now() + Duration::minutes(5))
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Забрать состояние ровно один раз: повторное использование challenge — это
/// replay, поэтому запись удаляется независимо от исхода проверки.
pub async fn take_webauthn_state(
    pool: &PgPool,
    id: Uuid,
    kind: &str,
) -> Result<Option<(Option<Uuid>, Value)>> {
    let row = sqlx::query_as::<_, (Option<Uuid>, Value)>(
        "DELETE FROM webauthn_states
         WHERE id = $1 AND kind = $2 AND expires_at > NOW()
         RETURNING user_id, state",
    )
    .bind(id)
    .bind(kind)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_passkey(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    credential_id: &str,
    credential: &Value,
) -> Result<PasskeyInfo> {
    let row = sqlx::query_as::<_, PasskeyInfo>(
        "INSERT INTO passkeys (user_id, name, credential_id, credential)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, created_at, last_used_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(credential_id)
    .bind(credential)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_passkeys_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PasskeyInfo>> {
    let rows = sqlx::query_as::<_, PasskeyInfo>(
        "SELECT id, name, created_at, last_used_at FROM passkeys
         WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Все ключи пользователя — для discoverable-входа и для проверки счётчика.
pub async fn passkeys_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PasskeyRow>> {
    let rows = sqlx::query_as::<_, PasskeyRow>("SELECT * FROM passkeys WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Записать обновлённые учётные данные: счётчик растёт с каждым входом, и
/// откат счётчика — признак клонированного ключа.
pub async fn update_passkey_credential(pool: &PgPool, id: Uuid, credential: &Value) -> Result<()> {
    sqlx::query("UPDATE passkeys SET credential = $2, last_used_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(credential)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_passkey(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM passkeys WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
