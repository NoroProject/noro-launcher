//! Игровые сервера сборки: регистрация, секреты, heartbeat.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::GameServerEntry;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Сервер считается живым, если heartbeat приходил недавно. Агент шлёт его раз
/// в 30 секунд, так что три пропуска подряд — уже не сетевая икота.
pub const LIVE_WINDOW_SECS: i64 = 90;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct GameServerRow {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub mc_host: String,
    pub mc_port: i32,
    #[serde(skip)]
    pub token_hash: String,
    pub sort_order: i32,
    pub online: i32,
    pub max_online: i32,
    pub version: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// `proxy` — точка входа, `server` — бэкенд с агентом.
    pub kind: String,
}

impl GameServerRow {
    pub fn live(&self) -> bool {
        self.last_seen_at
            .is_some_and(|t| (Utc::now() - t).num_seconds() < LIVE_WINDOW_SECS)
    }

    pub fn is_proxy(&self) -> bool {
        self.kind == "proxy"
    }

    pub fn to_entry(&self) -> GameServerEntry {
        let live = self.live();
        GameServerEntry {
            id: self.id,
            name: self.name.clone(),
            mc_host: self.mc_host.clone(),
            mc_port: self.mc_port as u16,
            proxy: self.is_proxy(),
            // Онлайн мёртвого сервера — это последнее, что он успел сказать
            // перед падением, показывать его как текущий нельзя.
            online: if live { self.online.max(0) as u32 } else { 0 },
            max_online: self.max_online.max(0) as u32,
            live,
        }
    }
}

pub async fn list_game_servers(pool: &PgPool, server_id: Uuid) -> Result<Vec<GameServerRow>> {
    Ok(sqlx::query_as::<_, GameServerRow>(
        "SELECT * FROM game_servers WHERE server_id = $1 ORDER BY sort_order, name",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?)
}

pub async fn game_server_by_token_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<GameServerRow>> {
    Ok(
        sqlx::query_as::<_, GameServerRow>("SELECT * FROM game_servers WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn create_game_server(
    pool: &PgPool,
    server_id: Uuid,
    name: &str,
    mc_host: &str,
    mc_port: i32,
    token_hash: &str,
    kind: &str,
) -> Result<GameServerRow> {
    Ok(sqlx::query_as::<_, GameServerRow>(
        "INSERT INTO game_servers (server_id, name, mc_host, mc_port, token_hash, kind)
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
    )
    .bind(server_id)
    .bind(name)
    .bind(mc_host)
    .bind(mc_port)
    .bind(token_hash)
    .bind(kind)
    .fetch_one(pool)
    .await?)
}

pub async fn update_game_server(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    mc_host: &str,
    mc_port: i32,
    sort_order: i32,
    kind: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE game_servers SET name=$2, mc_host=$3, mc_port=$4, sort_order=$5, kind=$6
         WHERE id=$1",
    )
    .bind(id)
    .bind(name)
    .bind(mc_host)
    .bind(mc_port)
    .bind(sort_order)
    .bind(kind)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn rotate_game_server_token(pool: &PgPool, id: Uuid, token_hash: &str) -> Result<()> {
    sqlx::query("UPDATE game_servers SET token_hash=$2 WHERE id=$1")
        .bind(id)
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_game_server(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM game_servers WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Отметка жизни от агента.
pub async fn touch_game_server(
    pool: &PgPool,
    id: Uuid,
    online: i32,
    max_online: i32,
    version: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE game_servers
         SET online=$2, max_online=$3, version=COALESCE($4, version), last_seen_at=NOW()
         WHERE id=$1",
    )
    .bind(id)
    .bind(online)
    .bind(max_online)
    .bind(version)
    .execute(pool)
    .await?;
    Ok(())
}
