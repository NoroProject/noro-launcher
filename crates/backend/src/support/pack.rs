//! Zip rather than tar.gz: the crate is already a dependency for unpacking
//! imported modpacks, and a zip opens in a browser or a file manager without
//! any extra steps.

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
        // Already redacted. The archive gets exactly what the preview showed.
        zip.start_file(&f.name, options)?;
        zip.write_all(f.text.as_bytes())?;
    }

    Ok(zip.finish()?.into_inner())
}
