use crate::state::AppState;
use axum::routing::{delete, get, post, put};
use axum::Router;

use super::{
    agents, audit, auth_methods, backup, backup_restore, blocklist, build_routes, capes, cases,
    catalog, chat_filters, cores, freezes_and_reports, game_actions, game_servers, impersonate,
    integrity, launcher, launcher_clients, log_requests, mod_install, mod_suggestions,
    moderation_messages, news, notes, oauth_apps, optional_upload, permission_nodes,
    prefix_preview, prefix_sync, punishments, remote, restarts, role_badge, roles, rules, servers,
    settings, stats, storage, tokens, user_launcher, users, versions, wrapper, wrapper_backups,
    wrapper_fs,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/freezes", post(freezes_and_reports::freeze))
        .route(
            "/api/admin/freezes/{user_id}",
            delete(freezes_and_reports::unfreeze),
        )
        .route("/api/admin/reports", get(freezes_and_reports::list_reports))
        .route(
            "/api/admin/reports/{id}/claim",
            post(freezes_and_reports::claim_report),
        )
        .route(
            "/api/admin/reports/{id}/resolve",
            put(freezes_and_reports::resolve_report),
        )
        .route("/api/admin/cases", get(cases::list))
        // Строго до `/{id}`: иначе `dossier` уедет в него как uuid и не разберётся.
        .route("/api/admin/cases/dossier", get(cases::dossier))
        .route("/api/admin/cases/{id}", get(cases::get))
        .route("/api/admin/cases/{id}/quote", post(cases::quote))
        .route("/api/admin/cases/{id}/claim", post(cases::claim))
        .route("/api/admin/cases/{id}/release", post(cases::release))
        .route("/api/admin/cases/{id}/resolve", put(cases::resolve))
        .route("/api/admin/cases/{id}/notes", post(cases::note))
        .route("/api/admin/cases/{id}/attachments", post(cases::upload))
        .route(
            "/api/admin/cases/{id}/chat-request",
            post(cases::request_chat),
        )
        .route(
            "/api/admin/cases/{id}/inventory-request",
            post(cases::request_inventory),
        )
        .route(
            "/api/admin/cases/{id}/client-check",
            post(cases::request_client_check),
        )
        .route(
            "/api/admin/restarts",
            get(restarts::list).post(restarts::create),
        )
        .route("/api/admin/restarts/{id}", delete(restarts::delete))
        .route("/api/admin/game/kick", post(game_actions::kick))
        .route("/api/admin/game/tell", post(game_actions::tell))
        .route("/api/admin/game/announce", post(game_actions::announce))
        .route("/api/admin/agents", get(agents::list))
        .route("/api/admin/audit", get(audit::list))
        .route("/api/admin/audit/actions", get(audit::actions))
        .route(
            "/api/admin/rules",
            get(rules::list_rules).post(rules::create_rule),
        )
        .route("/api/admin/rules/reorder", put(rules::reorder_rules))
        .route(
            "/api/admin/rules/categories",
            get(rules::list_categories).post(rules::create_category),
        )
        .route(
            "/api/admin/rules/categories/reorder",
            put(rules::reorder_categories),
        )
        .route(
            "/api/admin/rules/categories/{id}",
            put(rules::update_category).delete(rules::delete_category),
        )
        .route(
            "/api/admin/rules/categories/{id}/translations",
            get(rules::category_translations),
        )
        .route(
            "/api/admin/rules/{id}",
            put(rules::update_rule).delete(rules::delete_rule),
        )
        .route(
            "/api/admin/rules/{id}/translations",
            get(rules::rule_translations),
        )
        .route(
            "/api/admin/blocklist",
            get(blocklist::list).post(blocklist::create),
        )
        .route("/api/admin/blocklist/{id}", delete(blocklist::delete))
        .route(
            "/api/admin/settings",
            get(settings::list).put(settings::save),
        )
        .route("/api/admin/settings/env", get(settings::export_env))
        .route("/api/admin/auth-methods", get(auth_methods::list))
        .route(
            "/api/admin/auth-methods/{method}",
            put(auth_methods::save).post(auth_methods::save),
        )
        .route(
            "/api/admin/settings/image/{key}",
            post(settings::upload_image),
        )
        .route("/api/admin/diagnostics", get(settings::diagnostics))
        // OAuth2-приложения: очередь модерации, выдача scope'ов, выключатели.
        .route("/api/admin/oauth-apps", get(oauth_apps::list))
        .route(
            "/api/admin/oauth-apps/settings",
            put(oauth_apps::set_toggles),
        )
        .route("/api/admin/oauth-apps/{id}", delete(oauth_apps::delete))
        .route(
            "/api/admin/oauth-apps/{id}/status",
            put(oauth_apps::set_status),
        )
        .route(
            "/api/admin/oauth-apps/{id}/scopes",
            put(oauth_apps::set_scopes),
        )
        .route(
            "/api/admin/oauth-apps/{id}/icon",
            post(oauth_apps::upload_icon),
        )
        .route(
            "/api/admin/builds/{id}/optional-mods",
            post(optional_upload::upload),
        )
        .route(
            "/api/admin/moderation/messages",
            get(moderation_messages::get).put(moderation_messages::put),
        )
        .route(
            "/api/admin/users/{id}/impersonate",
            post(impersonate::start),
        )
        .route(
            "/api/admin/impersonate/{grant_id}",
            get(impersonate::status),
        )
        .route("/api/admin/step-up", get(impersonate::step_up::status))
        .route(
            "/api/admin/step-up/passkey",
            post(impersonate::step_up::confirm_passkey),
        )
        .route(
            "/api/admin/step-up/recovery",
            post(impersonate::step_up::confirm_recovery),
        )
        .route("/api/admin/integrity", get(integrity::list))
        .route("/api/admin/support/bundles", get(crate::api::support::list))
        .route("/api/admin/support/requests", get(log_requests::list))
        .route(
            "/api/admin/support/requests/{id}",
            delete(log_requests::cancel),
        )
        .route(
            "/api/admin/users/{id}/request-logs",
            post(log_requests::request),
        )
        .route(
            "/api/admin/users/{id}/diagnostics",
            get(remote::diagnostics).post(remote::request_diagnostics),
        )
        .route("/api/admin/users/{id}/action", post(remote::run_action))
        .route(
            "/api/admin/users/{id}/punishments",
            get(punishments::list).post(punishments::create),
        )
        .route(
            "/api/admin/users/{id}/punishments/{punishment_id}",
            delete(punishments::revoke),
        )
        .route(
            "/api/admin/users/{id}/notes",
            get(notes::list).post(notes::add),
        )
        .route(
            "/api/admin/users/{id}/notes/{note_id}",
            delete(notes::delete),
        )
        .route(
            "/api/admin/users/{id}/play-sessions",
            get(notes::play_sessions),
        )
        .route(
            "/api/admin/support/bundles/{id}",
            get(crate::api::support::download),
        )
        .route("/api/admin/integrity/{id}/review", post(integrity::review))
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
            "/api/mod-suggestions",
            post(mod_suggestions::create_suggestion),
        )
        .route(
            "/api/admin/mod-suggestions",
            get(mod_suggestions::list_suggestions),
        )
        .route(
            "/api/admin/mod-suggestions/{id}/approve",
            post(mod_suggestions::approve_suggestion),
        )
        .route(
            "/api/admin/mod-suggestions/{id}/reject",
            post(mod_suggestions::reject_suggestion),
        )
        .route(
            "/api/admin/mod-suggestions/{id}/accept",
            post(mod_suggestions::accept_suggestion),
        )
}

