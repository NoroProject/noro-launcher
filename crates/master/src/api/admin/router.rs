use crate::state::AppState;
use axum::routing::{delete, get, post, put};
use axum::Router;

use super::{
    agents, build_routes, capes, catalog, cores, game_servers, launcher, mod_install,
    mod_suggestions, news, permission_nodes, roles, servers, stats, tokens, users, versions,
    wrapper, wrapper_backups, wrapper_fs,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/agents", get(agents::list))
        .route("/api/admin/permission-nodes", get(permission_nodes::list))
        .merge(catalog_router())
        .merge(users_router())
        .merge(roles_router())
        .merge(content_router())
        .merge(servers_router())
        .merge(build_routes::router())
        .merge(system_router())
}

/// Каталог модов и установка. Не под `/builds/{id}`: тот же поиск нужен и без
/// сборки — в глобальном браузере и при заливке мода прямо на игровой сервер.
fn catalog_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/catalog/search", get(catalog::search))
        .route("/api/admin/catalog/categories", get(catalog::categories))
        .route("/api/admin/catalog/providers", get(catalog::providers))
        .route(
            "/api/admin/catalog/{provider}/project/{id}",
            get(catalog::project),
        )
        .route(
            "/api/admin/catalog/{provider}/project/{id}/versions",
            get(catalog::versions),
        )
        .route("/api/admin/mods/install", post(mod_install::install))
        .route(
            "/api/mod_suggestions",
            post(mod_suggestions::create_suggestion),
        )
        .route(
            "/api/admin/mod_suggestions",
            get(mod_suggestions::list_suggestions),
        )
        .route(
            "/api/admin/mod_suggestions/{id}/approve",
            post(mod_suggestions::approve_suggestion),
        )
        .route(
            "/api/admin/mod_suggestions/{id}/reject",
            post(mod_suggestions::reject_suggestion),
        )
        .route(
            "/api/admin/mod_suggestions/{id}/accept",
            post(mod_suggestions::accept_suggestion),
        )
}

fn users_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/users", get(users::list))
        .route("/api/admin/users/{id}", get(users::get))
        .route("/api/admin/users/{id}/ban", put(users::ban))
        .route("/api/admin/users/{id}/cape", put(users::set_cape))
        .route(
            "/api/admin/users/{id}/capes",
            get(users::get_capes).put(users::set_granted_capes),
        )
        .route(
            "/api/admin/users/{id}/skin",
            post(users::upload_skin_for_user).delete(users::delete_skin_for_user),
        )
        .route(
            "/api/admin/users/{id}/roles/{role_id}",
            post(users::add_role).delete(users::remove_role),
        )
        .route(
            "/api/admin/users/{id}/permissions",
            post(users::add_permission),
        )
        .route(
            "/api/admin/users/{id}/permissions/{perm}",
            delete(users::remove_permission),
        )
}

fn roles_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/roles", get(roles::list).post(roles::create))
        .route(
            "/api/admin/roles/{id}",
            put(roles::update).delete(roles::delete),
        )
        .route(
            "/api/admin/roles/{id}/permissions",
            post(roles::add_permission),
        )
        .route(
            "/api/admin/roles/{id}/permissions/{perm}",
            delete(roles::remove_permission),
        )
}

fn content_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/capes", get(capes::list).post(capes::upload))
        .route("/api/admin/capes/{id}", delete(capes::delete))
        .route("/api/admin/cores", get(cores::list).post(cores::upload))
        .route("/api/admin/cores/{id}/activate", post(cores::activate))
        .route("/api/admin/cores/{id}", delete(cores::delete))
        .route("/api/admin/news", get(news::list).post(news::create))
        .route("/api/admin/news/image", post(news::upload_image))
        .route(
            "/api/admin/news/{id}",
            put(news::update).delete(news::delete),
        )
}

fn servers_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/servers",
            get(servers::list).post(servers::create),
        )
        .route("/api/admin/servers/reorder", put(servers::reorder))
        .route(
            "/api/admin/servers/{id}",
            put(servers::update).delete(servers::delete),
        )
        .route("/api/admin/servers/{id}/icon", put(servers::upload_icon))
        .route(
            "/api/admin/servers/{id}/background",
            put(servers::upload_background),
        )
        .route(
            "/api/admin/servers/{id}/game-servers",
            get(game_servers::list).post(game_servers::create),
        )
        .route(
            "/api/admin/servers/{id}/game-servers/{gs_id}",
            put(game_servers::update).delete(game_servers::delete),
        )
        .route(
            "/api/admin/servers/{id}/game-servers/{gs_id}/token",
            post(game_servers::rotate_token),
        )
        .merge(wrapper_router())
}

/// Управление игровой машиной. Адресуется id игрового сервера, без сервера-пака
/// в пути: враппер отвечает за конкретный инстанс, и знать про пак ему незачем.
fn wrapper_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/game-servers/{id}/wrapper", get(wrapper::status))
        .route(
            "/api/admin/game-servers/{id}/wrapper/power",
            post(wrapper::power),
        )
        .route(
            "/api/admin/game-servers/{id}/wrapper/command",
            post(wrapper::command),
        )
        .route(
            "/api/admin/game-servers/{id}/wrapper/console",
            get(wrapper::console),
        )
        .route(
            "/api/admin/game-servers/{id}/wrapper/console/stream",
            get(wrapper::console_stream),
        )
        .route(
            "/api/admin/game-servers/{id}/fs",
            get(wrapper_fs::list).delete(wrapper_fs::delete),
        )
        .route(
            "/api/admin/game-servers/{id}/fs/file",
            get(wrapper_fs::read).put(wrapper_fs::write),
        )
        .route(
            "/api/admin/game-servers/{id}/fs/mkdir",
            post(wrapper_fs::mkdir),
        )
        .route(
            "/api/admin/game-servers/{id}/fs/apply",
            post(wrapper_fs::apply),
        )
        .route(
            "/api/admin/game-servers/{id}/backups",
            get(wrapper_backups::list).post(wrapper_backups::create),
        )
        .route(
            "/api/admin/game-servers/{id}/backups/{name}",
            delete(wrapper_backups::delete),
        )
        .route(
            "/api/admin/game-servers/{id}/backups/{name}/restore",
            post(wrapper_backups::restore),
        )
}

fn system_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/launcher/versions", get(launcher::list_versions))
        .route("/api/admin/launcher/github", get(launcher::github_latest))
        .route("/api/admin/launcher/build", post(launcher::build))
        .route("/api/admin/launcher/builds", get(launcher::build_jobs))
        .route(
            "/api/admin/launcher/build/{job_id}/log",
            get(launcher::build_log),
        )
        .route(
            "/api/admin/launcher/deploy/{version_id}",
            post(launcher::deploy),
        )
        .route("/api/admin/tokens", get(tokens::list).post(tokens::create))
        .route("/api/admin/tokens/{id}", delete(tokens::delete))
        .route("/api/admin/stats", get(stats::stats))
        .route("/api/admin/versions/minecraft", get(versions::minecraft))
        .route("/api/admin/versions/loader/{kind}", get(versions::loader))
}
