//! Синхронизация под работающей игрой.
//!
//! Обычная синхронизация трогает всю папку и запускается только перед стартом:
//! под работающей игрой она снесла бы то, что держит JVM. Здесь другой объём —
//! ровно те папки, содержимое которых игра читает по требованию, а не при
//! запуске.
//!
//! Что сюда не попало и почему:
//!
//! * `saves/` — у мира есть `session.lock` и открытые файлы регионов; запись под
//!   работающей игрой рвёт сохранение;
//! * `options.txt` — игра перезаписывает его при выходе, и наша правка просто
//!   исчезнет;
//! * `mods/`, `config/` — jar'ы держит JVM, а конфиги читаются один раз при
//!   старте, так что подмена ничего не даст.
//!
//! Применение остаётся за игроком: подменённый пак сам собой не подхватится,
//! клиенту нужна перезагрузка ресурсов. Смысл живой синхронизации не в том,
//! чтобы её избежать, а в том, чтобы она случилась один раз и по делу.

use anyhow::Result;
use schema::build::{BuildManifest, FileEntry};
use std::path::Path;

/// Папки, которые можно обновлять на ходу.
const LIVE_DIRS: [&str; 2] = ["resourcepacks/", "shaderpacks/"];

/// Что удалось сделать: сколько файлов обновилось и сколько ждёт перезапуска.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub updated: Vec<String>,
    /// Файл занят игрой — заменим при следующем запуске.
    pub locked: Vec<String>,
}

impl Applied {
    pub fn nothing(&self) -> bool {
        self.updated.is_empty() && self.locked.is_empty()
    }
}

/// Живые ли это файлы: путь внутри одной из разрешённых папок.
pub fn live(path: &str) -> bool {
    LIVE_DIRS.iter().any(|dir| path.starts_with(dir))
}

/// Что из манифеста разошлось с тем, что лежит на диске.
///
/// Сравнение по хешу, а не по времени правки: пак могли положить руками, и
/// время у него окажется свежее нашего.
pub async fn outdated(instance_dir: &Path, manifest: &BuildManifest) -> Vec<FileEntry> {
    let mut out = Vec::new();
    for entry in manifest.verified_files.iter().filter(|f| live(&f.path)) {
        let path = instance_dir.join(&entry.path);
        if !matches(&path, &entry.sha1).await {
            out.push(entry.clone());
        }
    }
    out
}

async fn matches(path: &Path, sha1: &str) -> bool {
    crate::sync::integrity::sha1_file(path)
        .await
        .is_ok_and(|had| had == sha1)
}

/// Заменить файл, если игра его не держит.
///
/// На Windows открытый клиентом zip заменить нельзя — файл заблокирован. Это не
/// ошибка и не повод ругаться: пак просто встанет при следующем запуске, а
/// игроку об этом скажут.
pub async fn replace(path: &Path, bytes: &[u8]) -> Result<bool> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    // Через временный файл рядом: обрыв закачки не должен оставить половину
    // пака под тем же именем.
    let tmp = path.with_extension("noro-part");
    tokio::fs::write(&tmp, bytes).await?;
    match tokio::fs::rename(&tmp, path).await {
        Ok(()) => Ok(true),
        Err(_) => {
            let _ = tokio::fs::remove_file(&tmp).await;
            Ok(false)
        }
    }
}

/// Подтянуть свежие паки и шейдеры под работающей игрой.
///
/// Скачивается только разошедшееся: обход всей папки под игрой ни к чему, а
/// лишний трафик тем более.
pub async fn apply(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
) -> Result<Applied> {
    let mut done = Applied::default();
    for entry in outdated(instance_dir, manifest).await {
        let bytes = client.get(&entry.url).send().await?.bytes().await?;
        // Мастер мог отдать не то, что обещал: класть такое в папку игрока
        // нельзя, лучше пропустить файл и оставить прежний.
        if hex::encode(<sha1::Sha1 as sha1::Digest>::digest(&bytes)) != entry.sha1 {
            continue;
        }
        if replace(&instance_dir.join(&entry.path), &bytes).await? {
            // Скачать мало: игра берёт из папки только перечисленное в
            // `resourcePacks`, иначе пак лежит выбранным-невыбранным.
            if let Some(name) = entry.path.strip_prefix("resourcepacks/") {
                let _ = enable(instance_dir, name).await;
            }
            done.updated.push(entry.path);
        } else {
            done.locked.push(entry.path);
        }
    }
    Ok(done)
}

/// Включить пак в `options.txt`, если игрок его ещё не включал.
///
/// Скачать пак мало: игра берёт из папки только то, что перечислено в
/// `resourcePacks`. Раньше пак приезжал и молча лежал невыбранным — со стороны
/// это выглядело как «ничего не произошло».
///
/// Дописываем в конец списка и только один раз: если игрок пак сам выключил,
/// второй раз не навязываемся — строка `resourcePacks` тогда его уже не
/// содержит, но в файле остаётся наша отметка.
pub async fn enable(instance_dir: &Path, pack: &str) -> Result<bool> {
    let path = instance_dir.join("options.txt");
    let Ok(text) = tokio::fs::read_to_string(&path).await else {
        // Игра ещё не создавала настройки: включим при следующем разе.
        return Ok(false);
    };
    let entry = format!("\"file/{pack}\"");
    let Some(updated) = add_pack(&text, &entry) else {
        return Ok(false);
    };
    tokio::fs::write(&path, updated).await?;
    Ok(true)
}

/// @return новый текст либо `None`, если менять нечего
pub(super) fn add_pack(text: &str, entry: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut changed = false;
    for line in text.lines() {
        if let Some(list) = line.strip_prefix("resourcePacks:") {
            if list.contains(entry) {
                return None;
            }
            let inner = list.trim().trim_start_matches('[').trim_end_matches(']');
            let joined = if inner.trim().is_empty() {
                entry.to_string()
            } else {
                format!("{inner},{entry}")
            };
            out.push(format!("resourcePacks:[{joined}]"));
            changed = true;
            continue;
        }
        out.push(line.to_string());
    }
    changed.then(|| out.join("\n") + "\n")
}
