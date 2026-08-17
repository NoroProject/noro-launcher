//! Админ: сборки — создание, публикация (bootstrap + подпись), файлы, моды,
//! опциональные моды, импорт mrpack/CurseForge.

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::db::models::{BuildFileRow, BuildRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use schema::{OptionalMod, PERM_BUILDS_DELETE, PERM_BUILDS_EDIT, PERM_BUILDS_IMPORT, PERM_BUILDS_PUBLISH, PERM_BUILDS_VIEW};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

pub(super) fn broadcast_builds_changed(state: &AppState, server_id: Uuid) {
    state
        .ws
        .broadcast(&schema::ServerWsMsg::BuildsChanged { server_id });
}

pub(super) async fn build_server_id(state: &AppState, id: Uuid) -> AppResult<Uuid> {
    let build = crate::db::get_build(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;
    Ok(build.server_id)
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub server_id: Uuid,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<BuildRow>>> {
    admin.require(PERM_BUILDS_VIEW)?;
    Ok(Json(crate::db::list_builds(&state.db, q.server_id).await?))
}

pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_VIEW)?;
    let build = crate::db::get_build(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;
    let files = crate::db::build_files(&state.db, id).await?;
    Ok(Json(json!({ "build": build, "file_count": files.len() })))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub server_id: Uuid,
    pub version: String,
    pub modloader: String,
    pub modloader_version: Option<String>,
    pub mc_version: String,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let id = crate::db::create_build(
        &state.db,
        req.server_id,
        &req.version,
        &req.modloader,
        req.modloader_version.as_deref(),
        &req.mc_version,
    )
    .await?;
    broadcast_builds_changed(&state, req.server_id);
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct DuplicateReq {
    pub version: String,
}

/// Создать новую сборку как копию существующей.
///
/// Копия приходит черновиком со всеми файлами и настройками оригинала: дальше
/// достаточно заменить пару модов и опубликовать, а не собирать модпак заново.
pub async fn duplicate(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<DuplicateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;

    let version = req.version.trim();
    if version.is_empty() {
        return Err(AppError::BadRequest("version cannot be empty".into()));
    }

    let server_id = crate::db::build_server_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build not found".into()))?;

    let new_id = crate::db::duplicate_build(&state.db, id, version).await?;
    broadcast_builds_changed(&state, server_id);

    Ok(Json(json!({ "id": new_id })))
}

/// Опубликовать сборку: bootstrap артефактов + подпись манифеста.
/// Выполняется синхронно (может занять минуты при первом скачивании ассетов/java).
pub async fn publish(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_PUBLISH)?;
    let manifest = rebuild_manifest(&state, id, false).await?;
    crate::db::set_build_published(&state.db, id, true).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::BUILD_PUBLISH,
        target("build", id),
        json!({ "summary": crate::manifest::manifest_summary(&manifest) }),
    )
    .await;
    broadcast_builds_changed(&state, manifest.server_id);

    Ok(Json(json!({
        "ok": true,
        "summary": crate::manifest::manifest_summary(&manifest),
    })))
}

/// Пересобрать Minecraft/loader артефакты и переподписать manifest без смены статуса publish.
pub async fn rebuild(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let manifest = rebuild_manifest(&state, id, false).await?;
    broadcast_builds_changed(&state, manifest.server_id);

    Ok(Json(json!({
        "ok": true,
        "summary": crate::manifest::manifest_summary(&manifest),
    })))
}

/// То же, но с нуля: base build сносится и качается заново мимо кэша стора.
///
/// Обычный `rebuild` переиспользует всё, что уже считается готовым, и потому
/// не лечит испортившееся — свёрнутые под одну ОС аргументы или битый блоб.
/// Моды и конфиги сборки остаются: их bootstrap воссоздать не может.
pub async fn rebuild_clean(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let manifest = rebuild_manifest(&state, id, true).await?;
    broadcast_builds_changed(&state, manifest.server_id);

    Ok(Json(json!({
        "ok": true,
        "summary": crate::manifest::manifest_summary(&manifest),
    })))
}

/// `clean` — снести base build и качать заново вместо переиспользования готового.
async fn rebuild_manifest(
    state: &AppState,
    id: Uuid,
    clean: bool,
) -> AppResult<schema::BuildManifest> {
    let build = crate::db::get_build(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;

    let log = |msg: &str| {
        tracing::info!(target: "bootstrap", "{msg}");
    };
    let mc = &build.mc_version;
    let loader = &build.modloader;
    let loader_version = build.modloader_version.as_deref();
    if clean {
        crate::mojang_bootstrap::rebuild_base_build(state, mc, loader, loader_version, log).await
    } else {
        crate::mojang_bootstrap::ensure_base_build(state, mc, loader, loader_version, log).await
    }
    .map_err(AppError::Other)?;

    let build = crate::db::get_build(&state.db, id).await?.unwrap();
    crate::manifest::ensure_signed(state, &build)
        .await
        .map_err(AppError::Other)
}

pub async fn unpublish(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_PUBLISH)?;
    let server_id = build_server_id(&state, id).await?;
    crate::db::set_build_published(&state.db, id, false).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::BUILD_UNPUBLISH,
        target("build", id),
        json!({}),
    )
    .await;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct SetVersionsReq {
    pub mc_version: String,
    pub modloader_version: String,
}

pub async fn set_versions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<SetVersionsReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let server_id = build_server_id(&state, id).await?;
    sqlx::query("UPDATE builds SET mc_version = $1, modloader_version = $2 WHERE id = $3")
        .bind(&req.mc_version)
        .bind(&req.modloader_version)
        .bind(id)
        .execute(&state.db)
        .await?;
    crate::db::set_build_published(&state.db, id, false).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_DELETE)?;
    let server_id = build_server_id(&state, id).await?;
    crate::db::delete_build(&state.db, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

// --- Файлы ---

pub async fn list_files(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<BuildFileRow>>> {
    admin.require(PERM_BUILDS_EDIT)?;
    Ok(Json(crate::db::build_files(&state.db, id).await?))
}

/// Загрузить файл сборки (multipart: поле `path` + поле `file`).
pub async fn upload_file(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let mut path: Option<String> = None;
    let mut data: Option<bytes::Bytes> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("path") => {
                path = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            Some("file") => {
                filename = field.file_name().map(String::from);
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            _ => {}
        }
    }

    let data = data.ok_or_else(|| AppError::BadRequest("missing the file field".into()))?;
    // Путь: явный, иначе mods/<filename>.
    let path = path
        .filter(|p| !p.is_empty())
        .or_else(|| filename.map(|f| format!("mods/{f}")))
        .ok_or_else(|| AppError::BadRequest("no path given".into()))?;

    let stored = state
        .files
        .put_bytes(&data)
        .await
        .map_err(AppError::Other)?;
    let kind = guess_kind(&path);
    crate::db::upsert_build_file(
        &state.db,
        id,
        &path,
        &stored.sha1,
        stored.size as i64,
        "both",
        kind,
    )
    .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(
        json!({ "path": path, "sha1": stored.sha1, "size": stored.size }),
    ))
}

pub async fn delete_file(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, file_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let server_id = build_server_id(&state, id).await?;
    crate::db::delete_build_file(&state.db, file_id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

/// Получить текстовое содержимое файла сборки (только для текстовых файлов).
pub async fn get_file_content(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<FileContentQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let files = crate::db::build_files(&state.db, id).await?;
    let file = files
        .into_iter()
        .find(|f| f.path == q.path)
        .ok_or_else(|| AppError::NotFound("file".into()))?;

    // Ограничение размера для текстового редактора
    if file.size > 512 * 1024 {
        return Err(AppError::BadRequest(
            "file too large for text editor".into(),
        ));
    }

    let mut f = state
        .files
        .open(&file.sha1)
        .await
        .map_err(AppError::Other)?;
    let mut buf = Vec::new();
    use tokio::io::AsyncReadExt;
    f.read_to_end(&mut buf)
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let content =
        String::from_utf8(buf).map_err(|_| AppError::BadRequest("not a valid text file".into()))?;

    Ok(Json(json!({
        "path": file.path,
        "sha1": file.sha1,
        "size": file.size,
        "content": content,
    })))
}

/// Извлечь иконку мода прямо из содержимого .jar файла сборки с персистентным дисковым кэшем.
pub async fn get_file_icon(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<FileContentQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let clean = clean_path(&q.path)?;
    let files = crate::db::build_files(&state.db, id).await?;
    let file = files
        .into_iter()
        .find(|f| f.path == clean)
        .ok_or_else(|| AppError::NotFound("file".into()))?;

    let cache_dir = state.config.data_dir.join("cache").join("mod_icons");
    let cache_file = cache_dir.join(format!("{}.txt", file.sha1));

    if cache_file.exists() {
        if let Ok(cached_url) = tokio::fs::read_to_string(&cache_file).await {
            let icon_url = if cached_url.is_empty() {
                None
            } else {
                Some(cached_url)
            };
            return Ok(Json(json!({
                "path": file.path,
                "sha1": file.sha1,
                "icon_url": icon_url,
            })));
        }
    }

    let path = state.files.path_for(&file.sha1);
    let icon_url = backend::mod_icon::extract_jar_icon(&path);

    let _ = tokio::fs::create_dir_all(&cache_dir).await;
    let _ = tokio::fs::write(&cache_file, icon_url.as_deref().unwrap_or("")).await;

    Ok(Json(json!({
        "path": file.path,
        "sha1": file.sha1,
        "icon_url": icon_url,
    })))
}

/// Обновить содержимое текстового файла (создаёт новую версию по SHA1).
pub async fn update_file_content(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFileContentReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let data = req.content.as_bytes();
    if data.len() > 512 * 1024 {
        return Err(AppError::BadRequest("content too large".into()));
    }

    let stored = state.files.put_bytes(data).await.map_err(AppError::Other)?;

    let kind = guess_kind(&req.path);
    crate::db::upsert_build_file(
        &state.db,
        id,
        &req.path,
        &stored.sha1,
        stored.size as i64,
        "both",
        kind,
    )
    .await?;

    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);

    Ok(Json(
        json!({ "path": req.path, "sha1": stored.sha1, "size": stored.size }),
    ))
}

/// Удалить все файлы сборки по префиксу пути (для удаления папок).
pub async fn delete_files_by_prefix(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<FileContentQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let prefix = if q.path.ends_with('/') {
        q.path.clone()
    } else {
        format!("{}/", q.path)
    };
    sqlx::query("DELETE FROM build_files WHERE build_id = $1 AND path LIKE $2 || '%'")
        .bind(id)
        .bind(&prefix)
        .execute(&state.db)
        .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct MoveReq {
    pub from: String,
    pub to: String,
}

/// Переименовать файл или папку.
///
/// <p>Содержимое остаётся на месте: в базе лежат только пути, поэтому и для
/// папки это просто смена префикса у её файлов — тот же приём, что у MOVE в
/// WebDAV.
pub async fn move_files(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let from = clean_path(&req.from)?;
    let to = clean_path(&req.to)?;
    // Переезд папки внутрь себя оставил бы её файлы без пути наверх.
    if to == from || to.starts_with(&format!("{from}/")) {
        return Err(AppError::BadRequest(
            "the destination path is inside the source".into(),
        ));
    }

    let prefix = format!("{from}/");
    let targets: Vec<BuildFileRow> = crate::db::build_files(&state.db, id)
        .await?
        .into_iter()
        .filter(|f| f.path == from || f.path.starts_with(&prefix))
        .collect();
    if targets.is_empty() {
        return Err(AppError::NotFound(format!("no files under {from}")));
    }

    for file in &targets {
        let moved = format!("{to}{}", &file.path[from.len()..]);
        crate::db::upsert_build_file(
            &state.db,
            id,
            &moved,
            &file.sha1,
            file.size,
            &file.side,
            guess_kind(&moved),
        )
        .await?;
        crate::db::delete_build_file(&state.db, file.id).await?;
    }

    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "moved": targets.len() })))
}

