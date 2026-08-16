//! Режим `merged`: three-way по хешам, без хранения копий файлов.
//!
//! Главный дефект `user_managed` в том, что файл не обновляется никогда:
//! сервер поправил конфиг мода — игрок остаётся на старом навсегда, даже если
//! сам файл ни разу не трогал.
//!
//! Решение стоит один json: в `.noro/base-hashes.json` лежит sha1 того, что мы
//! установили в прошлый раз. Сравнение «моё против базы» и «серверное против
//! базы» отвечает на вопрос, кто именно менял файл.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Имя внутри служебного каталога лаунчера.
const BASE_PATH: &str = ".noro/base-hashes.json";

/// Что делать с файлом.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Игрок не трогал — обновить.
    Update,
    /// Сервер не менял — оставить как есть.
    KeepMine,
    /// Меняли обе стороны.
    Conflict,
    /// Совпадает всё — делать нечего.
    Nothing,
}

/// Решение по одному файлу.
///
/// `mine` — `None`, если файла нет: тогда его надо поставить, кто бы что ни
/// менял. `base` — `None` при первом проходе, и это тоже «поставить»: базы ещё
/// нет, спорить не с чем.
pub fn decide(mine: Option<&str>, base: Option<&str>, theirs: &str) -> Decision {
    let Some(mine) = mine else {
        return Decision::Update;
    };
    let Some(base) = base else {
        // База неизвестна. Совпало с серверным — ничего не делаем, разошлось —
        // это правки игрока, которых мы не видели: считаем конфликтом, а не
        // поводом затереть.
        return if mine == theirs {
            Decision::Nothing
        } else {
            Decision::Conflict
        };
    };

    match (mine == base, theirs == base) {
        // Игрок не трогал, сервер обновил.
        (true, false) => Decision::Update,
        // Игрок правил, сервер не менял.
        (false, true) => Decision::KeepMine,
        // Правили обе стороны.
        (false, false) => {
            if mine == theirs {
                Decision::Nothing
            } else {
                Decision::Conflict
            }
        }
        (true, true) => Decision::Nothing,
    }
}

/// Хеши того, что мы установили в прошлый раз.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BaseHashes(HashMap<String, String>);

impl BaseHashes {
    pub async fn load(instance_dir: &Path) -> Self {
        match tokio::fs::read(instance_dir.join(BASE_PATH)).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn get(&self, path: &str) -> Option<&str> {
        self.0.get(path).map(String::as_str)
    }

    pub fn set(&mut self, path: &str, sha1: &str) {
        self.0.insert(path.to_string(), sha1.to_string());
    }

    /// Ошибка записи не срывает запуск: без базы следующий проход просто
    /// посчитает файлы неизвестными и не станет ничего затирать.
    pub async fn save(&self, instance_dir: &Path) {
        let path = instance_dir.join(BASE_PATH);
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(bytes) = serde_json::to_vec(&self.0) {
            let _ = tokio::fs::write(path, bytes).await;
        }
    }
}

/// Отложить версию игрока перед тем, как взять серверную.
///
/// Без копии `take_theirs` означал бы «молча стереть правки», а это ровно то,
/// от чего режим и защищает.
pub async fn backup_conflict(instance_dir: &Path, rel: &str, stamp: &str) -> std::io::Result<()> {
    let src = instance_dir.join(rel);
    let dst = instance_dir.join(".noro/conflicts").join(stamp).join(rel);
    if let Some(parent) = dst.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::copy(&src, &dst).await?;
    Ok(())
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
