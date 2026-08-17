//! Профиль игрока для агента: роли, доступ, наказания и текстуры одним
//! запросом — чтобы не держать игрока в лимбе на входе.

use super::types::{AgentPlayer, AgentPunishmentSummary, AgentRole};
use crate::api::auth::AgentAuth;
use crate::db::models::UserRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

/// `GET /api/agent/players/{mc_uuid}`
pub async fn player(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
) -> AppResult<Json<AgentPlayer>> {
    let row = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    Ok(Json(build(&state, row, agent.game_server.server_id).await?))
}

/// `GET /api/agent/players/by-name/{username}` — команды модерации называют
/// игрока ником, а оффлайн-игрока в UUID по нику на сервере перевести нечем:
/// ванильный кэш знает только тех, кто заходил.
pub async fn player_by_name(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(username): Path<String>,
) -> AppResult<Json<AgentPlayer>> {
    let row = crate::db::user_by_mc_username(&state.db, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("no player named {username}")))?;
    Ok(Json(build(&state, row, agent.game_server.server_id).await?))
}

async fn build(state: &AppState, row: UserRow, server_id: Uuid) -> AppResult<AgentPlayer> {
    let profile = crate::db::load_profile(&state.db, row.id).await?;

    // Доступ считает мастер, а не агент: правила ограниченных серверов уже
    // живут здесь и не должны расходиться между тремя реализациями агента.
    let allowed = match crate::db::get_server(&state.db, server_id).await? {
        Some(server) => !profile.banned && profile.can_join_server(&server_id, server.limited),
        None => false,
    };

    let active_mute = crate::db::active_mute_for_user(&state.db, row.id, Some(server_id))
        .await?
        .map(AgentPunishmentSummary::from);
    let pending_warns = crate::db::pending_warns(&state.db, row.id)
        .await?
        .into_iter()
        .map(AgentPunishmentSummary::from)
        .collect();

    let mut roles: Vec<AgentRole> = profile
        .roles
        .iter()
        .map(|r| AgentRole {
            name: r.name.clone(),
            display_name: r.display_name.clone(),
            lp_group: r.lp_group.clone(),
            color: r.color.clone(),
            icon: r.icon.clone(),
            prefix: r.prefix.clone(),
            suffix: r.suffix.clone(),
            sort_order: r.sort_order,
        })
        .collect();
    // По убыванию важности: агент берёт первую роль как основную.
    roles.sort_by_key(|r| std::cmp::Reverse(r.sort_order));

    let lp_groups = roles.iter().filter_map(|r| r.lp_group.clone()).collect();

    let mut permissions = crate::db::effective_permissions(&state.db, row.id, server_id).await?;
    permissions.extend(crate::api::agent_prefix::meta_nodes(&roles));

    Ok(AgentPlayer {
        uuid: profile.uuid,
        username: profile.username,
        banned: profile.banned,
        muted: active_mute.is_some(),
        active_mute,
        pending_warns,
        allowed,
        roles,
        // Скин есть всегда: у игрока свой либо общий Стив.
        skin_url: profile
            .skin_url
            .unwrap_or_else(|| state.config.default_skin_url()),
        cape_url: profile.cape_url,
        lp_groups,
        permissions,
    })
}
