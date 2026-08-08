//! noro-master — мастер-сервер: авторизация, Yggdrasil, раздача файлов,
//! WebSocket с лаунчерами, админ-API, bootstrap артефактов.

pub mod api;
pub mod build_importer;
pub mod config;
pub mod dav;
pub mod db;
pub mod error;
pub mod files;
pub mod launcher_builder;
pub mod manifest;
pub mod mojang_bootstrap;
pub mod signing;
pub mod state;
pub mod ws;

use anyhow::Result;
use axum::routing::{get, post, put};
use axum::Router;
use config::Config;
use state::AppState;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Точка входа сервера.
pub async fn run() -> Result<()> {
    let config = Config::from_env()?;
    tokio::fs::create_dir_all(&config.data_dir).await.ok();

    let signer = signing::Signer25519::from_config(&config.signing_key_hex)?;
    if config.is_dev_signing() {
        tracing::warn!(
            "DEV-режим подписи (seed из исходников). Публичный ключ: {}",
            signer.public_key_hex()
        );
    } else {
        tracing::info!("публичный ключ подписи: {}", signer.public_key_hex());
    }

    let db = db::connect_and_migrate(&config.database_url).await?;
    let files = files::FileStore::new(&config.data_dir);
    let http = reqwest::Client::builder()
        .user_agent("noro-master/0.1")
        .build()?;

    let state = AppState {
        db,
        ws: ws::WsHub::new(),
        files,
        signer,
        config: Arc::new(config.clone()),
        http,
        import_jobs: Arc::new(dashmap::DashMap::new()),
    };

    // Фоновый опрос GitHub (если настроен).
    launcher_builder::github_watcher::spawn(state.clone());

    let app = router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("noro-master слушает на {}", config.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    use api::{
        agent, agent_artifact, agent_nodes, auth, cabinet, file_serve, launcher, textures,
        translations,
    };

    // Yggdrasil (authlib-injector) — без авторизации.
    let yggdrasil = Router::new().nest(
        "/api/yggdrasil",
        Router::new()
            .route("/", get(auth::yggdrasil::root))
            .route(
                "/authserver/authenticate",
                post(auth::yggdrasil::authenticate),
            )
            .route("/authserver/refresh", post(auth::yggdrasil::refresh))
            .route("/authserver/validate", post(auth::yggdrasil::validate))
            .route("/authserver/invalidate", post(auth::yggdrasil::invalidate))
            // Пути как у Mojang, но с хостом вместо префикса: authlib-injector
            // отображает sessionserver.mojang.com в `{root}/sessionserver`,
            // сохраняя остаток пути. Без этого префикса join и hasJoined
            // просто не находятся.
            .route(
                "/sessionserver/session/minecraft/join",
                post(auth::yggdrasil::join),
            )
            .route(
                "/sessionserver/session/minecraft/hasJoined",
                get(auth::yggdrasil::has_joined),
            )
            .route(
                "/sessionserver/session/minecraft/profile/{uuid}",
                get(auth::yggdrasil::profile),
            )
            .route(
                "/api/profiles/minecraft",
                post(auth::yggdrasil::profiles_bulk),
            ),
    );

    // Discord OAuth.
    let discord = Router::new()
        .route("/auth/discord/login", get(auth::discord::login))
        .route("/auth/discord/callback", get(auth::discord::callback))
        .route("/auth/discord/launcher", get(auth::discord::launcher_login))
        .route(
            "/auth/discord/launcher/callback",
            get(auth::discord::launcher_callback),
        )
        .route(
            "/auth/launcher/exchange",
            post(auth::discord::launcher_exchange),
        )
        .route("/auth/refresh", post(auth::discord::refresh))
        .route("/auth/logout", get(auth::discord::logout))
        .route("/auth/me", get(cabinet::me));

    // Лаунчер.
    let launcher_api = Router::new()
        .route("/ws/launcher", get(launcher::ws_handler))
        .route("/api/launcher/version", get(launcher::current_version))
        .route("/api/launcher/downloads", get(launcher::downloads))
        .route("/files/{sha1}", get(file_serve::serve_file))
        .route("/api/textures/default-skin", get(textures::default_skin))
        .route("/api/launcher/locales", get(translations::list))
        .route("/api/launcher/locales/{locale}", get(translations::get));

    // Личный кабинет.
    let cabinet_api = Router::new()
        .route("/api/me", get(cabinet::me))
        .route("/api/me/username", put(cabinet::set_username))
        .route(
            "/api/me/skin",
            post(cabinet::upload_skin).delete(cabinet::delete_skin),
        );

    // Агенты игровых серверов.
    let agent_api = Router::new()
        .route("/api/agent/players/{mc_uuid}", get(agent::player))
        .route("/api/agent/heartbeat", post(agent::heartbeat))
        .route("/api/agent/artifact", get(agent_artifact::artifact))
        .route("/api/agent/pubkey", get(agent_artifact::pubkey))
        .route("/api/agent/nodes", post(agent_nodes::report));

    let admin_api = api::admin::router().route(
        "/api/admin/locales/{locale}",
        put(translations::put).delete(translations::delete),
    );

    Router::new()
        .merge(yggdrasil)
        .merge(discord)
        .merge(launcher_api)
        .merge(cabinet_api)
        .merge(agent_api)
        .merge(admin_api)
        .layer(axum::extract::DefaultBodyLimit::disable())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        // WebDAV подключается ПОСЛЕ слоёв: CorsLayer сам отвечает на OPTIONS,
        // а Finder ждёт от него заголовок `DAV` — без него том не монтируется.
        .merge(
            dav::router()
                .layer(axum::extract::DefaultBodyLimit::disable())
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state)
}
