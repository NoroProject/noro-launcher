//! Какой лаунчер у какого игрока: версия, платформа, когда последний раз выходил на связь.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Строка для админки: игрок и его клиент.
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct LauncherClientRow {
    pub user_id: Uuid,
    pub mc_username: String,
    pub version: String,
    pub platform: String,
    pub last_seen_at: DateTime<Utc>,
}

/// Запомнить клиент при авторизации по WebSocket.
///
/// История не хранится — важно текущее состояние, поэтому одна строка на игрока.
pub async fn record_launcher_client(
    pool: &PgPool,
    user_id: Uuid,
    version: &str,
    platform: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO launcher_clients (user_id, version, platform, last_seen_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO UPDATE
         SET version = EXCLUDED.version,
             platform = EXCLUDED.platform,
             last_seen_at = NOW()",
    )
    .bind(user_id)
    .bind(version)
    .bind(platform)
    .execute(pool)
    .await?;
    Ok(())
}

/// Версия, которая сейчас раздаётся, безотносительно платформы.
///
/// `current_launcher_version` ищет по платформе, но клиент сообщает её как
/// `macos-aarch64`, а в `launcher_versions` лежит target-триплет. Релиз всё
/// равно выкатывается одной версией на все платформы, поэтому сравнивать проще
/// по ней одной.
pub async fn current_launcher_version_any(pool: &PgPool) -> Result<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT version FROM launcher_versions
         WHERE is_current = TRUE AND kind = 'core' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?)
}

/// Клиент одного игрока — для карточки в админке.
pub async fn launcher_client(pool: &PgPool, user_id: Uuid) -> Result<Option<LauncherClientRow>> {
    Ok(sqlx::query_as::<_, LauncherClientRow>(
        "SELECT c.user_id, u.mc_username, c.version, c.platform, c.last_seen_at
         FROM launcher_clients c
         JOIN users u ON u.id = c.user_id
         WHERE c.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

/// Страница известных клиентов, свежие сверху.
pub async fn list_launcher_clients(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<LauncherClientRow>, i64)> {
    let rows = sqlx::query_as::<_, LauncherClientRow>(
        "SELECT c.user_id, u.mc_username, c.version, c.platform, c.last_seen_at
         FROM launcher_clients c
         JOIN users u ON u.id = c.user_id
         ORDER BY c.last_seen_at DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM launcher_clients")
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

/// Сводка по всем клиентам: сколько на какой версии и платформе.
///
/// Считается запросом, а не перебором выданной страницы. Раньше отчёт собирался
/// из первых 500 строк списка: на большем числе установок «всего» показывало
/// ровно 500, а гистограмма версий строилась по случайной их части — то есть
/// цифры выглядели правдоподобно и были неверны.
pub async fn launcher_client_counts(pool: &PgPool, column: &str) -> Result<Vec<(String, i64)>> {
    // `column` не из запроса: вызывается двумя фиксированными строками ниже.
    let sql = format!(
        "SELECT CASE WHEN {column} = '' THEN 'unknown' ELSE {column} END AS key, count(*)
           FROM launcher_clients GROUP BY key"
    );
    Ok(sqlx::query_as::<_, (String, i64)>(&sql)
        .fetch_all(pool)
        .await?)
}

/// Сколько клиентов не на указанной версии.
pub async fn launcher_clients_outdated(pool: &PgPool, current: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT count(*) FROM launcher_clients WHERE version <> $1")
            .bind(current)
            .fetch_one(pool)
            .await?,
    )
}
