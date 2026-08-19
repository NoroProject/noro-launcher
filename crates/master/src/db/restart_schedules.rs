//! Расписание рестартов серверов (`restart_schedules`).

use anyhow::Result;
use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RestartScheduleRow {
    pub id: Uuid,
    pub game_server_id: Uuid,
    pub cron_expr: Option<String>,
    pub at_times: Option<Vec<String>>,
    pub interval_minutes: Option<i32>,
    pub notice_minutes: i32,
    pub online_policy: String,
    pub max_defer_minutes: i32,
    pub active: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_restart_schedules(
    pool: &PgPool,
    game_server_id: Uuid,
) -> Result<Vec<RestartScheduleRow>> {
    Ok(sqlx::query_as::<_, RestartScheduleRow>(
        "SELECT id, game_server_id, cron_expr, at_times, interval_minutes, notice_minutes, online_policy, max_defer_minutes, active, last_run_at, next_run_at, created_at FROM restart_schedules WHERE game_server_id = $1 ORDER BY created_at DESC",
    )
    .bind(game_server_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create_restart_schedule(
    pool: &PgPool,
    game_server_id: Uuid,
    cron_expr: Option<&str>,
    at_times: Option<Vec<String>>,
    interval_minutes: Option<i32>,
    notice_minutes: i32,
    online_policy: &str,
    max_defer_minutes: i32,
) -> Result<RestartScheduleRow> {
    let now = Utc::now();
    let next_run = compute_next_run(cron_expr, at_times.as_deref(), interval_minutes, now)
        .ok_or_else(|| {
            anyhow::anyhow!("cannot compute next run from provided schedule parameters")
        })?;

    Ok(sqlx::query_as::<_, RestartScheduleRow>(
        "INSERT INTO restart_schedules
            (game_server_id, cron_expr, at_times, interval_minutes, notice_minutes, online_policy, max_defer_minutes, next_run_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, game_server_id, cron_expr, at_times, interval_minutes, notice_minutes, online_policy, max_defer_minutes, active, last_run_at, next_run_at, created_at",
    )
    .bind(game_server_id)
    .bind(cron_expr)
    .bind(at_times)
    .bind(interval_minutes)
    .bind(notice_minutes)
    .bind(online_policy)
    .bind(max_defer_minutes)
    .bind(next_run)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_restart_schedule(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM restart_schedules WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub fn compute_next_run(
    cron_expr: Option<&str>,
    at_times: Option<&[String]>,
    interval_minutes: Option<i32>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if let Some(mins) = interval_minutes {
        if mins > 0 {
            return Some(now + chrono::Duration::minutes(mins as i64));
        }
    }
    if let Some(times) = at_times {
        if !times.is_empty() {
            if let Some(next) = parse_at_times(times, now) {
                return Some(next);
            }
        }
    }
    if let Some(expr) = cron_expr {
        if !expr.trim().is_empty() {
            if let Some(next) = parse_cron(expr.trim(), now) {
                return Some(next);
            }
        }
    }
    None
}

fn parse_at_times(times: &[String], now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let current_time = now.time();
    let mut candidates = Vec::new();
    for t_str in times {
        if let Ok(t) = NaiveTime::parse_from_str(t_str.trim(), "%H:%M") {
            if t > current_time {
                if let Some(dt) = now
                    .date_naive()
                    .and_time(t)
                    .and_local_timezone(Utc)
                    .single()
                {
                    candidates.push(dt);
                }
            } else if let Some(tomorrow) = now.date_naive().succ_opt() {
                if let Some(dt) = tomorrow.and_time(t).and_local_timezone(Utc).single() {
                    candidates.push(dt);
                }
            }
        }
    }
    candidates.into_iter().min()
}

fn parse_cron(cron_expr: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    for minute_offset in 1..=(7 * 24 * 60) {
        let candidate = now + chrono::Duration::minutes(minute_offset);
        let min: u32 = candidate.format("%M").to_string().parse().unwrap_or(0);
        let hour: u32 = candidate.format("%H").to_string().parse().unwrap_or(0);
        let dom: u32 = candidate.format("%d").to_string().parse().unwrap_or(0);
        let month: u32 = candidate.format("%m").to_string().parse().unwrap_or(0);
        let dow: u32 = candidate
            .format("%u")
            .to_string()
            .parse::<u32>()
            .unwrap_or(0)
            % 7;

        if field_matches(parts[0], min)
            && field_matches(parts[1], hour)
            && field_matches(parts[2], dom)
            && field_matches(parts[3], month)
            && field_matches(parts[4], dow)
        {
            return Some(candidate);
        }
    }
    None
}

fn field_matches(expr: &str, val: u32) -> bool {
    if expr == "*" {
        return true;
    }
    if let Ok(num) = expr.parse::<u32>() {
        return num == val;
    }
    if expr.contains('/') {
        let parts: Vec<&str> = expr.split('/').collect();
        if parts.len() == 2 {
            if let Ok(step) = parts[1].parse::<u32>() {
                return step > 0 && val % step == 0;
            }
        }
    }
    if expr.contains(',') {
        return expr.split(',').any(|p| field_matches(p, val));
    }
    false
}

pub fn spawn_restart_scheduler(state: crate::state::AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            if let Err(e) = check_and_trigger_restarts(&state).await {
                tracing::warn!(error = %e, "restart scheduler tick error");
            }
        }
    });
}

async fn check_and_trigger_restarts(state: &crate::state::AppState) -> Result<()> {
    let now = Utc::now();
    let rows = sqlx::query_as::<_, RestartScheduleRow>(
        "SELECT id, game_server_id, cron_expr, at_times, interval_minutes, notice_minutes, online_policy, max_defer_minutes, active, last_run_at, next_run_at, created_at
         FROM restart_schedules
         WHERE active = true AND next_run_at IS NOT NULL AND next_run_at <= $1",
    )
    .bind(now)
    .fetch_all(&state.db)
    .await?;

    for s in rows {
        let next_scheduled = s.next_run_at.unwrap_or(now);

        // Обработка online_policy (defer)
        if s.online_policy == "defer" {
            let roster = state.roster.of(s.game_server_id);
            let online_count = roster.visible.len() + roster.vanished.len();
            if online_count > 0 {
                let deferred_mins = (now - next_scheduled).num_minutes();
                if deferred_mins < s.max_defer_minutes as i64 {
                    let new_next = now + chrono::Duration::minutes(5);
                    tracing::info!(
                        server_id = %s.game_server_id,
                        online = online_count,
                        deferred_mins,
                        max_defer = s.max_defer_minutes,
                        "откладываем рестарт сервера на 5 минут из-за игроков онлайн"
                    );
                    sqlx::query("UPDATE restart_schedules SET next_run_at = $1 WHERE id = $2")
                        .bind(new_next)
                        .bind(s.id)
                        .execute(&state.db)
                        .await?;
                    continue;
                }
            }
        }

        // Высчитываем следующий запуск
        let next_run = compute_next_run(
            s.cron_expr.as_deref(),
            s.at_times.as_deref(),
            s.interval_minutes,
            now,
        );
        sqlx::query(
            "UPDATE restart_schedules SET last_run_at = $1, next_run_at = $2 WHERE id = $3",
        )
        .bind(now)
        .bind(next_run)
        .bind(s.id)
        .execute(&state.db)
        .await?;

        // Шлём предупреждение игрокам
        let notice_sec = (s.notice_minutes.max(0) * 60) as u32;
        crate::agent_link::notify::restart_notice(
            state,
            None,
            Some(s.game_server_id),
            notice_sec,
            Some("Scheduled restart".to_string()),
        );

        // Запускаем отложенный рестарт через враппер после отсчёта notice_minutes
        let state_clone = state.clone();
        let server_id = s.game_server_id;
        let delay = Duration::from_secs(notice_sec as u64);
        tokio::spawn(async move {
            if delay > Duration::ZERO {
                tokio::time::sleep(delay).await;
            }
            tracing::info!(server_id = %server_id, "выполняем рестарт сервера через wrapper");
            if let Err(e) = crate::wrapper::ops::power(&state_clone, server_id, "restart").await {
                tracing::error!(error = %e, server_id = %server_id, "ошибка вызова power restart во wrapper");
            }
        });
    }
    Ok(())
}
