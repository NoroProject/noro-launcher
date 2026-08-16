//! Кеш хешей по mtime+size.
//!
//! Без него сверка перед каждым запуском перечитывает все моды и конфиги — это
//! сотни мегабайт и заметная пауза перед окном игры. Файл, у которого не
//! изменились ни размер, ни время правки, почти наверняка не изменился и сам;
//! «почти» здесь допустимо, потому что это телеметрия, а не защита.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Имя внутри `.noro/` — служебного каталога лаунчера в инстансе.
const CACHE_PATH: &str = ".noro/hash-cache.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Entry {
    size: u64,
    /// Секунды unix-времени: наносекунды на разных ФС округляются по-разному.
    mtime: i64,
    sha1: String,
}

#[derive(Default)]
pub struct HashCache {
    entries: HashMap<String, Entry>,
    dirty: bool,
}

impl HashCache {
    pub async fn load(instance_dir: &Path) -> Self {
        let path = instance_dir.join(CACHE_PATH);
        let entries = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => HashMap::new(),
        };
        HashCache {
            entries,
            dirty: false,
        }
    }

    /// SHA1 файла. `None` — файла нет или он не читается.
    pub async fn sha1_of(&mut self, path: &Path) -> Option<String> {
        let meta = tokio::fs::metadata(path).await.ok()?;
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or_default();

        let key = path.to_string_lossy().replace('\\', "/");
        if let Some(hit) = self.entries.get(&key) {
            if hit.size == size && hit.mtime == mtime {
                return Some(hit.sha1.clone());
            }
        }

        let sha1 = super::super::integrity::sha1_file(path).await.ok()?;
        self.entries.insert(
            key,
            Entry {
                size,
                mtime,
                sha1: sha1.clone(),
            },
        );
        self.dirty = true;
        Some(sha1)
    }

    /// Ошибка записи не важна: кеш восстановится следующим проходом, а срывать
    /// из-за него запуск игры незачем.
    pub async fn save(&self, instance_dir: &Path) {
        if !self.dirty {
            return;
        }
        let path = instance_dir.join(CACHE_PATH);
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(bytes) = serde_json::to_vec(&self.entries) {
            let _ = tokio::fs::write(&path, bytes).await;
        }
    }
}
