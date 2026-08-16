//! Фоновая уборка протухших строк авторизации.
//!
//! Ни одна из этих таблиц раньше не чистилась: одноразовые коды и сессии
//! копились с первого дня и никогда не удалялись.

use sqlx::PgPool;
use std::time::Duration;

/// Как часто проходить по таблицам.
const INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Сколько сессия живёт после истечения access-токена.
///
/// Удалять строку сразу по `expires_at` нельзя: `refresh_session` обновляет
/// сессию, не глядя на срок, — то есть refresh-токен работает и после того, как
/// access протух. Снести такую строку значит разлогинить игрока, который просто
/// не заходил неделю. Отсечка нужна большая, она только ограничивает рост.
const SESSION_GRACE: &str = "90 days";

/// Запустить уборку в фоне.
pub fn spawn(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(INTERVAL);
        loop {
            ticker.tick().await;
            if let Err(e) = sweep(&pool).await {
                tracing::warn!(error = %e, "уборка протухших сессий не удалась");
            }
        }
    });
}

/// Один проход. Возвращает число удалённых строк по каждой таблице.
async fn sweep(pool: &PgPool) -> anyhow::Result<()> {
    // Одноразовые коды с коротким TTL: после истечения они бесполезны.
    let codes = delete(pool, "DELETE FROM oauth_codes WHERE expires_at < NOW()").await?;
    let launcher_codes = delete(
        pool,
        "DELETE FROM launcher_auth_codes WHERE expires_at < NOW()",
    )
    .await?;

    // Начатые и брошенные диалоги passkey: их никто уже не заберёт.
    let webauthn = delete(pool, "DELETE FROM webauthn_states WHERE expires_at < NOW()").await?;

    let sessions = delete(
        pool,
        &format!(
            "DELETE FROM oauth_sessions WHERE expires_at < NOW() - INTERVAL '{SESSION_GRACE}'"
        ),
    )
    .await?;

    if codes + launcher_codes + webauthn + sessions > 0 {
        tracing::info!(
            oauth_codes = codes,
            launcher_auth_codes = launcher_codes,
            webauthn_states = webauthn,
            oauth_sessions = sessions,
            "убраны протухшие строки авторизации"
        );
    }
    Ok(())
}

async fn delete(pool: &PgPool, sql: &str) -> anyhow::Result<u64> {
    Ok(sqlx::query(sql).execute(pool).await?.rows_affected())
}
