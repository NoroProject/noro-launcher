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
}

impl AppState {
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}
