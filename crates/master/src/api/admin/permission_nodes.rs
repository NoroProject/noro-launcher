//! Подсказки прав для админки: что вообще можно выдать.
//!
//! Два источника, и они разной природы. Игровые узлы присылает агент — их
//! состав зависит от модов сборки и заранее неизвестен. Лаунчерные выводятся
//! из данных самого мастера: ограниченная сборка порождает право на вход,
//! ограниченный опциональный мод — право на него.

use crate::api::auth::AdminAuth;
use crate::api::paging::Page;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;

use schema::{PERM_ROLES_VIEW, PERM_SERVERS_VIEW, PERM_USERS_VIEW};

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
    /// Раздел для группировки в редакторе ролей.
    pub group: Option<&'static str>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(query): Query<NodesQuery>,
) -> AppResult<Json<Page<Suggestion>>> {
    // Подсказками пользуются и на экране ролей, и на экране пользователя,
    // поэтому одного права на серверы мало: иначе редактор ролей получал бы
    // 403 и молча терял автодополнение.
    if [PERM_ROLES_VIEW, PERM_USERS_VIEW, PERM_SERVERS_VIEW]
        .iter()
        .all(|perm| admin.require(perm).is_err())
    {
        return Err(crate::error::AppError::Forbidden(
            "a permission over roles, users or servers is required".into(),
        ));
    }
    let mut out = builtin();

    for server in crate::db::list_servers(&state.db, false).await? {
        if server.limited {
            out.push(Suggestion {
                node: schema::perm_server_join(&server.id.to_string()),
                source: "launcher",
                label: Some(format!("Join build \u{201c}{}\u{201d}", server.name)),
                group: Some("perm-group-access"),
            });
        }
    }

    // Узлы по сборкам: выдать тестеру превью-версию, не открывая остальные.
    // Плюс wildcard на сервер — чтобы не перевыдавать право на каждую новую.
    for server in crate::db::list_servers(&state.db, false).await? {
        let builds = crate::db::list_server_builds(&state.db, server.id).await?;
        if builds.is_empty() {
            continue;
        }
        out.push(Suggestion {
            node: format!("noro.build.{}.*", server.id),
            source: "launcher",
            label: Some(format!("All builds of \u{201c}{}\u{201d}", server.name)),
            group: Some("perm-group-access"),
        });
        for b in builds {
            out.push(Suggestion {
                node: schema::perm_build_access(&server.id.to_string(), &b.id.to_string()),
                source: "launcher",
                label: Some(format!(
                    "Build \u{201c}{}\u{201d} {}{}",
                    server.name,
                    b.version,
                    if b.published { "" } else { " (preview)" }
                )),
                group: Some("perm-group-access"),
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
                group: Some("perm-group-game"),
            });
        }
    }

    out.sort_by(|a, b| a.node.cmp(&b.node));
    out.dedup_by(|a, b| a.node == b.node);
    Ok(Json(Page::whole(out)))
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
                group: Some("perm-group-access"),
            });
        }
    }
    Ok(out)
}

/// Встроенные узлы приходят из реестра прав: список в двух местах разъезжался
/// бы ровно до первого нового права, и админ узнавал бы о нём из чужого кода.
fn builtin() -> Vec<Suggestion> {
    let mut out = vec![
        Suggestion {
            node: schema::PERM_SUPERADMIN.to_string(),
            source: "launcher",
            label: Some("perm-superadmin-desc".into()),
            group: Some("perm-group-panel"),
        },
        Suggestion {
            node: schema::PERM_ADMIN_ALL.to_string(),
            source: "launcher",
            label: Some("perm-admin-all-desc".into()),
            group: Some("perm-group-panel"),
        },
    ];
    // Ветку целиком («все права на игроков») выдают чаще, чем перечисляют узлы
    // по одному, поэтому шаблоны веток тоже попадают в подсказки.
    let mut branches: Vec<&str> = schema::ALL_NODES
        .iter()
        .filter_map(|n| n.name.rsplit_once('.').map(|(prefix, _)| prefix))
        .collect();
    branches.sort_unstable();
    branches.dedup();
    for prefix in branches {
        out.push(Suggestion {
            node: format!("{prefix}.*"),
            source: "launcher",
            label: Some(format!("Все права в ветке {prefix}")),
            group: Some("perm-group-branches"),
        });
    }
    out.extend(schema::ALL_NODES.iter().map(|node| Suggestion {
        node: node.name.to_string(),
        source: "launcher",
        label: Some(node.title.to_string()),
        group: Some(node.group),
    }));
    out
}
