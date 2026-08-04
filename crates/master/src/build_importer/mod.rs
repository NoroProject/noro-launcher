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
