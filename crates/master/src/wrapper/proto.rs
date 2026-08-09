//! Протокол мастер ↔ ServerWrapper поверх WebSocket.
//!
//! Кадры — JSON с полем `type`. Мастер шлёт только запросы, враппер — ответы,
//! строки консоли и статус. Каждый запрос несёт `id`, по нему же приходит
//! `reply`: без него параллельные операции перепутались бы ответами.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Что мастер просит сделать.
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum Op {
    /// `start` | `stop` | `restart` | `kill`.
    Power { action: String },
    /// Строка в консоль сервера, как если бы её набрали руками.
    Command { line: String },
    FsList { path: String },
    FsRead { path: String },
    FsWrite { path: String, content: String },
    FsDelete { path: String },
    FsMkdir { path: String },
    /// Файл качается враппером из стора мастера и кладётся в `dir`. Списка
    /// установленного отдельной операцией нет: каталог модов читается обычным
    /// `FsList`, а имя каталога мастер знает из платформы в hello.
    ModInstall {
        url: String,
        sha1: String,
        filename: String,
        dir: String,
    },
    BackupCreate { name: String },
    BackupList,
    BackupRestore { name: String },
    BackupDelete { name: String },
}

impl Op {
    /// Сколько ждать ответа. Архив мира пакуется минутами, а `stop` ждёт, пока
    /// сервер сохранится, — общий короткий таймаут рвал бы обе операции на
    /// половине.
    pub fn timeout(&self) -> Duration {
        match self {
            Op::BackupCreate { .. } | Op::BackupRestore { .. } => Duration::from_secs(900),
            Op::Power { .. } => Duration::from_secs(180),
            Op::ModInstall { .. } => Duration::from_secs(300),
            _ => Duration::from_secs(30),
        }
    }
}

/// Кто подключился. Приходит первым кадром, до всего остального.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WrapperInfo {
    pub platform: String,
    pub mc_version: String,
    pub wrapper_version: String,
    /// Абсолютный путь на игровой машине — виден админу, чтобы понимать, куда
    /// он на самом деле пишет.
    pub server_dir: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct WrapperStatus {
    pub running: bool,
    /// Сервер отпечатал `Done (` — с этого момента он принимает игроков.
    pub ready: bool,
    pub uptime_secs: u64,
    /// Код возврата последнего завершившегося процесса.
    pub exit_code: Option<i32>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromWrapper {
    Hello {
        #[serde(flatten)]
        info: WrapperInfo,
        #[serde(default)]
        status: WrapperStatus,
    },
    Status {
        #[serde(flatten)]
        status: WrapperStatus,
    },
    Console {
        line: String,
    },
    Reply {
        id: u64,
        ok: bool,
        #[serde(default)]
        data: Value,
        #[serde(default)]
        error: Option<String>,
    },
}

/// Снимок подключения для админки.
#[derive(Serialize, Clone)]
pub struct WrapperState {
    pub connected: bool,
    pub info: Option<WrapperInfo>,
    pub status: WrapperStatus,
}

impl WrapperState {
    /// Враппера нет: сервер либо погашен целиком, либо запущен мимо него.
    pub fn offline() -> Self {
        Self {
            connected: false,
            info: None,
            status: WrapperStatus::default(),
        }
    }
}
