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

use chrono::{DateTime, Utc};

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
pub struct AgentPunishmentSummary {
    pub id: Uuid,
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct AgentPlayer {
    pub uuid: Uuid,
    pub username: String,
    pub banned: bool,
    pub muted: bool,
    pub active_mute: Option<AgentPunishmentSummary>,
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
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    let profile = crate::db::load_profile(&state.db, row.id).await?;

    // Доступ считает мастер, а не агент: правила ограниченных серверов уже
    // живут здесь и не должны расходиться между тремя реализациями агента.
    let server_id = agent.game_server.server_id;
    let allowed = match crate::db::get_server(&state.db, server_id).await? {
        Some(server) => !profile.banned && profile.can_join_server(&server_id, server.limited),
        None => false,
    };

    let mute_row = crate::db::active_mute_for_user(&state.db, row.id, Some(server_id)).await?;
    let (muted, active_mute) = match mute_row {
        Some(m) => (
            true,
            Some(AgentPunishmentSummary {
                id: m.id,
                kind: m.kind,
                reason: m.reason,
                actor_label: m.actor_label,
                created_at: m.created_at,
                expires_at: m.expires_at,
            }),
        ),
        None => (false, None),
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
        muted,
        active_mute,
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

#[derive(Deserialize)]
pub struct CreateAgentPunishmentReq {
    /// `mute` | `warn` | `ban` | `server_ban`
    pub kind: String,
    pub reason: String,
    /// Срок в минутах. None — навсегда.
    #[serde(default)]
    pub minutes: Option<i64>,
    /// Код правила из свода. Плагин присылает его из автодополнения, и по нему
    /// же мастер проверяет, что срок укладывается в рамки правила.
    #[serde(default)]
    pub rule_code: Option<String>,
}

/// GET /api/agent/players/{mc_uuid}/punishments
/// Получить список всех действующих и прошедших наказаний игрока.
pub async fn list_punishments(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
) -> AppResult<Json<Vec<crate::db::punishments::PunishmentRow>>> {
    let _ = agent;
    let row = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    Ok(Json(crate::db::list_punishments(&state.db, row.id).await?))
}

/// POST /api/agent/players/{mc_uuid}/punishments
/// Выдать наказание (например, мут или варн) от имени игрового сервера/плагина.
pub async fn create_punishment(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
    Json(req): Json<CreateAgentPunishmentReq>,
) -> AppResult<Json<crate::db::punishments::PunishmentRow>> {
    if !matches!(req.kind.as_str(), "ban" | "warn" | "server_ban" | "mute") {
        return Err(AppError::BadRequest("unknown punishment kind".into()));
    }
    if req.reason.trim().len() < 3 {
        return Err(AppError::BadRequest("reason is required".into()));
    }
    let row = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;

    let actor_label = format!("Agent: {}", agent.game_server.name);
    let server_id = if req.kind == "server_ban" || req.kind == "mute" {
        Some(agent.game_server.server_id)
    } else {
        None
    };

    // Рамки правила действуют и здесь: команда из игры не должна выдавать то,
    // чего свод не предусматривает. Байпаса у плагина нет — в игре некому
    // предъявить право, там действует сервер целиком.
    let rule = match req.rule_code.as_deref() {
        Some(code) => {
            crate::db::rule_by_code(&state.db, code, Some(agent.game_server.server_id)).await?
        }
        None => None,
    };
    if let Some(rule) = &rule {
        let sanctions = crate::db::sanctions_of_rule(&state.db, rule.id).await?;
        if !sanctions.is_empty()
            && !sanctions
                .iter()
                .any(|s| s.kind == req.kind && s.allows(req.minutes))
        {
            return Err(AppError::Forbidden(format!(
                "rule {} allows only: {}",
                rule.code,
                crate::api::admin::punish_limits::describe(&sanctions)
            )));
        }
    }

    let expires_at = req
        .minutes
        .map(|m| chrono::Utc::now() + chrono::Duration::minutes(m.max(1)));

    let punishment = crate::db::create_punishment(
        &state.db,
        crate::db::punishments::NewPunishment {
            user_id: row.id,
            kind: &req.kind,
            reason: req.reason.trim(),
            actor_id: None,
            actor_label: &actor_label,
            server_id,
            expires_at,
            rule_id: rule.as_ref().map(|r| r.id),
            rule_code: rule.as_ref().map(|r| r.code.as_str()),
        },
    )
    .await?;

    if req.kind == "ban" {
        crate::db::refresh_ban_flag(&state.db, row.id).await?;
    }

    Ok(Json(punishment))
}
