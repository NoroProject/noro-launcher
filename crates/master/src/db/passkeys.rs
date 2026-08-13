use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasskeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub credential_id: String,
    pub public_key: String,
    pub counter: i64,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn save_challenge(pool: &PgPool, challenge: &str, user_id: Option<Uuid>) -> Result<()> {
    let expires = Utc::now() + Duration::minutes(5);
    sqlx::query(
        "INSERT INTO passkey_challenges (challenge, user_id, expires_at)
         VALUES ($1, $2, $3)
         ON CONFLICT (challenge) DO UPDATE SET expires_at = $3",
    )
    .bind(challenge)
    .bind(user_id)
    .bind(expires)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn verify_and_consume_challenge(
    pool: &PgPool,
    challenge: &str,
) -> Result<Option<Option<Uuid>>> {
    let row = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT user_id FROM passkey_challenges WHERE challenge = $1 AND expires_at > NOW()",
    )
    .bind(challenge)
    .fetch_optional(pool)
    .await?;

    if let Some(user_id) = row {
        sqlx::query("DELETE FROM passkey_challenges WHERE challenge = $1")
            .bind(challenge)
            .execute(pool)
            .await?;
        Ok(Some(user_id))
    } else {
        Ok(None)
    }
}

pub async fn create_passkey(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    credential_id: &str,
    public_key: &str,
) -> Result<PasskeyRow> {
    let row = sqlx::query_as::<_, PasskeyRow>(
        "INSERT INTO passkeys (user_id, name, credential_id, public_key)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(user_id)
    .bind(name)
    .bind(credential_id)
    .bind(public_key)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_passkeys_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PasskeyRow>> {
    let rows = sqlx::query_as::<_, PasskeyRow>(
        "SELECT * FROM passkeys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_passkey_by_credential_id(
    pool: &PgPool,
    credential_id: &str,
) -> Result<Option<PasskeyRow>> {
    let row = sqlx::query_as::<_, PasskeyRow>("SELECT * FROM passkeys WHERE credential_id = $1")
        .bind(credential_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn delete_passkey(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM passkeys WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
