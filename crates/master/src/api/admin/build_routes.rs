use crate::state::AppState;
use axum::routing::{delete, get, post, put};
use axum::Router;

use super::builds;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/builds", get(builds::list).post(builds::create))
        .route(
            "/api/admin/builds/{id}",
            get(builds::get).delete(builds::delete),
        )
        .route("/api/admin/builds/{id}/duplicate", post(builds::duplicate))
        .route("/api/admin/builds/{id}/publish", post(builds::publish))
        .route("/api/admin/builds/{id}/rebuild", post(builds::rebuild))
        .route(
            "/api/admin/builds/{id}/rebuild-clean",
            post(builds::rebuild_clean),
        )
        .route("/api/admin/builds/{id}/unpublish", post(builds::unpublish))
        .route("/api/admin/builds/{id}/paths", put(builds::set_paths))
        .route("/api/admin/builds/{id}/versions", put(builds::set_versions))
        .route(
            "/api/admin/builds/{id}/recommended-settings",
            put(builds::set_recommended_settings),
        )
        .route(
            "/api/admin/builds/{id}/import/mrpack",
            post(builds::import_mrpack),
        )
        .route(
            "/api/admin/builds/{id}/import/curseforge",
            post(builds::import_curseforge),
        )
        .route(
            "/api/admin/builds/{id}/import/zip",
            post(builds::import_instance_zip),
        )
        .route(
            "/api/admin/builds/{id}/import-progress/{job_id}",
            get(builds::import_progress),
        )
        .route(
            "/api/admin/builds/{id}/files",
            get(builds::list_files).post(builds::upload_file),
        )
        .route(
            "/api/admin/builds/{id}/files/{file_id}",
            delete(builds::delete_file),
        )
        .route(
            "/api/admin/builds/{id}/files/content",
            get(builds::get_file_content).put(builds::update_file_content),
        )
        .route(
            "/api/admin/builds/{id}/files/icon",
            get(builds::get_file_icon),
        )
        .route(
            "/api/admin/builds/{id}/files/prefix",
            delete(builds::delete_files_by_prefix),
        )
        .route(
            "/api/admin/builds/{id}/files/move",
            post(builds::move_files),
        )
        .route(
            "/api/admin/builds/{id}/optional-mods",
            get(builds::get_optional).put(builds::set_optional),
        )
        .route(
            "/api/admin/builds/{id}/allow-suggestions",
            put(builds::set_allow_suggestions),
        )
        .route(
            "/api/admin/builds/{id}/installed-mods",
            get(builds::list_installed_mods),
        )
}
