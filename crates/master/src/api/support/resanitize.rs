//! Second sanitising pass, on the master this time.
//!
//! Not redundant with the launcher's pass: a patched client can send whatever
//! it likes, and its logs must not reach the viewer with tokens still in them.
//! The launcher pass exists for a different reason — so the player's preview
//! shows exactly what will be uploaded.

use anyhow::{bail, Result};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Cap on the uncompressed size of one entry — the archive may be a zip bomb.
const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 64;

/// Unpack the archive, redact every entry and pack it again.
///
/// Anything that doesn't parse is an error: storing an archive we couldn't read
/// would defeat the point of the pass.
pub fn resanitize_zip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    if archive.len() > MAX_ENTRIES {
        bail!("too many files in the bundle: {}", archive.len());
    }

    let mut out = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if !entry.is_file() {
            continue;
        }
        if entry.size() > MAX_ENTRY_BYTES {
            bail!("bundle entry {} is too large", entry.name());
        }
        // The name never reaches the filesystem — it only goes into the new
        // zip — but there's no reason to carry traversal into that either.
        let name = entry.name().replace('\\', "/").replace("..", "_");

        let mut raw = String::new();
        if entry.read_to_string(&mut raw).is_err() {
            // Binary content can't be redacted and has no place in a log bundle.
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
