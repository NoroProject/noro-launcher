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
}
