//! Сигнал жизни от игрового сервера.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

/// Всё, кроме первых трёх полей, необязательно: старый агент должен работать с
/// новым мастером, а новый — со старым. Поэтому расширение контракта здесь
/// всегда идёт через `Option` и `#[serde(default)]`.
#[derive(Deserialize)]
pub struct HeartbeatReq {
    pub online: u32,
    pub max_players: u32,
    /// Версия ядра/лоадера — видно в админке, помогает при разборе проблем.
    #[serde(default)]
    pub version: Option<String>,
    /// Полный состав видимых игроков. Именно он, а не события `player_join`,
    /// задаёт истину: канал рвётся, и без сверки в списке копились бы призраки.
    #[serde(default)]
    pub players: Option<Vec<Uuid>>,
    /// Скрытые ванишем. Отдельно от `players`: в публичном числе их нет, а в
    /// составе для стаффа и в сессиях — есть.
    #[serde(default)]
    pub vanished: Option<Vec<Uuid>>,
    /// Телеметрия: тики, память, аптайм. Хранение — п.7, отдельной таблицей с
    /// TTL; здесь поля объявлены, чтобы контракт был один и агент не гадал,
    /// какой мастер их поймёт.
    #[serde(flatten)]
    pub telemetry: Telemetry,
}

/// Замеры сервера на момент сигнала. Всё необязательно: у платформы может не
/// быть счётчика тиков, а у старого агента — самих полей.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct Telemetry {
    #[serde(default)]
    pub tps: Option<f64>,
    #[serde(default)]
    pub mspt: Option<f64>,
    #[serde(default)]
    pub heap_used_mb: Option<i64>,
    #[serde(default)]
    pub heap_max_mb: Option<i64>,
    #[serde(default)]
    pub uptime_secs: Option<i64>,
}

/// Обновляет онлайн и время последней связи. Лаунчеру уходит `ServersChanged`,
/// чтобы список пересчитал сумму.
pub async fn heartbeat(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<HeartbeatReq>,
) -> AppResult<Json<serde_json::Value>> {
    let was_live = agent.game_server.live();
    crate::db::touch_game_server(
        &state.db,
        agent.game_server.id,
        req.online as i32,
        req.max_players as i32,
        req.version.as_deref(),
    )
    .await?;

    // Состав присылает только новый агент. У старого поля нет — и тогда список
    // ведут одни события: затирать его пустотой значит показать пустой сервер
    // там, где просто не обновили агента.
    if let Some(players) = req.players {
        state.roster.reconcile(
            agent.game_server.id,
            players,
            req.vanished.unwrap_or_default(),
        );
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO server_telemetry (game_server_id, tps, mspt, heap_used_mb, heap_max_mb, online_players)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(agent.game_server.id)
    .bind(req.telemetry.tps)
    .bind(req.telemetry.mspt)
    .bind(req.telemetry.heap_used_mb)
    .bind(req.telemetry.heap_max_mb)
    .bind(req.online as i32)
    .execute(&state.db)
    .await {
        tracing::warn!(server = %agent.game_server.name, error = %e, "не удалось записать телеметрию");
    }

    // Рассылать на каждый heartbeat — это шторм из 120 сообщений в час на
    // сервер ради чисел, которые лаунчер и так перечитает при открытии списка.
    // Важен только переход «сервер ожил» — карточка должна перестать быть
    // серой сразу.
    if !was_live {
        state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
