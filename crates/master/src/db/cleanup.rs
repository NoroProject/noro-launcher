//! Background sweep of expired auth rows and logs.

use sqlx::PgPool;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60 * 60);

/// How long a session sticks around after its access token expires.
const SESSION_GRACE: &str = "90 days";

pub fn spawn(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(INTERVAL);
        loop {
            ticker.tick().await;
            if let Err(e) = sweep(&pool).await {
                tracing::warn!(error = %e, "cleanup sweep failed");
            }
        }
    });
}

/// One pass over every table. Counts go to the log, not to the caller.
async fn sweep(pool: &PgPool) -> anyhow::Result<()> {
    let codes = delete(pool, "DELETE FROM oauth_codes WHERE expires_at < NOW()").await?;
    let launcher_codes = delete(
        pool,
        "DELETE FROM launcher_auth_codes WHERE expires_at < NOW()",
    )
    .await?;

    let unbanned = crate::db::expire_punishments(pool).await?;
    if unbanned > 0 {
        tracing::info!(count = unbanned, "expired punishments lifted");
    }

    let bundles = delete(pool, "DELETE FROM support_bundles WHERE expires_at < NOW()").await?;

    let triggers = delete(
        pool,
        "DELETE FROM automod_triggers WHERE created_at < NOW() - INTERVAL '30 days'",
    )
    .await?;

    // Начатые и брошенные диалоги passkey.
    let webauthn = delete(pool, "DELETE FROM webauthn_states WHERE expires_at < NOW()").await?;

    // Закрытие потерянных игровых сессий (сервер упал, агент перезапущен)
    let lost_sessions = sqlx::query(
        "UPDATE player_sessions ps
         SET ended_at = COALESCE(gs.last_seen_at, NOW()), end_reason = 'lost'
         FROM game_servers gs
         WHERE ps.game_server_id = gs.id
           AND ps.ended_at IS NULL
           AND (gs.last_seen_at IS NULL OR gs.last_seen_at < NOW() - INTERVAL '90 seconds')",
    )
    .execute(pool)
    .await?
    .rows_affected();
    if lost_sessions > 0 {
        tracing::info!(
            count = lost_sessions,
            "закрыты потерянные игровые сессии (lost)"
        );
    }

    // Агрегирование сырой телеметрии старше 7 дней в часовые бакеты перед её удалением
    let _ = sqlx::query(
        "INSERT INTO server_telemetry_hourly
            (game_server_id, bucket_hour, tps_min, tps_max, tps_avg, mspt_avg, online_max)
         SELECT
            game_server_id,
            date_trunc('hour', created_at) AS bucket_hour,
            MIN(tps) AS tps_min,
            MAX(tps) AS tps_max,
            AVG(tps) AS tps_avg,
            AVG(mspt) AS mspt_avg,
            MAX(online_players) AS online_max
         FROM server_telemetry
         WHERE created_at < NOW() - INTERVAL '7 days'
         GROUP BY game_server_id, date_trunc('hour', created_at)
         ON CONFLICT (game_server_id, bucket_hour) DO NOTHING",
    )
    .execute(pool)
    .await;

    // Сырая телеметрия старше 7 дней
    let telemetry = delete(
        pool,
        "DELETE FROM server_telemetry WHERE created_at < NOW() - INTERVAL '7 days'",
    )
    .await?;

    let sessions = delete(
        pool,
        &format!(
            "DELETE FROM oauth_sessions WHERE expires_at < NOW() - INTERVAL '{SESSION_GRACE}'"
        ),
    )
    .await?;

    if codes + launcher_codes + webauthn + bundles + sessions + telemetry + triggers > 0 {
        tracing::info!(
            oauth_codes = codes,
            launcher_auth_codes = launcher_codes,
            webauthn_states = webauthn,
            support_bundles = bundles,
            oauth_sessions = sessions,
            server_telemetry = telemetry,
            automod_triggers = triggers,
            "убраны протухшие строки авторизации, телеметрии и логов"
        );
    }
    Ok(())
}

async fn delete(pool: &PgPool, sql: &str) -> anyhow::Result<u64> {
    Ok(sqlx::query(sql).execute(pool).await?.rows_affected())
}
