//! Распаковка архива рядом и подмена содержимого тома данных.
//!
//! Сначала архив разворачивается целиком в staging, и только потом файлы
//! меняются местами. Половинчатая распаковка не должна убивать текущие данные:
//! место кончилось на середине — значит, не тронуто ничего.

use super::{DUMP, WORK};
use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Развернуть `data/` в staging, а `dump.sql` — отдельным файлом рядом.
/// Возвращает оба пути.
pub fn extract(archive: &Path, work: &Path, stamp: &str) -> Result<(PathBuf, PathBuf)> {
    let staged = work.join(format!("restore-{stamp}"));
    let dump = work.join(format!("restore-{stamp}.sql"));
    if staged.exists() {
        std::fs::remove_dir_all(&staged)?;
    }
    std::fs::create_dir_all(&staged)?;

    let file = std::fs::File::open(archive)?;
    let mut tar = tar::Archive::new(GzDecoder::new(std::io::BufReader::new(file)));
    for entry in tar.entries()? {
        let mut entry = entry.context("повреждённая запись архива")?;
        let name = entry.path()?.to_string_lossy().replace('\\', "/");

        let dest = if name == DUMP {
            dump.clone()
        } else if let Some(rel) = safe_rel(&name) {
            staged.join(rel)
        } else {
            // meta.json, signature — уже проверены; всё прочее отсеяно проверкой
            // как лишняя часть, до сюда не доходит.
            continue;
        };
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry
            .unpack(&dest)
            .with_context(|| format!("распаковать {name}"))?;
    }
    Ok((staged, dump))
}

/// Путь внутри `data/`, если он безопасен.
///
/// Архив мог собрать не мастер: запись `data/../../etc/passwd` развернулась бы
/// мимо тома. Наружу ведут `..`, абсолютные пути и корни дисков — всё это тут
/// и отсекается.
pub(super) fn safe_rel(name: &str) -> Option<PathBuf> {
    let rel = name.strip_prefix(&format!("{}/", super::DATA))?;
    let mut out = PathBuf::new();
    for part in Path::new(rel).components() {
        match part {
            Component::Normal(p) => out.push(p),
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// Поменять содержимое тома на распакованное.
///
/// Меняется содержимое, а не сам каталог: на проде `NORO_DATA_DIR` — точка
/// монтирования (`noro-data:/app/data`), и `rename` на нём отвечает `EBUSY`.
/// Все переносы идут внутри одной файловой системы, поэтому это перевешивание
/// ссылок, а не копирование гигабайтов.
///
/// Прежние файлы не удаляются, а уезжают в `.noro-backup/old-{дата}`: если
/// архив окажется не тем, вернуть их — вопрос одной команды `mv`.
pub fn swap(data_dir: &Path, staged: &Path, stamp: &str) -> Result<()> {
    let old = old_dir(data_dir, stamp);
    std::fs::create_dir_all(&old)?;

    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        // Служебный каталог остаётся на месте: в нём и staging, и то, куда мы
        // сейчас складываем вытесненное.
        if entry.file_name() == OsStr::new(WORK) {
            continue;
        }
        std::fs::rename(entry.path(), old.join(entry.file_name()))
            .with_context(|| format!("отложить {}", entry.path().display()))?;
    }
    for entry in std::fs::read_dir(staged)? {
        let entry = entry?;
        std::fs::rename(entry.path(), data_dir.join(entry.file_name()))
            .with_context(|| format!("вернуть {}", entry.path().display()))?;
    }
    std::fs::remove_dir_all(staged)?;
    Ok(())
}

/// Куда уезжают вытесненные файлы. Имя считается здесь, чтобы ответ админке и
/// само перемещение не разъехались.
pub fn old_dir(data_dir: &Path, stamp: &str) -> PathBuf {
    super::work_dir(data_dir).join(format!("old-{stamp}"))
}

/// То же имя относительно тома — его и показывают админу.
pub fn old_label(stamp: &str) -> String {
    format!("{WORK}/old-{stamp}")
}

/// Убрать из служебного каталога всё, чему больше `max_age_hours`.
///
/// Загруженные архивы весят как весь том, и без уборки первый же отменённый
/// разбор оставлял бы их лежать до конца места на диске.
pub fn sweep(work: &Path, max_age_hours: i64) -> Result<()> {
    let Ok(dir) = std::fs::read_dir(work) else {
        return Ok(());
    };
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(max_age_hours.max(1) as u64 * 3600);
    for entry in dir.flatten() {
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .map(|m| m < cutoff)
            .unwrap_or(false);
        if !stale {
            continue;
        }
        let path = entry.path();
        let removed = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(e) = removed {
            tracing::warn!(path = %path.display(), error = %e, "не убрался старый файл бэкапа");
        }
    }
    Ok(())
}

/// Проверка, что служебный каталог вообще пригоден: без него ни загрузить
/// архив, ни развернуть его негде.
pub fn ensure(work: &Path) -> Result<()> {
    std::fs::create_dir_all(work).with_context(|| format!("создать {}", work.display()))?;
    if !work.is_dir() {
        bail!("{} — не каталог", work.display());
    }
    Ok(())
}
