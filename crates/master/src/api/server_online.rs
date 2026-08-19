//! Список игроков на сервере (публичный и для персонала).

use crate::api::auth::OptionalAuthUser;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct OnlinePlayer {
    pub uuid: Uuid,
    pub username: String,
    pub skin_url: Option<String>,
    pub hidden: bool,
    pub vanished: bool,
}

#[derive(Serialize)]
pub struct ServerOnlineResponse {
    pub total_count: usize,
    pub players: Vec<OnlinePlayer>,
}

/// GET /api/servers/{server_id}/online — поимённый состав с учётом прав и hide_from_online.
pub async fn get_online(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    user: OptionalAuthUser,
) -> AppResult<Json<ServerOnlineResponse>> {
    let roster = state.roster.of(server_id);
    let all_uuids: Vec<Uuid> = roster.everyone().copied().collect();

    if all_uuids.is_empty() {
        return Ok(Json(ServerOnlineResponse {
            total_count: 0,
            players: vec![],
        }));
    }

    let is_staff = user
        .as_ref()
        .map(|u| {
            u.profile.has_permission("noro.mod.punish.view")
                || u.profile.has_permission("noro.admin.users.view")
        })
        .unwrap_or(false);

    let rows: Vec<(Uuid, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT mc_uuid, mc_username, skin_url, hide_from_online FROM users WHERE mc_uuid = ANY($1)",
    )
    .bind(&all_uuids)
    .fetch_all(&state.db)
    .await?;

    let total_count = all_uuids.len();
    let mut players = Vec::new();

    for (uuid, username, skin_url, hidden) in rows {
        let is_vanished = roster.vanished.contains(&uuid);
        if is_staff {
            players.push(OnlinePlayer {
                uuid,
                username,
                skin_url,
                hidden,
                vanished: is_vanished,
            });
        } else if !hidden && !is_vanished {
            players.push(OnlinePlayer {
                uuid,
                username,
                skin_url,
                hidden: false,
                vanished: false,
            });
        }
    }

    Ok(Json(ServerOnlineResponse {
        total_count,
        players,
    }))
}
