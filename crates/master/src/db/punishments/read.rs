//! Выборки по наказаниям. Их спрашивают с игрового сервера на каждом входе,
//! поэтому каждая — один запрос без дозагрузок.

use super::PunishmentRow;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Вся история, включая снятое: снятый бан — это тоже факт, который нужен при
/// разборе следующего случая.
pub async fn list_punishments(pool: &PgPool, user_id: Uuid) -> Result<Vec<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn punishment_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PunishmentRow>> {
    Ok(
        sqlx::query_as::<_, PunishmentRow>("SELECT * FROM punishments WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Предупреждения, которых игрок ещё не видел. Агент показывает их при входе:
/// варн, о котором наказанный не узнал, не значит ничего.
pub async fn pending_warns(pool: &PgPool, user_id: Uuid) -> Result<Vec<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments
         WHERE user_id = $1 AND kind = 'warn' AND revoked_at IS NULL
           AND acknowledged_at IS NULL
         ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Действующие наказания.
pub async fn active_punishments(pool: &PgPool, user_id: Uuid) -> Result<Vec<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments
         WHERE user_id = $1 AND revoked_at IS NULL
           AND (expires_at IS NULL OR expires_at > NOW())
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}
/// Проверить, замучен ли игрок на данном сервере (или глобально).
pub async fn active_mute_for_user(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Option<Uuid>,
) -> Result<Option<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments
         WHERE user_id = $1 AND kind = 'mute' AND revoked_at IS NULL
           AND (expires_at IS NULL OR expires_at > NOW())
           AND (server_id IS NULL OR server_id = $2)
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await?)
}

/// Действующий бан: сети либо этой сборки.
///
/// Симметрично [`active_mute_for_user`], и нужен ровно затем же: агент знает
/// `banned: bool` и не может показать на экране отказа ни причины, ни срока —
/// а забаненный вчера видит только сухую строку.
///
/// Сетевой бан старше серверного: если игрок забанен и там, и там, отпустит
/// его первым серверный, и показывать надо тот, что держит дольше.
pub async fn active_ban_for_user(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Option<Uuid>,
) -> Result<Option<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments
         WHERE user_id = $1 AND kind IN ('ban', 'server_ban') AND revoked_at IS NULL
           AND (expires_at IS NULL OR expires_at > NOW())
           AND (server_id IS NULL OR server_id = $2)
         ORDER BY kind = 'ban' DESC, expires_at IS NULL DESC, expires_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await?)
}