/// Путь внутри сборки: без ведущих и хвостовых слэшей, без `..` и пустых
/// сегментов — иначе переименование стало бы способом писать мимо сборки.
fn clean_path(raw: &str) -> AppResult<String> {
    let path = raw.trim().trim_matches('/');
    if path.is_empty() || path.split('/').any(|part| part == ".." || part.is_empty()) {
        return Err(AppError::BadRequest(format!("invalid path: {raw}")));
    }
    Ok(path.to_string())
}

fn guess_kind(path: &str) -> &'static str {
    if path.starts_with("mods/") {
        "mod"
    } else if path.starts_with("config/") || path.ends_with(".toml") || path.ends_with(".json") {
        "config"
    } else {
        "other"
    }
}

// --- Импорт ---

pub async fn import_mrpack(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_IMPORT)?;
    let server_id = build_server_id(&state, id).await?;
    let bytes = crate::build_importer::read_upload(multipart)
        .await
        .map_err(AppError::Other)?;

    let job_id = Uuid::new_v4();
    state
        .import_jobs
        .insert(job_id, crate::build_importer::ImportProgress::default());

    let state_clone = state.clone();
    tokio::spawn(async move {
        let res = crate::build_importer::mrpack::import(&state_clone, id, job_id, bytes).await;
        if let Some(mut prog) = state_clone.import_jobs.get_mut(&job_id) {
            prog.done = true;
            if let Err(e) = res {
                prog.error = Some(e.to_string());
            }
        }
        broadcast_builds_changed(&state_clone, server_id);
    });

    Ok(Json(json!({ "job_id": job_id })))
}

