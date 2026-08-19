//! noro-master — мастер-сервер: авторизация, Yggdrasil, раздача файлов,
//! WebSocket с лаунчерами, админ-API, bootstrap артефактов.

pub mod agent_link;
pub mod api;
pub mod audit;
pub mod build_importer;
pub mod catalog;
pub mod config;
pub mod dav;
pub mod db;
pub mod error;
pub mod files;
pub mod launcher_builder;
pub mod manifest;
pub mod mojang_bootstrap;
pub mod setup;
pub mod signing;
pub mod state;
pub mod telemetry;
pub mod wrapper;
pub mod ws;
pub mod yggdrasil_sign;

use anyhow::Result;
use axum::routing::{delete, get, post, put};
use axum::Router;
use config::Config;
use state::AppState;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Точка входа сервера.
pub async fn run() -> Result<()> {
    // До подъёма БД мастер знает только это: где слушать, куда подключаться и
    // где лежат данные. Всё остальное читается уже из БД.
    let boot = config::Bootstrap::from_env()?;
    tokio::fs::create_dir_all(&boot.data_dir).await.ok();

    let db = db::connect_and_migrate(&boot.database_url).await?;

    // Боевой инстанс перехода не замечает: его настройки уже заданы в
    // окружении, и спрашивать его о них заново незачем.
    setup::migrate_env_if_needed(&db).await?;

    let config = Config::load(boot, &db).await?;
    announce(&config, &db).await?;

    let signer = signing::Signer25519::from_config(&config.signing_key_hex)?;
    if config.is_dev_signing() {
        tracing::warn!(
            "DEV-режим подписи (seed из исходников). Публичный ключ: {}",
            signer.public_key_hex()
        );
    } else {
        tracing::info!("публичный ключ подписи: {}", signer.public_key_hex());
    }

    // Ключ создаётся при первом старте и живёт в data_dir рядом с файлами.
    let profile_signer = yggdrasil_sign::ProfileSigner::load_or_create(&config.data_dir)?;

    let files = files::FileStore::new(&config.data_dir);
    let http = reqwest::Client::builder()
        .user_agent("noro-master/0.1")
        .build()?;

    let state = AppState {
        db,
        ws: ws::WsHub::new(),
        files,
        signer,
        profile_signer: Arc::new(profile_signer),
        config: Arc::new(config.clone()),
        http,
        import_jobs: Arc::new(dashmap::DashMap::new()),
        catalog: catalog::HttpCache::default(),
        wrappers: wrapper::WrapperHub::default(),
        agents: agent_link::AgentHub::default(),
        roster: agent_link::Roster::default(),
        // Без публичных адресов домен для passkey не вывести. Это не повод не
        // подняться: инстанс как раз и поднимается, чтобы их задать.
        webauthn: build_webauthn(&config),
    };

    // Фоновый опрос GitHub (если настроен).
    launcher_builder::github_watcher::spawn(state.clone());
    db::cleanup::spawn(state.db.clone());
    db::restart_schedules::spawn_restart_scheduler(state.clone());

    let app = router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("noro-master слушает на {}", config.bind_addr);
    // ConnectInfo нужен ограничителю частоты: без прокси-заголовка адрес
    // клиента берётся из соединения.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

fn build_webauthn(config: &Config) -> Option<Arc<webauthn_rs::Webauthn>> {
    if !config.is_usable() {
        return None;
    }
    match api::auth::webauthn::build(config) {
        Ok(w) => Some(Arc::new(w)),
        // Не валим старт: без passkey остаётся вход через Discord, а вот
        // мастер, не поднявшийся из-за опечатки в домене, не оставляет ничего.
        Err(e) => {
            tracing::error!(error = %e, "passkey отключён: не собрать WebAuthn");
            None
        }
    }
}

/// Сказать при старте, в каком состоянии инстанс.
///
/// Ненастроенный мастер молчал бы и отвечал 503 на всё подряд — оператор
/// увидел бы сломанный сайт вместо инструкции.
async fn announce(config: &Config, db: &sqlx::PgPool) -> Result<()> {
    if let Some(token) = setup::ensure_token(db, &config.data_dir).await? {
        tracing::warn!(
            "инстанс не настроен. Откройте {}/setup и введите токен:\n\n    {token}\n\n\
             он же лежит в {}",
            if config.web_url.is_empty() {
                "http://<адрес сайта>"
            } else {
                &config.web_url
            },
            config.data_dir.join("setup-token.txt").display(),
        );
    }
    Ok(())
}

fn router(state: AppState) -> Router {
    use api::{
        agent, agent_artifact, agent_nodes, auth, cabinet, cabinet_locale, cabinet_online,
        cabinet_sessions, cabinet_skin_model, file_serve, launcher, server_online, support,
        textures, translations,
    };

    // Yggdrasil (authlib-injector) — без авторизации.
    let yggdrasil = Router::new()
        // Корень ALI обязан отвечать и со слэшем: лаунчер передаёт агенту
        // `{master}/api/yggdrasil`, а authlib-injector дописывает `/` перед тем,
        // как забрать метаданные. Вложенный роутер отдавал на такой адрес 404 —
        // агент оставался без `skinDomains`, клиент подставлял дефолтный список
        // Mojang и молча отбрасывал все текстуры с нашего CDN.
        .route("/api/yggdrasil/", get(auth::yggdrasil::root))
        .nest(
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

    // Общий счётчик на все эндпоинты входа: окно считается по IP, а не по
    // маршруту, иначе перебор просто чередовал бы ручки.
    let limiter = api::rate_limit::RateLimiter::new();

    // Discord OAuth & Passkeys.
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
        .route(
            "/auth/passkeys/login/options",
            post(auth::passkeys::login_options).options(|| async {}),
        )
        .route(
            "/auth/passkeys/login/verify",
            post(auth::passkeys::login_verify).options(|| async {}),
        )
        .route(
            "/api/auth/passkeys/login/options",
            post(auth::passkeys::login_options).options(|| async {}),
        )
        .route(
            "/api/auth/passkeys/login/verify",
            post(auth::passkeys::login_verify).options(|| async {}),
        )
        .route(
            "/api/auth/recovery/login",
            post(auth::recovery::login).options(|| async {}),
        )
        .route("/auth/refresh", post(auth::discord::refresh))
        .route("/auth/logout", get(auth::discord::logout))
        .route("/auth/me", get(cabinet::me))
        .layer(axum::middleware::from_fn_with_state(
            limiter.clone(),
            api::rate_limit::limit,
        ));

    // Лаунчер.
    let launcher_api = Router::new()
        .route("/ws/launcher", get(launcher::ws_handler))
        .route("/api/launcher/version", get(launcher::current_version))
        .route("/api/launcher/downloads", get(launcher::downloads))
        // Инициатива игрока: кнопка «Сообщить о проблеме» и предложение после
        // краша. Ни запроса от админа, ни гранта здесь не нужно.
        .route("/api/launcher/support-bundle", post(support::upload))
        // Обменять подтверждённый грант на сессию. Ходит сам лаунчер своим
        // текущим Bearer — токен не идёт через браузер и не светится в `ps`.
        .route(
            "/api/launcher/impersonate/claim",
            post(api::admin::impersonate::claim),
        )
        .route("/files/{sha1}", get(file_serve::serve_file))
        .route("/api/textures/default-skin", get(textures::default_skin))
        .route(
            "/api/textures/presets/{name}",
            get(textures::preset_skin_endpoint),
        )
        .route("/api/textures/renders", get(textures::render_endpoint))
        .route(
            "/api/textures/renders/head",
            get(textures::render_head_endpoint),
        )
        .route(
            "/api/textures/renders/bust",
            get(textures::render_bust_endpoint),
        )
        .route(
            "/api/textures/renders/body",
            get(textures::render_body_endpoint),
        )
        .route(
            "/api/textures/renders/cape",
            get(textures::render_cape_endpoint),
        )
        .route("/api/launcher/locales", get(translations::list))
        .route("/api/launcher/locales/{locale}", get(translations::get))
        // Свод правил открыт всем: на него ссылается каждый бан, и забаненный
        // должен иметь возможность прочитать, за что именно.
        // Витрина проекта: список серверов с онлайном, как на любом сайте
        // модового проекта.
        .route("/api/servers", get(api::servers::list))
        .route(
            "/api/public/settings",
            get(api::public_settings::get_public_settings),
        )
        .route("/api/rules", get(api::rules::list))
        .route("/api/rules/scopes", get(api::rules::scopes))
        .route("/api/rules/servers/{server_id}", get(api::rules::by_server))
        .route("/api/servers/{server_id}/online", get(server_online::get_online));

    // Личный кабинет.
    let cabinet_api = Router::new()
        .route("/api/me", get(cabinet::me))
        .route("/api/me/username", put(cabinet::set_username))
        .route("/api/me/locale", put(cabinet_locale::set_locale))
        .route("/api/me/skin/model", put(cabinet_skin_model::set_model))
        .route("/api/me/hide-from-online", put(cabinet_online::set_hide_from_online))
        .route("/api/me/silent-join", put(cabinet_online::set_silent_join))
        .route("/api/me/punishments", get(cabinet::punishments))
        .route("/api/me/activity-heatmap", get(cabinet_sessions::activity_heatmap))
        .route("/api/me/sessions", get(cabinet_sessions::list))
        .route(
            "/api/me/sessions/others",
            delete(cabinet_sessions::revoke_others),
        )
        .route("/api/me/sessions/{id}", delete(cabinet_sessions::revoke))
        .route("/api/me/recovery-codes", get(auth::recovery::remaining))
        .route(
            "/api/me/recovery-codes/reissue",
            post(auth::recovery::reissue),
        )
        .route("/api/me/support-bundles", get(support::my_bundles))
        .route("/api/me/support-bundles/{id}", delete(support::delete_mine))
        .route(
            "/api/me/skin",
            post(cabinet::upload_skin).delete(cabinet::delete_skin),
        )
        .route(
            "/api/me/skin/from-username",
            post(cabinet::upload_skin_from_username),
        )
        .route(
            "/api/me/passkeys/register/options",
            post(auth::passkeys::register_options).options(|| async {}),
        )
        .route(
            "/api/me/passkeys/register/verify",
            post(auth::passkeys::register_verify).options(|| async {}),
        )
        .route(
            "/api/me/passkeys",
            get(auth::passkeys::list_passkeys).options(|| async {}),
        )
        .route(
            "/api/me/passkeys/{id}",
            delete(auth::passkeys::delete_passkey).options(|| async {}),
        )
        .route("/api/capes", get(cabinet::list_capes))
        .route("/api/me/cape", put(cabinet::set_cape))
        .route(
            "/api/me/skin-presets",
            get(cabinet::list_skin_presets)
                .post(cabinet::create_skin_preset)
                .options(|| async {}),
        )
        .route(
            "/api/me/skin-presets/{id}",
            put(cabinet::rename_skin_preset)
                .delete(cabinet::delete_skin_preset)
                .options(|| async {}),
        )
        .route(
            "/api/me/authorized-apps",
            get(auth::oauth2_provider::list_authorized_apps).options(|| async {}),
        )
        .route(
            "/api/me/authorized-apps/{app_id}",
            delete(auth::oauth2_provider::revoke_authorized_app).options(|| async {}),
        );

    // Полноценный OAuth 2.0 Провайдер
    let oauth2_provider_api = Router::new()
        .route(
            "/oauth2/authorize",
            get(auth::oauth2_provider::authorize_page),
        )
        .route(
            "/oauth2/authorize/accept",
            post(auth::oauth2_provider::accept_authorize),
        )
        .route("/oauth2/token", post(auth::oauth2_provider::token_endpoint))
        .layer(axum::middleware::from_fn_with_state(
            limiter.clone(),
            api::rate_limit::limit,
        ));

    // Агенты игровых серверов.
    let agent_api = Router::new()
        .route("/api/agent/players/{mc_uuid}", get(agent::player))
        .route("/api/agent/players/batch", post(agent::players_batch))
        .route(
            "/api/agent/players/by-name/{username}",
            get(agent::player_by_name),
        )
        .route(
            "/api/agent/players/{mc_uuid}/punishments",
            get(agent::list_punishments).post(agent::create_punishment),
        )
        .route(
            "/api/agent/punishments/{id}/revoke",
            post(agent::revoke_punishment),
        )
        .route("/api/agent/punishments/{id}/ack", post(agent::acknowledge))
        .route(
            "/api/agent/messages",
            get(api::moderation_messages::agent_messages),
        )
        .route("/api/agent/rules", get(api::rules::agent_list))
        .route("/api/agent/rules/{code}", get(api::rules::agent_by_code))
        .route("/api/agent/heartbeat", post(agent::heartbeat))
        .route("/api/agent/chat-filters", get(agent::list_chat_filters))
        .route("/api/agent/automod-triggers", post(agent::record_automod_trigger))
        .route("/api/agent/reports", post(agent::agent_create_report))
        .route("/api/agent/artifact", get(agent_artifact::artifact))
        .route("/api/agent/pubkey", get(agent_artifact::pubkey))
        .route("/api/agent/nodes", post(agent_nodes::report))
        // Канал управления враппером. Живёт рядом с остальным агентским API:
        // авторизация та же — секрет игрового сервера.
        .route("/api/agent/ws", get(wrapper::session::ws_handler))
        // Канал наказаний до агента внутри игры. Отдельный от враппера: тот
        // управляет процессом снаружи и есть не везде, а бан обязан долетать
        // до игрока в тот же момент, а не к его следующему входу.
        .route("/api/agent/link", get(agent_link::session::ws_handler));

    let admin_api = api::admin::router().route(
        "/api/admin/locales/{locale}",
        put(translations::put).delete(translations::delete),
    );

    let cors = cors_layer(&state.config);

    // Публичное и пользовательское API: тело ограничено. Раньше лимит был снят
    // на всём роутере разом, и аноним мог занять память запросом любого размера
    // на любом эндпоинте. Самая крупная загрузка здесь — скин на 256 КБ.
    // Визард. Живёт до завершения настройки: после неё `SetupAuth` перестаёт
    // принимать что-либо, потому что токен сожжён.
    let setup_api = Router::new()
        .route("/api/setup/status", get(setup::api::status))
        .route("/api/setup/settings", post(setup::api::save))
        .route(
            "/api/setup/signing-key",
            post(setup::api::generate_signing_key),
        )
        .route("/api/setup/env", get(setup::api::env_block))
        .route("/api/setup/root", post(setup::api::create_root))
        .route("/api/setup/complete", post(setup::api::complete));

    let public_api = Router::new()
        .route("/health", get(api::health::health))
        .merge(setup_api)
        .merge(yggdrasil)
        .merge(discord)
        .merge(launcher_api)
        .merge(cabinet_api)
        .merge(oauth2_provider_api)
        .merge(agent_api)
        .layer(axum::extract::DefaultBodyLimit::max(PUBLIC_BODY_LIMIT));

    Router::new()
        .merge(public_api)
        // Админка заливает сборки, моды и бинарники лаунчера — тут лимит снят
        // осознанно, и маршруты закрыты проверкой прав.
        .merge(admin_api.layer(axum::extract::DefaultBodyLimit::disable()))
        // Заслонка идёт до всего остального: ненастроенный мастер не должен
        // делать вид, что работает, — манифесты уехали бы со ссылками в никуда.
        // Детальный аудит под impersonation. Идёт первым слоем: под чужим
        // аккаунтом каждое изменение должно оставить след, иначе спор
        // «меня обокрали» / «это был не я» разрешать будет нечем.
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            audit::impersonation::layer,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            setup::gate::gate,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // WebDAV подключается ПОСЛЕ слоёв: CorsLayer сам отвечает на OPTIONS,
        // а Finder ждёт от него заголовок `DAV` — без него том не монтируется.
        .merge(
            dav::router()
                .layer(axum::extract::DefaultBodyLimit::disable())
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state)
}

/// Потолок тела запроса для публичного API.
const PUBLIC_BODY_LIMIT: usize = 8 * 1024 * 1024;

/// CORS по списку origin'ов из конфига.
///
/// Без `NORO_ALLOWED_ORIGINS` остаётся permissive — иначе локальная разработка
/// перестала бы работать молча. В проде переменную нужно задать.
fn cors_layer(config: &Config) -> CorsLayer {
    if config.allowed_origins.is_empty() {
        tracing::warn!("NORO_ALLOWED_ORIGINS не задан — CORS открыт для любого origin");
        return CorsLayer::permissive();
    }

    let origins: Vec<_> = config
        .allowed_origins
        .iter()
        .filter_map(|o| match o.parse::<axum::http::HeaderValue>() {
            Ok(v) => Some(v),
            Err(_) => {
                tracing::error!(origin = %o, "origin не разобран, пропущен");
                None
            }
        })
        .collect();

    tracing::info!(count = origins.len(), "CORS ограничен списком origin'ов");
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}
