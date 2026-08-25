//! Полная резервная копия мастера: сборка, проверка, восстановление.
//!
//! В `GET /api/admin/backup` лежит только дамп БД. Он не спасает от потери
//! тома: сборки, скины, плащи и агенты живут файлами, и без них восстановленная
//! база ссылается в пустоту. Здесь — архив целиком.
//!
//! Порядок записей в архиве не случаен. `meta.json` держит sha256 остальных
//! частей, поэтому пишется **последним**: пока файлы не прочитаны, их хеши
//! неизвестны. Побочный выигрыш — оборванная закачка не проходит проверку сама
//! собой: у обрезанного архива нет `meta.json`, и он отвергается целиком, а не
//! разворачивается наполовину.

pub mod build;
pub mod db_load;
pub mod hash_read;
pub mod inspect;
pub mod restore;
pub mod swap;
pub mod tar_out;
pub mod ticket;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Имена частей внутри архива.
pub const META: &str = "meta.json";
pub const DUMP: &str = "dump.sql";
pub const SIGNATURE: &str = "signature";
/// Префикс, под которым лежит содержимое `NORO_DATA_DIR`.
pub const DATA: &str = "data";

/// Служебный каталог **внутри** `NORO_DATA_DIR`: загрузки, распакованный дамп,
/// staging восстановления и вытесненные файлы.
///
/// Именно внутри, а не рядом. На проде `NORO_DATA_DIR` — точка монтирования
/// тома (`noro-data:/app/data`), и переименовать сам каталог нельзя: ядро
/// отвечает `EBUSY`. Подменяется содержимое, а для этого staging обязан лежать
/// на той же файловой системе, иначе `rename` уходит в `EXDEV`.
pub const WORK: &str = ".noro-backup";

/// Паспорт архива. Единственная часть, которая подписывается: sha256 остальных
/// частей лежат здесь, так что одна проверка закрывает весь архив.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    /// Версия мастера, собравшего архив.
    pub master_version: String,
    /// `MAX(version)` миграций на момент сборки.
    pub schema_version: i64,
    pub created_at: DateTime<Utc>,
    pub dump_bytes: u64,
    pub data_bytes: u64,
    pub data_files: usize,
    /// Подписан ли архив. Без `NORO_SIGNING_KEY` мастер подписывать нечем —
    /// молча отдать неподписанный архив нельзя, поэтому это поле есть всегда.
    pub signed: bool,
    /// Путь части внутри архива → sha256 в hex. Сам `meta.json` и `signature`
    /// сюда не входят: они себя не хешируют.
    pub parts: BTreeMap<String, String>,
}

/// Итог проверки архива — то же самое, что видит и `restore`, и админка перед
/// подтверждением.
#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub meta: BackupMeta,
    /// Подпись сошлась с публичным ключом этого мастера.
    pub signature_ok: bool,
    /// Все части на месте и совпали по sha256.
    pub parts_ok: bool,
    /// Что именно не сошлось — по одной строке на часть.
    pub problems: Vec<String>,
    /// Версия схемы этого бинарника.
    pub known_schema_version: i64,
    /// Архив не новее бинарника.
    pub schema_ok: bool,
}

impl Verdict {
    /// Почему восстановление запрещено, если запрещено.
    ///
    /// Проверяется до того, как тронут хоть один файл: развернуть архив и
    /// узнать о расхождении потом — это ровно та поломка, ради которой бэкап и
    /// делался.
    pub fn refusal(&self) -> Option<String> {
        if !self.parts_ok {
            return Some(format!(
                "содержимое архива не сошлось с meta.json: {}",
                self.problems.join("; ")
            ));
        }
        if self.meta.signed && !self.signature_ok {
            return Some(
                "подпись архива не сходится с ключом этого мастера — архив собран \
                 не здесь или изменён после сборки"
                    .into(),
            );
        }
        if !self.schema_ok {
            return Some(format!(
                "архив собран на схеме {}, а этот мастер знает {}. Миграции вперёд \
                 не откатываются: обновите мастер до версии не ниже той, что \
                 собирала архив",
                self.meta.schema_version, self.known_schema_version
            ));
        }
        None
    }
}

/// Служебный каталог внутри тома данных.
pub fn work_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(WORK)
}

/// Имя файла архива для заголовка `Content-Disposition`.
pub fn archive_name(now: DateTime<Utc>) -> String {
    format!("noro-backup-{}.tar.gz", now.format("%Y-%m-%d-%H%M%S"))
}

/// Метка времени для служебных каталогов (`old-…`, `restore-…`).
pub fn stamp(now: DateTime<Utc>) -> String {
    now.format("%Y-%m-%d-%H%M%S").to_string()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "swap_tests.rs"]
mod swap_tests;
