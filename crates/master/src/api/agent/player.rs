//! Профиль игрока для агента: роли, доступ, наказания и текстуры одним
//! запросом — чтобы не держать игрока в лимбе на входе.

use std::collections::HashSet;
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
    Ok(Json(build(&state, row, &agent.game_server, HashSet::new()).await?))
}

/// `GET /api/agent/players/by-name/{username}`
pub async fn player_by_name(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(username): Path<String>,
) -> AppResult<Json<AgentPlayer>> {
    let row = crate::db::user_by_mc_username(&state.db, &username)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("no player named {username}")))?;
    Ok(Json(build(&state, row, &agent.game_server, HashSet::new()).await?))
}

const MAX_BATCH: usize = 500;

#[derive(serde::Deserialize)]
pub struct BatchReq {
    pub uuids: Vec<Uuid>,
}

/// `POST /api/agent/players/batch`
pub async fn players_batch(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<BatchReq>,
) -> AppResult<Json<Vec<AgentPlayer>>> {
    let uuids: Vec<Uuid> = req.uuids.into_iter().take(MAX_BATCH).collect();
    if uuids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let rows = crate::db::users_by_mc_uuids(&state.db, &uuids).await?;
    let user_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();

    let vanished_ids: HashSet<Uuid> =
        sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM player_vanish WHERE user_id = ANY($1)")
            .bind(&user_ids)
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .collect();

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(build(&state, row, &agent.game_server, vanished_ids.clone()).await?);
    }
    Ok(Json(out))
}

async fn build(
    state: &AppState,
    row: UserRow,
    game_server: &crate::db::game_servers::GameServerRow,
    vanished_ids: HashSet<Uuid>,
) -> AppResult<AgentPlayer> {
    let server_id = game_server.server_id;
    let profile = crate::db::load_profile(&state.db, row.id).await?;

    let active_mute = crate::db::punishments::active_mute_for_user(&state.db, row.id, Some(server_id))
        .await?
        .map(AgentPunishmentSummary::from);

    let active_ban = crate::db::punishments::active_ban_for_user(&state.db, row.id, Some(server_id))
        .await?
        .map(AgentPunishmentSummary::from);

    let pending_warns = crate::db::punishments::pending_warns(&state.db, row.id)
        .await?
        .into_iter()
        .map(AgentPunishmentSummary::from)
        .collect();

    let roles = profile
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

    let has_maintenance_bypass = profile.has_permission("noro.server.maintenance.bypass");

    let (allowed, denial_reason) = match crate::db::get_server(&state.db, server_id).await? {
        Some(server) => {
            if game_server.maintenance && !has_maintenance_bypass {
                (false, Some("maintenance".to_string()))
            } else if profile.banned {
                (false, Some("banned".to_string()))
            } else if !profile.can_join_server(&server_id, server.limited) {
                (false, Some("no_access".to_string()))
            } else {
                (true, None)
            }
        }
        None => (false, Some("server_not_found".to_string())),
    };

    let vanish_on_join = profile.silent_join || vanished_ids.contains(&row.id);

    let permissions: Vec<String> = profile.all_permissions().map(String::from).collect();

    Ok(AgentPlayer {
        uuid: profile.uuid,
        username: profile.username,
        banned: profile.banned,
        muted: active_mute.is_some(),
        active_mute,
        active_ban,
        pending_warns,
        allowed,
        denial_reason,
        maintenance_bypass: has_maintenance_bypass,
        roles,
        skin_url: profile
            .skin_url
            .unwrap_or_else(|| state.config.default_skin_url()),
        cape_url: profile.cape_url,
        locale: row.locale,
        lp_groups: profile
            .roles
            .iter()
            .filter_map(|r| r.lp_group.clone())
            .collect(),
        permissions,
        frozen: profile.freeze_info,
        vanish_on_join,
    })
}