fn users_router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/users", get(users::list))
        .route("/api/admin/users/by-identity", get(users::get_by_identity))
        .route("/api/admin/users/{id}", get(users::get))
        .route(
            "/api/admin/users/{id}/launcher",
            get(user_launcher::launcher_status),
        )
        .route(
            "/api/admin/users/{id}/sessions",
            get(user_launcher::sessions).delete(user_launcher::revoke_sessions),
        )
        .route(
            "/api/admin/users/{id}/sessions/{session_id}",
            delete(user_launcher::revoke_session),
        )
        .route("/api/admin/users/{id}/ban", put(users::ban))
        .route(
            "/api/admin/users/{id}/identities/{provider}",
            delete(users::unlink_identity),
        )
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
            "/api/admin/users/{id}/skin-presets",
            get(users::list_skin_presets_for_user).post(users::add_skin_preset_for_user),
        )
        .route(
            "/api/admin/users/{id}/skin-presets/select",
            post(users::select_skin_preset_for_user),
        )
        .route(
            "/api/admin/users/{id}/skin-presets/{preset_id}",
            delete(users::delete_skin_preset_for_user),
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
        .route("/api/admin/prefix-badge", get(prefix_preview::badge))
        .route("/api/admin/roles/sync-badges", post(prefix_sync::sync))
        .route(
            "/api/admin/roles/{id}/badge",
            put(role_badge::upload).delete(role_badge::clear),
        )
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
            get(news::get_one).put(news::update).delete(news::delete),
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
            "/api/admin/servers/{id}/game-servers/maintenance",
            put(game_servers::bulk_maintenance),
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
            "/api/admin/chat-filters",
            get(chat_filters::list).put(chat_filters::save),
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
        // Дамп базы, полный архив и восстановление из него. Разбор отделён от
        // применения: сперва показать, что в архиве, и только потом спрашивать
        // подтверждение — восстановление необратимо.
        .route("/api/admin/backup", get(backup::download))
        .route("/api/admin/backup/full", get(backup::full))
        // Скачивание из браузера: навигация не носит Authorization, а тянуть
        // гигабайты через fetch значит держать их в памяти вкладки.
        .route("/api/admin/backup/ticket", post(backup::ticket))
        .route(
            "/api/admin/backup/full/{ticket}",
            get(backup::full_by_ticket),
        )
        .route("/api/admin/backup/inspect", post(backup_restore::inspect))
        .route("/api/admin/backup/restore", post(backup_restore::restore))
        .route(
            "/api/admin/launcher/clients",
            get(launcher_clients::clients),
        )
        // Уборка хранилища: GET считает, DELETE удаляет.
        .route(
            "/api/admin/storage/orphans",
            get(storage::scan_orphans).delete(storage::delete_orphans),
        )
        .route("/api/admin/versions/minecraft", get(versions::minecraft))
        .route("/api/admin/versions/loader/{kind}", get(versions::loader))
}
