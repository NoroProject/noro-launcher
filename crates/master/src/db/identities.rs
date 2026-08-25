//! Привязки аккаунта к внешним платформам.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::UserIdentity;
use sqlx::PgPool;
use uuid::Uuid;

/// Ряд таблицы. Отдельно от `schema::UserIdentity`: тот тип общий с лаунчером,
/// а sqlx туда не заезжает.
#[derive(sqlx::FromRow)]
struct IdentityRow {
    provider: String,
    provider_user_id: String,
    username: Option<String>,
    avatar_url: Option<String>,
    is_primary: bool,
    linked_at: DateTime<Utc>,
}

impl From<IdentityRow> for UserIdentity {
    fn from(r: IdentityRow) -> Self {
        UserIdentity {
            provider: r.provider,
            provider_user_id: r.provider_user_id,
            username: r.username,
            avatar_url: r.avatar_url,
            is_primary: r.is_primary,
            linked_at: r.linked_at,
        }
    }
}

/// Все привязки игрока. Первичная — первой: с неё начинается аккаунт, и в
/// списке она главная.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<UserIdentity>> {
    let rows = sqlx::query_as::<_, IdentityRow>(
        "SELECT provider, provider_user_id, username, avatar_url, is_primary, linked_at
           FROM user_identities WHERE user_id = $1
          ORDER BY is_primary DESC, linked_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(UserIdentity::from).collect())
}

/// Чей это аккаунт на стороне платформы, если он вообще известен.
pub async fn find_user(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<Uuid>> {
    let id = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM user_identities WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

/// Обновить ник и аватар: на той стороне их могли сменить с прошлого входа.
pub async fn touch(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
    username: &str,
    avatar: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE user_identities SET username = $3, avatar_url = $4
          WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider)
    .bind(provider_user_id)
    .bind(username)
    .bind(avatar)
    .execute(pool)
    .await?;
    Ok(())
}

/// Привязать платформу к существующему аккаунту.
///
/// `Ok(false)` — привязка уже занята кем-то другим. Это не ошибка уровня БД, а
/// обычный случай: игрок пробует привязать Discord, которым уже входил раньше.
pub async fn link(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    provider_user_id: &str,
    username: &str,
    avatar: Option<&str>,
    primary: bool,
) -> Result<bool> {
    let inserted = sqlx::query(
        "INSERT INTO user_identities (user_id, provider, provider_user_id, username, avatar_url, is_primary)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (provider, provider_user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(provider)
    .bind(provider_user_id)
    .bind(username)
    .bind(avatar)
    .bind(primary)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(inserted == 1)
}

/// Отвязать платформу.
///
/// Первичную не отдаём никогда: из неё выведен mc_uuid игрока, а вместе с ним
/// его инвентарь, прогресс и права на всех серверах. Отвязать её означало бы
/// оставить UUID, который больше ни на что не ссылается.
pub async fn unlink(pool: &PgPool, user_id: Uuid, provider: &str) -> Result<UnlinkResult> {
    let row: Option<(bool,)> = sqlx::query_as(
        "SELECT is_primary FROM user_identities WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await?;

    let Some((is_primary,)) = row else {
        return Ok(UnlinkResult::NotLinked);
    };
    if is_primary {
        return Ok(UnlinkResult::Primary);
    }

    sqlx::query("DELETE FROM user_identities WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await?;
    Ok(UnlinkResult::Removed)
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnlinkResult {
    Removed,
    /// Платформа регистрации — не отвязывается.
    Primary,
    NotLinked,
}
