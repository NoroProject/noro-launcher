//! Диагностика инстанса: то, что иначе видно только строчкой в логе при старте.
//!
//! «Прод на dev-ключе подписи» — самый дорогой из таких случаев: он ничего не
//! ломает сразу, а всплывает через месяц, когда лаунчер с боевым ключом
//! перестаёт принимать манифесты.

use super::PERM_SETTINGS_VIEW;
use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
pub struct Check {
    pub id: &'static str,
    pub title: &'static str,
    /// `ok` | `warn` | `fail`
    pub level: &'static str,
    pub detail: String,
}

fn check(id: &'static str, title: &'static str, ok: bool, warn: bool, detail: String) -> Check {
    Check {
        id,
        title,
        level: if ok {
            "ok"
        } else if warn {
            "warn"
        } else {
            "fail"
        },
        detail,
    }
}

pub async fn diagnostics(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    let cfg = &state.config;

    let mut checks = vec![
        check(
            "signing_key",
            "Ключ подписи манифестов",
            !cfg.is_dev_signing(),
            false,
            if cfg.is_dev_signing() {
                "DEV-ключ из исходников. Лаунчер, собранный с боевым публичным \
                 ключом, такие манифесты не примет."
                    .into()
            } else {
                "Задан в окружении".into()
            },
        ),
        check(
            "cors",
            "CORS",
            !cfg.allowed_origins.is_empty(),
            true,
            if cfg.allowed_origins.is_empty() {
                "Список пуст — API принимает любой origin. Для локальной \
                 разработки это удобно, для прода нет."
                    .into()
            } else {
                format!("{} origin'ов", cfg.allowed_origins.len())
            },
        ),
        check(
            "discord",
            "Discord OAuth",
            !cfg.discord_client_id.is_empty() && !cfg.discord_client_secret.is_empty(),
            true,
            match (
                cfg.discord_client_id.is_empty(),
                cfg.discord_client_secret.is_empty(),
            ) {
                (false, false) => "Client ID и секрет заданы".into(),
                (true, _) => "Не задан DISCORD_CLIENT_ID — вход через Discord выключен".into(),
                (_, true) => "Не задан DISCORD_CLIENT_SECRET — вход через Discord выключен".into(),
            },
        ),
        check(
            "passkey",
            "Passkey",
            state.webauthn.is_some() && cfg.web_url.starts_with("https://"),
            true,
            if state.webauthn.is_none() {
                "Выключен: не заданы публичные адреса".into()
            } else if !cfg.web_url.starts_with("https://") {
                "Сайт отдаётся по http:// — браузер не даст привязать ключ \
                 нигде, кроме localhost."
                    .into()
            } else {
                "Доступен".into()
            },
        ),
        check(
            "storage",
            "Хранилище файлов",
            true,
            false,
            match (&cfg.s3, &cfg.files_cdn_url) {
                (Some(_), _) => "S3".into(),
                (None, Some(cdn)) => format!("Диск мастера, раздача через {cdn}"),
                (None, None) => "Диск мастера, раздача мастером".into(),
            },
        ),
    ];

    // Живая проверка CDN: настроенный, но недоступный CDN означает игроков,
    // которые не могут скачать ни одного файла сборки.
    if let Some(cdn) = &cfg.files_cdn_url {
        let reachable = state
            .http
            .head(cdn)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .is_ok();
        checks.push(check(
            "cdn_reachable",
            "CDN отвечает",
            reachable,
            false,
            if reachable {
                format!("{cdn} доступен")
            } else {
                format!("{cdn} не отвечает — игроки не скачают файлы сборки")
            },
        ));
    }

    Ok(Json(json!({ "checks": checks })))
}
