//! Импорт модпаков: Modrinth (.mrpack), CurseForge (.zip) и обычный zip
//! с корнем сборки внутри.

pub mod curseforge;
pub mod instance_zip;
pub mod mrpack;

use anyhow::{anyhow, Result};
use axum::extract::Multipart;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ImportProgress {
    pub total: usize,
    pub current: usize,
    pub current_file: String,
    pub done: bool,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub recommended_mc_version: Option<String>,
    pub recommended_modloader_version: Option<String>,
}

/// Путь внутри сборки, либо `None`, если он опасен.
///
/// Лаунчер разложит эти пути по диску у игрока, поэтому выход за корень сборки
/// отсекается здесь, а не на клиенте: `instance_dir.join(path)` на Windows
/// принимает и `\\` как разделитель, и `C:` как корень, так что путь вида
/// `..\\..\\evil.exe` уехал бы мимо инстанса, ничем себя не выдав.
///
/// Живёт в общем модуле, потому что входов у путей сборки три: zip-импорт,
/// mrpack и переименование в админке. Пока проверка была только у первого,
/// остальные два принимали что угодно.
pub(crate) fn safe_path(raw: &str) -> Option<String> {
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with('/') || normalized.contains(':') {
        return None;
    }
    if normalized
        .split('/')
        .any(|part| part == ".." || part.trim().is_empty())
    {
        return None;
    }
    let trimmed = normalized.trim_start_matches("./").to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(test)]
#[path = "safe_path_tests.rs"]
mod safe_path_tests;

/// Вытащить байты файла из поля `file` multipart-формы.
pub(crate) async fn read_upload(mut multipart: Multipart) -> Result<Vec<u8>> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            return Ok(field.bytes().await?.to_vec());
        }
    }
    Err(anyhow!("нет поля file"))
}

/// Угадать kind по пути внутри инстанса.
pub(crate) fn kind_for(path: &str) -> &'static str {
    if path.starts_with("mods/") {
        "mod"
    } else if path.starts_with("config/") {
        "config"
    } else {
        "other"
    }
}
