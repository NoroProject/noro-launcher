//! Подсказки прав для админки: что вообще можно выдать.
//!
//! Два источника, и они разной природы. Игровые узлы присылает агент — их
//! состав зависит от модов сборки и заранее неизвестен. Лаунчерные выводятся
//! из данных самого мастера: ограниченная сборка порождает право на вход,
//! ограниченный опциональный мод — право на него.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use schema::{PERM_ADMIN_ROLES, PERM_ADMIN_SERVERS, PERM_ADMIN_USERS};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct NodesQuery {
    /// Сборка, для которой собираем подсказки. Без неё — только лаунчерные.
    pub server_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct Suggestion {
    pub node: String,
    /// `game` — принесено агентом, `launcher` — выведено из данных мастера.
    pub source: &'static str,
    /// Человекочитаемое пояснение; у игровых узлов его нет.
    pub label: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(query): Query<NodesQuery>,
) -> AppResult<Json<Vec<Suggestion>>> {
    // Подсказками пользуются и на экране ролей, и на экране пользователя,
    // поэтому одного PERM_ADMIN_SERVERS мало: иначе редактор ролей получал бы
    // 403 и молча терял автодополнение.
    if [PERM_ADMIN_SERVERS, PERM_ADMIN_ROLES, PERM_ADMIN_USERS]
        .iter()
        .all(|perm| admin.require(perm).is_err())
    {
        return Err(crate::error::AppError::Forbidden(
            "нужно право на роли, пользователей или серверы".into(),
        ));
    }
    let mut out = builtin();

    for server in crate::db::list_servers(&state.db, false).await? {
        if server.limited {
            out.push(Suggestion {
                node: schema::perm_server_join(&server.id.to_string()),
                source: "launcher",
                label: Some(format!("Join build \u{201c}{}\u{201d}", server.name)),
            });
        }
    }

    out.extend(optional_mod_nodes(&state).await?);

    if let Some(server_id) = query.server_id {
        for node in crate::db::permission_nodes(&state.db, server_id).await? {
            out.push(Suggestion {
                node,
                source: "game",
                label: None,
            });
        }
    }

    out.sort_by(|a, b| a.node.cmp(&b.node));
    out.dedup_by(|a, b| a.node == b.node);
    Ok(Json(out))
}

/// Права ограниченных опциональных модов. Лежат в JSONB сборки, поэтому
/// собираются перебором, а не запросом.
async fn optional_mod_nodes(state: &AppState) -> AppResult<Vec<Suggestion>> {
    let mut out = Vec::new();
    for build in crate::db::builds_with_optional_mods(&state.db).await? {
        // optional_mods лежит JSONB'ом; разбираем в типизированный вид, а битую
        // запись пропускаем — подсказки не повод ронять админку.
        let mods: Vec<schema::OptionalMod> =
            serde_json::from_value(build.optional_mods.clone()).unwrap_or_default();
        for opt in mods.iter().filter(|m| m.limited) {
            out.push(Suggestion {
                node: schema::perm_optional_mod(&build.server_id.to_string(), &opt.name),
                source: "launcher",
                label: Some(format!("Optional mod \u{201c}{}\u{201d}", opt.name)),
            });
        }
    }
    Ok(out)
}

fn builtin() -> Vec<Suggestion> {
    [
        (schema::PERM_SUPERADMIN, "Everything, no limits"),
        (schema::PERM_ADMIN_ALL, "Full admin panel"),
        (schema::PERM_ADMIN_USERS, "Users"),
        (schema::PERM_ADMIN_SERVERS, "Builds and game servers"),
        (schema::PERM_ADMIN_BUILDS, "Build files"),
        (schema::PERM_ADMIN_NEWS, "News"),
        (schema::PERM_ADMIN_ROLES, "Roles"),
        (schema::PERM_ADMIN_LAUNCHER, "Launcher builds"),
        (schema::PERM_MOD_USERS_BAN, "Ban players"),
        (schema::PERM_LAUNCHER_BETA, "Launcher beta channel"),
    ]
    .into_iter()
    .map(|(node, label)| Suggestion {
        node: node.to_string(),
        source: "launcher",
        label: Some(label.to_string()),
    })
    .collect()
}
