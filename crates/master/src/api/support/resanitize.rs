//! Второй проход санитизации, уже на мастере.
//!
//! Обязателен и не является дублированием: клиента можно подменить, и логи от
//! подменённого клиента не должны попасть во вьювер вместе с токенами. Первый
//! проход в лаунчере нужен ради другого — чтобы игрок в предпросмотре видел
//! ровно то, что уедет.

use anyhow::{bail, Result};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Потолок на распакованный файл: архив может быть zip-бомбой.
const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 64;

/// Разобрать архив, очистить содержимое и собрать заново.
///
/// Возвращает ошибку на всём, что не разбирается: принять непонятный архив и
/// положить его в хранилище значит потерять смысл проверки.
pub fn resanitize_zip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    if archive.len() > MAX_ENTRIES {
        bail!("в бандле слишком много файлов: {}", archive.len());
    }

    let mut out = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if !entry.is_file() {
            continue;
        }
        if entry.size() > MAX_ENTRY_BYTES {
            bail!("файл {} в бандле слишком велик", entry.name());
        }
        // Имя из архива в файловую систему не попадает — оно только внутри
        // нового zip, но обход каталогов в нём тоже ни к чему.
        let name = entry.name().replace('\\', "/").replace("..", "_");

        let mut raw = String::new();
        if entry.read_to_string(&mut raw).is_err() {
            // Бинарь в бандле логов не нужен и очистке не поддаётся.
            continue;
        }
        out.start_file(&name, options)?;
        out.write_all(schema::redact(&raw).as_bytes())?;
    }

    Ok(out.finish()?.into_inner())
}

#[cfg(test)]
#[path = "resanitize_tests.rs"]
mod tests;
