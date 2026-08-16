//! Отчёт о целостности игрового каталога.
//!
//! Клиентский сигнал, а не доказательство: лаунчер открыт, свой билд отправит
//! что угодно. Поэтому находки — повод для ручного разбора, никогда не автобан.
//! Против мотивированного нарушителя работает серверная проверка через агента;
//! это второй, дешёвый слой, который ловит остальное.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityKind {
    /// Файл в managed-пути, которого нет в манифесте.
    ExtraFile,
    /// Хеш не совпал с манифестом.
    ModifiedFile,
    /// Файл из манифеста отсутствует.
    MissingFile,
    /// Включён limited-мод, права на который нет.
    ForbiddenOptionalMod,
}

impl IntegrityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            IntegrityKind::ExtraFile => "extra_file",
            IntegrityKind::ModifiedFile => "modified_file",
            IntegrityKind::MissingFile => "missing_file",
            IntegrityKind::ForbiddenOptionalMod => "forbidden_optional_mod",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityFinding {
    pub kind: IntegrityKind,
    /// Относительный путь внутри инстанса либо имя мода.
    pub subject: String,
    /// Что именно разошлось: ожидаемый и фактический хеш, размер.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Лаунчер убрал находку сам (удалил лишний файл).
    #[serde(default)]
    pub repaired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub server_id: Uuid,
    pub build_id: Uuid,
    pub build_version: String,
    pub launcher_version: String,
    /// Опциональные моды, включённые на момент запуска.
    #[serde(default)]
    pub enabled_optional: Vec<String>,
    pub findings: Vec<IntegrityFinding>,
    /// Сколько файлов сверено — чтобы отличать «всё чисто» от «проверка не шла».
    pub checked_files: u32,
    /// Нашёлся файл, из-за которого игру запускать нельзя.
    #[serde(default)]
    pub block_launch: bool,
}

/// Диагностический снапшот лаунчера.
///
/// Личного здесь нет — версии, железо и скорость до мастера, — поэтому согласие
/// не спрашивается. Закрывает половину тикетов «не запускается» без единого
/// файла логов.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub launcher_version: String,
    pub os: String,
    pub arch: String,
    /// Java, которой запускается игра, если она уже установлена.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<String>,
    /// Свободно на диске под каталогом лаунчера, в мегабайтах.
    #[serde(default)]
    pub disk_free_mb: u64,
    /// Сколько занимает каталог лаунчера, в мегабайтах.
    #[serde(default)]
    pub data_size_mb: u64,
    /// Миллисекунды до мастера. `None` — не ответил.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_ping_ms: Option<u64>,
    /// Последняя ошибка синхронизации, если была.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_error: Option<String>,
    /// Установленные сборки: сервер и версия.
    #[serde(default)]
    pub instances: Vec<(String, String)>,
}

/// Что мастер просит сделать лаунчер.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteAction {
    /// Пересверить каталог с манифестом.
    VerifyIntegrity,
    /// Снести кэш ассетов: он восстановим и чаще всего именно он и битый.
    ClearAssetCache,
    /// Переустановить сборку с нуля.
    ReinstallBuild,
    /// Перезапустить лаунчер.
    RestartLauncher,
}

impl RemoteAction {
    /// Требует ли подтверждения у игрока.
    ///
    /// Всё, что стирает файлы или прерывает работу, — да. Сверка целостности
    /// ничего не портит и идёт молча.
    pub fn needs_confirmation(self) -> bool {
        !matches!(self, RemoteAction::VerifyIntegrity)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RemoteAction::VerifyIntegrity => "verify_integrity",
            RemoteAction::ClearAssetCache => "clear_asset_cache",
            RemoteAction::ReinstallBuild => "reinstall_build",
            RemoteAction::RestartLauncher => "restart_launcher",
        }
    }
}
