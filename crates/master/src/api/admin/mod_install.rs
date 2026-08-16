//! Установка мода в выбранные цели.
//!
//! Один источник — сколько угодно назначений: клиентская сборка, конкретный
//! игровой сервер, все сервера сборки сразу. Файл скачивается один раз, дальше
//! расходится по целям, и каждая отчитывается отдельно: упавший сервер не
//! должен отменять установку в остальные.

use super::builds::{append_optional_mod, optional_from_draft, OptionalModDraft};
use crate::api::auth::AdminAuth;
use crate::catalog::{resolve, ModSource, ResolvedMod};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ADMIN_BUILDS;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InstallReq {
    pub source: ModSource,
    pub targets: Vec<InstallTarget>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstallTarget {
    Build {
        build_id: Uuid,
        /// Заполнено — мод попадёт в список опциональных сборки.
        #[serde(default)]
        optional: Option<OptionalModDraft>,
    },
    GameServer {
        id: Uuid,
    },
    /// Разворачивается в игровые сервера этой сборки.
    AllServers {
        server_id: Uuid,
    },
}

#[derive(Serialize)]
pub struct TargetResult {
    pub kind: &'static str,
    pub id: Uuid,
    pub label: String,
    pub ok: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

pub async fn install(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<InstallReq>,
) -> AppResult<Json<Vec<TargetResult>>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    if req.targets.is_empty() {
        return Err(AppError::BadRequest("не выбрано ни одной цели".into()));
    }
    let resolved = resolve::resolve(&state, &req.source).await?;

    let mut results = Vec::new();
    // Сборок в одном запросе может быть несколько; лаунчеру хватит одного
    // сообщения на сервер, а не одного на цель.
    let mut changed: HashSet<Uuid> = HashSet::new();

    for target in expand(&state, req.targets).await? {
        results.push(apply(&state, target, &resolved, &mut changed).await);
    }
    for server_id in changed {
        super::builds::broadcast_builds_changed(&state, server_id);
    }
    Ok(Json(results))
}

/// `all_servers` превращается в конкретные сервера ещё до установки — так в
/// отчёте видно каждый, а не одна строка «применено куда-то».
async fn expand(state: &AppState, targets: Vec<InstallTarget>) -> AppResult<Vec<InstallTarget>> {
    let mut out = Vec::new();
    for target in targets {
        match target {
            InstallTarget::AllServers { server_id } => {
                for gs in crate::db::list_game_servers(&state.db, server_id).await? {
                    // Прокси модов не держит: там нет ни mods/, ни plugins/.
                    if !gs.is_proxy() {
                        out.push(InstallTarget::GameServer { id: gs.id });
                    }
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

async fn apply(
    state: &AppState,
    target: InstallTarget,
    resolved: &ResolvedMod,
    changed: &mut HashSet<Uuid>,
) -> TargetResult {
    match target {
        InstallTarget::Build { build_id, optional } => {
            let outcome = into_build(state, build_id, optional, resolved, changed).await;
            result("build", build_id, resolved, outcome)
        }
        InstallTarget::GameServer { id } => {
            let outcome = crate::wrapper::ops::install_mod(state, id, resolved).await;
            result("game_server", id, resolved, outcome)
        }
        // Развёрнут в expand().
        InstallTarget::AllServers { server_id } => TargetResult {
            kind: "all_servers",
            id: server_id,
            label: String::new(),
            ok: true,
            path: None,
            error: None,
        },
    }
}

fn result(
    kind: &'static str,
    id: Uuid,
    resolved: &ResolvedMod,
    outcome: AppResult<String>,
) -> TargetResult {
    match outcome {
        Ok(path) => TargetResult {
            kind,
            id,
            label: resolved.filename.clone(),
            ok: true,
            path: Some(path),
            error: None,
        },
        Err(e) => TargetResult {
            kind,
            id,
            label: resolved.filename.clone(),
            ok: false,
            path: None,
            error: Some(e.to_string()),
        },
    }
}

async fn into_build(
    state: &AppState,
    build_id: Uuid,
    optional: Option<OptionalModDraft>,
    m: &ResolvedMod,
    changed: &mut HashSet<Uuid>,
) -> AppResult<String> {
    let path = format!("mods/{}", m.filename);

    // Удалим прошлую версию этого же мода из сборки, если она там лежит.
    let new_disk_path = state.files.path_for(&m.sha1);
    let new_meta = backend::mod_icon::extract_jar_metadata(&new_disk_path);

    if let Ok(existing_files) = crate::db::build_files(&state.db, build_id).await {
        for f in existing_files {
            if f.path.starts_with("mods/") && f.path != path {
                let old_disk_path = state.files.path_for(&f.sha1);
                let old_meta = backend::mod_icon::extract_jar_metadata(&old_disk_path);

                let is_same_mod = match (&new_meta, &old_meta) {
                    (Some(n), Some(o)) if n.mod_id.is_some() && n.mod_id == o.mod_id => true,
                    _ => {
                        let new_base = stem_mod_name(&m.filename);
                        let old_base = stem_mod_name(&f.path);
                        !new_base.is_empty() && new_base == old_base
                    }
                };

                if is_same_mod {
                    tracing::info!(
                        "Removing older mod version file '{}' from build {} to install '{}'",
                        f.path,
                        build_id,
                        path
                    );
                    let _ = crate::db::delete_build_file(&state.db, f.id).await;
                }
            }
        }
    }

    crate::db::upsert_build_file(
        &state.db,
        build_id,
        &path,
        &m.sha1,
        m.size as i64,
        "both",
        "mod",
    )
    .await?;
    if let Some(draft) = optional {
        // Значения из админки идут первыми: она берёт их из карточки поиска,
        // где есть автор. В карточке проекта Modrinth его нет вовсе, так что
        // «уточнить» их метаданными установки не выйдет — только затереть.
        let icon = draft.icon_url.clone().or_else(|| m.icon_url.clone());
        let author = draft.author.clone().or_else(|| m.author.clone());
        let entry = optional_from_draft(draft, path.clone(), icon, author);
        append_optional_mod(state, build_id, entry).await?;
    }
    changed.insert(super::builds::build_server_id(state, build_id).await?);
    Ok(path)
}

fn stem_mod_name(filename: &str) -> String {
    let name = filename.strip_prefix("mods/").unwrap_or(filename);
    let name = name.strip_suffix(".jar").unwrap_or(name);
    let clean = name
        .split(['-', '_', '+'])
        .take_while(|part| !part.chars().next().is_some_and(|c| c.is_ascii_digit() || c == 'v'))
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    if clean.is_empty() {
        name.to_lowercase()
    } else {
        clean
    }
}
