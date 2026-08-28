//! Инвентарь несинхронизируемых папок.
//!
//! Unmanaged-пути не сканировались вообще: что бы туда ни положили, мастер об
//! этом не узнавал. Здесь они обходятся, а новое и изменённое уезжает флагом.
//!
//! Ключевое ограничение — **не хешировать всё подряд**: `saves/` это гигабайты.
//! Смотрим только на список наблюдаемых путей, а миры, логи, скриншоты и креши
//! исключены жёстко.

use schema::{IntegrityFinding, IntegrityKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

const INVENTORY_PATH: &str = ".noro/inventory.json";

/// Где вообще имеет смысл смотреть.
///
/// Ровно те каталоги, куда игрок кладёт исполняемое или влияющее на игру.
/// `saves/`, `logs/`, `screenshots/` и `crash-reports/` исключены жёстко: там
/// либо гигабайты, либо ничего интересного.
const WATCHED: [&str; 4] = ["mods", "config", "resourcepacks", "shaderpacks"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Entry {
    size: u64,
    sha1: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    files: BTreeMap<String, Entry>,
    /// Первый проход помечается, чтобы админку не завалило флагами при
    /// раскатке: у всех игроков разом «появились» бы все их файлы.
    #[serde(default)]
    initial_done: bool,
}

impl Inventory {
    pub async fn load(instance_dir: &Path) -> Self {
        match tokio::fs::read(instance_dir.join(INVENTORY_PATH)).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub async fn save(&self, instance_dir: &Path) {
        let path = instance_dir.join(INVENTORY_PATH);
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(bytes) = serde_json::to_vec(self) {
            let _ = tokio::fs::write(path, bytes).await;
        }
    }
}

/// Пройти наблюдаемые пути и сравнить с прошлым разом.
///
/// Возвращает находки — новые и изменённые файлы, которых нет в манифесте.
pub async fn scan(instance_dir: &Path, known: &[String]) -> (Vec<IntegrityFinding>, Inventory) {
    let inventory = Inventory::load(instance_dir).await;
    let mut fresh = Inventory {
        files: BTreeMap::new(),
        initial_done: true,
    };
    let mut findings = Vec::new();

    for (rel, path) in candidates(instance_dir).await {
        // Файлы сборки нас не интересуют: их целостность проверяется отдельно.
        if known.iter().any(|k| k == &rel) {
            continue;
        }
        let Ok(meta) = tokio::fs::metadata(&path).await else {
            continue;
        };
        let Ok(sha1) = super::integrity::sha1_file(&path).await else {
            continue;
        };
        let entry = Entry {
            size: meta.len(),
            sha1,
        };

        // Первый проход только запоминает: иначе при раскатке админка получила
        // бы флаг на каждый файл каждого игрока разом.
        if inventory.initial_done {
            match inventory.files.get(&rel) {
                None => findings.push(finding(&rel, "новый файл")),
                Some(old) if old != &entry => findings.push(finding(&rel, "изменён")),
                Some(_) => {}
            }
        }
        fresh.files.insert(rel, entry);
    }

    (findings, fresh)
}

fn finding(rel: &str, detail: &str) -> IntegrityFinding {
    IntegrityFinding {
        kind: IntegrityKind::ExtraFile,
        subject: rel.to_string(),
        detail: Some(detail.to_string()),
        // Ничего не удаляем: это несинхронизируемые пути, файлы там принадлежат
        // игроку. Флаг — повод посмотреть, а не действие.
        repaired: false,
    }
}

async fn candidates(instance_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let root = instance_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut out = Vec::new();
        for dir in WATCHED {
            for entry in walkdir::WalkDir::new(root.join(dir))
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let Ok(rel) = entry.path().strip_prefix(&root) else {
                    continue;
                };
                out.push((
                    rel.to_string_lossy().replace('\\', "/"),
                    entry.path().to_path_buf(),
                ));
            }
        }
        out
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
#[path = "inventory_tests.rs"]
mod tests;
