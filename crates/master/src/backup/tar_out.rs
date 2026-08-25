//! Запись частей в tar и труба из блокирующего писателя в тело HTTP-ответа.

use super::hash_read::{self, HashRead};
use anyhow::{Context, Result};
use bytes::Bytes;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use tokio::sync::mpsc::Sender;

/// Куда сложены sha256 уже записанных частей.
pub type Parts = BTreeMap<String, String>;

/// Писатель, отдающий байты в канал тела ответа.
///
/// Архив собирается в `spawn_blocking` (tar и gzip блокирующие), а тело ответа
/// асинхронное. Промежуточного файла нет намеренно: копия архива в гигабайты
/// легла бы на тот же том, где может не хватить места, — то есть ровно там, где
/// её быть не должно.
pub struct ChanWrite(pub Sender<io::Result<Bytes>>);

impl Write for ChanWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .blocking_send(Ok(Bytes::copy_from_slice(buf)))
            // Клиент закрыл соединение — сборку продолжать незачем.
            .map_err(|_| io::Error::other("получатель архива отключился"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Записать файл с диска и запомнить его sha256.
///
/// Возвращает размер, который реально ушёл в архив.
pub fn append_file<W: Write>(
    tar: &mut tar::Builder<W>,
    parts: &mut Parts,
    path: &Path,
    name: &str,
) -> Result<u64> {
    let file = std::fs::File::open(path).with_context(|| format!("открыть {}", path.display()))?;
    let size = file.metadata()?.len();
    let digest = hash_read::digest();

    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    tar.append_data(&mut header, name, HashRead::new(file, size, digest.clone()))
        .with_context(|| format!("записать в архив {name}"))?;

    parts.insert(name.to_string(), hash_read::finish(&digest));
    Ok(size)
}

/// Записать готовые байты: `meta.json` и `signature`.
///
/// В `parts` не попадают — `meta.json` не может держать хеш самого себя, а
/// подпись покрывается им же.
pub fn append_bytes<W: Write>(tar: &mut tar::Builder<W>, name: &str, data: &[u8]) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    tar.append_data(&mut header, name, data)
        .with_context(|| format!("записать в архив {name}"))?;
    Ok(())
}
