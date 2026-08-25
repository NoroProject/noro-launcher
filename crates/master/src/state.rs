//! Общее состояние приложения, шарится между всеми хендлерами.

use crate::catalog::HttpCache;
use crate::config::Config;
use crate::files::FileStore;
use crate::signing::Signer25519;
use crate::ws::WsHub;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub ws: WsHub,
    /// Открытые вкладки админки. Отдельно от лаунчеров: кадры у них разные.
    pub admin_ws: crate::ws::AdminHub,
    pub files: FileStore,
    pub signer: Signer25519,
    /// RSA-подпись профилей Yggdrasil — без неё игроки не видят чужие скины.
    pub profile_signer: Arc<crate::yggdrasil_sign::ProfileSigner>,
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    pub import_jobs: Arc<dashmap::DashMap<uuid::Uuid, crate::build_importer::ImportProgress>>,
    /// Кеш ответов Modrinth/CurseForge.
    pub catalog: HttpCache,
    /// Подключённые ServerWrapper'ы игровых серверов.
    pub wrappers: crate::wrapper::WrapperHub,
    /// Агенты внутри игры — им уходят наказания.
    pub agents: crate::agent_link::AgentHub,
    /// Кто сейчас в игре, поимённо. В памяти: это снимок текущего момента, а
    /// не история — её ведут сессии игроков в базе.
    pub roster: crate::agent_link::Roster,
    /// Проверка passkey. Собирается один раз при старте. `None` — публичные
    /// адреса ещё не заданы: домен, к которому браузер привяжет ключ, вывести
    /// не из чего, и обещать игроку passkey нельзя.
    pub webauthn: Option<Arc<webauthn_rs::Webauthn>>,
}

impl AppState {
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Проверка passkey либо внятный отказ.
    pub fn webauthn(&self) -> Result<&webauthn_rs::Webauthn, crate::error::AppError> {
        self.webauthn.as_deref().ok_or_else(|| {
            crate::error::AppError::BadRequest(
                "passkeys need the public website and API addresses to be set".into(),
            )
        })
    }

    /// Работают ли сторонние OAuth2-приложения.
    ///
    /// Читается из БД на каждый вход, а не из конфига при старте: выключатель
    /// нужен ровно в тот момент, когда чужое приложение ведёт себя плохо, и
    /// ждать перезапуска мастера тогда неуместно. Официальных не касается.
    pub async fn oauth_apps_enabled(&self) -> bool {
        self.flag(crate::config::keys::OAUTH_APPS_ENABLED).await
    }

    /// Можно ли игрокам заводить новые приложения.
    pub async fn oauth_apps_creation_enabled(&self) -> bool {
        self.flag(crate::config::keys::OAUTH_APPS_CREATION).await
    }

    /// Булев тумблер инстанса. Умолчание — «включено»: инстанс без записи в
    /// таблице ведёт себя как до появления выключателя.
    async fn flag(&self, key: &str) -> bool {
        match crate::db::get_setting(&self.db, key).await {
            Ok(Some(serde_json::Value::Bool(v))) => v,
            Ok(Some(serde_json::Value::String(s))) => s != "false",
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("не прочитать настройку {key}: {e:#} — считаю включённой");
                true
            }
        }
    }
}
