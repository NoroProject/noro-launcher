//! API для серверных агентов (плагин Paper, моды NeoForge и Fabric).
//!
//! Агент живёт на игровом сервере и знает игрока только по MC UUID. Мастер —
//! источник истины по ролям и доступу; LuckPerms получается его проекцией, а
//! не второй независимой базой.
//!
//! Авторизация — секретом конкретного игрового сервера, а не админ-токеном:
//! сервер должен уметь спросить только про себя. Из секрета же берётся, на
//! какой сервер заходит игрок, поэтому подменить его в запросе нельзя.

use crate::api::auth::AgentAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct AgentRole {
    pub name: String,
    pub display_name: String,
    /// Группа LuckPerms. `None` — роль в игру не проецируется.
    pub lp_group: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    /// Больше — важнее. Совпадает с весом группы в LuckPerms.
    pub sort_order: i32,
}

#[derive(Serialize)]
pub struct AgentPlayer {
    pub uuid: Uuid,
    pub username: String,
    pub banned: bool,
    /// Игроку разрешён вход на этот сервер. Агент обязан проверить: манифест
    /// сборки — не пропуск, до сервера можно дойти и мимо лаунчера.
    pub allowed: bool,
    pub roles: Vec<AgentRole>,
    pub skin_url: String,
    pub cape_url: Option<String>,
    /// Группы LuckPerms в порядке важности — готовый результат для агента,
    /// чтобы он не повторял у себя логику отбора.
    pub lp_groups: Vec<String>,
    /// Права, действующие на этом сервере: свои и от ролей, глобальные и
    /// привязанные к сборке. Плюс узлы `prefix.<вес>.<значение>` — LuckPerms
    /// хранит префикс так же, и моды, читающие права напрямую, ищут именно там.
    pub permissions: Vec<String>,
}

/// Профиль игрока для агента. Отдаёт всё, что нужно при входе: роли, группы,
/// бан и текстуры — одним запросом, чтобы не держать игрока в лимбе.
pub async fn player(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
) -> AppResult<Json<AgentPlayer>> {
    let row = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("игрок не найден".into()))?;
    let profile = crate::db::load_profile(&state.db, row.id).await?;

    // Доступ считает мастер, а не агент: правила ограниченных серверов уже
    // живут здесь и не должны расходиться между тремя реализациями агента.
    let server_id = agent.game_server.server_id;
    let allowed = match crate::db::get_server(&state.db, server_id).await? {
        Some(server) => !profile.banned && profile.can_join_server(&server_id, server.limited),
        None => false,
    };

    let mut roles: Vec<AgentRole> = profile
        .roles
        .iter()
        .map(|r| AgentRole {
            name: r.name.clone(),
            display_name: r.display_name.clone(),
            lp_group: r.lp_group.clone(),
            color: r.color.clone(),
            icon: r.icon.clone(),
            sort_order: r.sort_order,
        })
        .collect();
    // По убыванию важности: агент берёт первую роль как основную.
    roles.sort_by_key(|r| std::cmp::Reverse(r.sort_order));

    let lp_groups = roles.iter().filter_map(|r| r.lp_group.clone()).collect();

    let mut permissions = crate::db::effective_permissions(&state.db, row.id, server_id).await?;
    permissions.extend(super::agent_prefix::prefix_nodes(&roles));

    Ok(Json(AgentPlayer {
        uuid: profile.uuid,
        username: profile.username,
        banned: profile.banned,
        allowed,
        roles,
        // Скин есть всегда: у игрока свой либо общий Стив.
        skin_url: profile
            .skin_url
            .unwrap_or_else(|| state.config.default_skin_url()),
        cape_url: profile.cape_url,
        lp_groups,
        permissions,
    }))
}

#[derive(Deserialize)]
pub struct HeartbeatReq {
    pub online: u32,
    pub max_players: u32,
    /// Версия ядра/лоадера — видно в админке, помогает при разборе проблем.
    #[serde(default)]
    pub version: Option<String>,
}

/// Сигнал жизни от игрового сервера: обновляет онлайн и время последней связи.
/// Лаунчеру уходит `ServersChanged`, чтобы список пересчитал сумму.
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

    // Рассылать на каждый heartbeat — это шторм из 120 сообщений в час на
    // сервер ради чисел, которые лаунчер и так перечитает при открытии списка.
    // Важен только переход «сервер ожил» — карточка должна перестать быть
    // серой сразу.
    if !was_live {
        state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
