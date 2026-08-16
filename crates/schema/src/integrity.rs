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
}
