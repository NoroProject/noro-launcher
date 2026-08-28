//! Упаковка бандла в архив.
//!
//! Zip, а не tar.gz: файлов в бандле несколько, а zip уже есть в зависимостях
//! (им распаковываются импортируемые модпаки). Читается он в браузере и в
//! проводнике без лишних шагов.

use super::Bundle;
use anyhow::Result;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn pack(bundle: &Bundle) -> Result<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("environment.txt", options)?;
    zip.write_all(bundle.environment.as_bytes())?;

    for f in &bundle.files {
        // Уже очищенный текст: в архив кладётся ровно то, что игрок видел в
        // предпросмотре, иначе кнопка «посмотреть, что отправится» врёт.
        zip.start_file(&f.name, options)?;
        zip.write_all(f.text.as_bytes())?;
    }

    Ok(zip.finish()?.into_inner())
}