pub async fn import_curseforge(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_IMPORT)?;
    let server_id = build_server_id(&state, id).await?;
    let bytes = crate::build_importer::read_upload(multipart)
        .await
        .map_err(AppError::Other)?;

    let job_id = Uuid::new_v4();
    state
        .import_jobs
        .insert(job_id, crate::build_importer::ImportProgress::default());

    let state_clone = state.clone();
    tokio::spawn(async move {
        let res = crate::build_importer::curseforge::import(&state_clone, id, job_id, bytes).await;
        if let Some(mut prog) = state_clone.import_jobs.get_mut(&job_id) {
            prog.done = true;
            if let Err(e) = res {
                prog.error = Some(e.to_string());
            }
        }
        broadcast_builds_changed(&state_clone, server_id);
    });

    Ok(Json(json!({ "job_id": job_id })))
}

/// Импорт обычного zip, внутри которого лежит корень сборки.
pub async fn import_instance_zip(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_IMPORT)?;
    let server_id = build_server_id(&state, id).await?;
    let bytes = crate::build_importer::read_upload(multipart)
        .await
        .map_err(AppError::Other)?;

    let job_id = Uuid::new_v4();
    state
        .import_jobs
        .insert(job_id, crate::build_importer::ImportProgress::default());

    let state_clone = state.clone();
    tokio::spawn(async move {
        let res =
            crate::build_importer::instance_zip::import(&state_clone, id, job_id, bytes).await;
        if let Some(mut prog) = state_clone.import_jobs.get_mut(&job_id) {
            prog.done = true;
            if let Err(e) = res {
                prog.error = Some(e.to_string());
            }
        }
        broadcast_builds_changed(&state_clone, server_id);
    });

    Ok(Json(json!({ "job_id": job_id })))
}

pub async fn import_progress(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_build_id, job_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<crate::build_importer::ImportProgress>> {
    admin.require(PERM_BUILDS_EDIT)?;
    if let Some(prog) = state.import_jobs.get(&job_id) {
        Ok(Json(prog.clone()))
    } else {
        Err(AppError::NotFound("job".into()))
    }
}

// --- Опциональные моды ---

/// Заготовка карточки опционального мода: приходит из админки вместе с
/// установкой, потому что имя и категорию задаёт человек, а не Modrinth.
#[derive(Deserialize, Clone)]
pub struct OptionalModDraft {
    pub name: String,
    pub description: String,
    pub category: String,
    pub enabled_by_default: bool,
    pub visible: bool,
    pub limited: bool,
    pub icon_url: Option<String>,
    pub author: Option<String>,
}

pub(super) fn optional_from_draft(
    draft: OptionalModDraft,
    path: String,
    icon_url: Option<String>,
    author: Option<String>,
) -> OptionalMod {
    OptionalMod {
        name: draft.name,
        description: draft.description,
        category: draft.category,
        files: vec![path],
        enabled_by_default: draft.enabled_by_default,
        visible: draft.visible,
        limited: draft.limited,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        triggers: Vec::new(),
        icon_url,
        author,
    }
}

pub(super) async fn append_optional_mod(
    state: &AppState,
    id: Uuid,
    new_mod: OptionalMod,
) -> AppResult<()> {
    let build = crate::db::get_build(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;
    let mut mods: Vec<OptionalMod> =
        serde_json::from_value(build.optional_mods).unwrap_or_default();
    mods.push(new_mod);
    let val = serde_json::to_value(&mods).map_err(|e| AppError::Other(e.into()))?;
    sqlx::query("UPDATE builds SET optional_mods = $2 WHERE id = $1")
        .bind(id)
        .bind(val)
        .execute(&state.db)
        .await?;
    Ok(())
}

pub async fn get_optional(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<OptionalMod>>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let build = crate::db::get_build(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;
    let mods: Vec<OptionalMod> = serde_json::from_value(build.optional_mods).unwrap_or_default();
    Ok(Json(mods))
}

/// Заменить весь список опциональных модов сборки.
pub async fn set_optional(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(mut mods): Json<Vec<OptionalMod>>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    fill_optional_icons(&state, &mut mods).await;
    let val = serde_json::to_value(&mods).map_err(|e| AppError::Other(e.into()))?;
    sqlx::query("UPDATE builds SET optional_mods = $2 WHERE id = $1")
        .bind(id)
        .bind(val)
        .execute(&state.db)
        .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct AllowSuggestionsReq {
    pub allow: bool,
}

pub async fn set_allow_suggestions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<AllowSuggestionsReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    sqlx::query("UPDATE builds SET allow_optional_mod_suggestions = $2 WHERE id = $1")
        .bind(id)
        .bind(req.allow)
        .execute(&state.db)
        .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

async fn fill_optional_icons(state: &AppState, mods: &mut [OptionalMod]) {
    for m in mods {
        if m.icon_url.as_deref().is_some_and(|u| !u.trim().is_empty()) {
            continue;
        }
        let url = format!(
            "https://api.modrinth.com/v2/search?query={}&facets={}&limit=1",
            urlencoding::encode(&m.name),
            urlencoding::encode(r#"[["project_type:mod"]]"#)
        );
        let result = state.http().get(&url).send().await;
        let Ok(resp) = result else {
            continue;
        };
        let parsed = resp.json::<Value>().await;
        let Ok(json) = parsed else {
            continue;
        };
        let Some(hit) = json["hits"].as_array().and_then(|hits| hits.first()) else {
            continue;
        };
        m.icon_url = hit["icon_url"].as_str().map(String::from);
        if m.author.is_none() {
            m.author = hit["author"].as_str().map(String::from);
        }
    }
}

#[derive(Deserialize)]
pub struct PathsReq {
    pub unmanaged_paths: Vec<String>,
    pub user_managed_paths: Vec<String>,
}

#[derive(Deserialize)]
pub struct FileContentQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct UpdateFileContentReq {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct RecommendedSettingsReq {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
}

pub async fn set_recommended_settings(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<RecommendedSettingsReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let min = req.memory_min_mb.clamp(512, 65536);
    let max = req.memory_max_mb.clamp(min, 65536);
    sqlx::query(
        "UPDATE builds SET recommended_memory_min_mb=$2,
         recommended_memory_max_mb=$3, recommended_jvm_flags=$4,
         recommended_show_console_on_launch=$5 WHERE id=$1",
    )
    .bind(id)
    .bind(min as i32)
    .bind(max as i32)
    .bind(req.jvm_flags)
    .bind(req.show_console_on_launch)
    .execute(&state.db)
    .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

pub async fn set_paths(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<PathsReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    sqlx::query("UPDATE builds SET unmanaged_paths=$2, user_managed_paths=$3 WHERE id=$1")
        .bind(id)
        .bind(serde_json::to_value(&req.unmanaged_paths).unwrap())
        .bind(serde_json::to_value(&req.user_managed_paths).unwrap())
        .execute(&state.db)
        .await?;
    let server_id = build_server_id(&state, id).await?;
    broadcast_builds_changed(&state, server_id);
    Ok(Json(json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct InstalledModItem {
    pub file_id: Uuid,
    pub path: String,
    pub sha1: String,
    pub size: i64,
    pub mod_id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

pub async fn list_installed_mods(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<InstalledModItem>>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let files = crate::db::build_files(&state.db, id).await?;
    let mut result = Vec::new();

    for f in files {
        if f.path.starts_with("mods/") || f.kind == "mod" {
            let path = state.files.path_for(&f.sha1);
            let meta = backend::mod_icon::extract_jar_metadata(&path);
            result.push(InstalledModItem {
                file_id: f.id,
                path: f.path,
                sha1: f.sha1,
                size: f.size,
                mod_id: meta.as_ref().and_then(|m| m.mod_id.clone()),
                name: meta.as_ref().and_then(|m| m.name.clone()),
                version: meta.as_ref().and_then(|m| m.version.clone()),
            });
        }
    }

    Ok(Json(result))
}
