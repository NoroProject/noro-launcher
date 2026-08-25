//! Проверка архива без применения.
//!
//! Один проход по файлу: части хешируются на лету, а `meta.json` лежит в конце,
//! поэтому к моменту сравнения известно и ожидаемое, и фактическое. Тот же код
//! вызывает `restore` перед тем, как тронуть хоть один файл, — верить прошлому
//! вердикту нельзя, между проверкой и применением архив мог подменить кто
//! угодно с доступом к тому.

use super::{BackupMeta, Verdict, META, SIGNATURE};
use anyhow::{Context, Result};
use ed25519_dalek::{Signature, VerifyingKey};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

/// Прочитать архив и вынести вердикт. Ничего не трогает.
pub fn verify(archive: &Path, key: &VerifyingKey, known_schema: i64) -> Result<Verdict> {
    let (meta_bytes, signature, actual) = scan(archive)?;

    let meta_bytes = meta_bytes.context(
        "в архиве нет meta.json. Так выглядит оборванная закачка: паспорт \
         пишется последним, и до него дело не дошло",
    )?;
    let meta: BackupMeta = serde_json::from_slice(&meta_bytes).context("разобрать meta.json")?;

    let problems = compare(&meta.parts, &actual);
    let signature_ok = match (&signature, meta.signed) {
        (Some(sig), true) => check(sig, &meta_bytes, key),
        _ => false,
    };

    Ok(Verdict {
        parts_ok: problems.is_empty(),
        problems,
        signature_ok,
        known_schema_version: known_schema,
        schema_ok: meta.schema_version <= known_schema,
        meta,
    })
}

/// Пройти архив: вернуть `meta.json`, подпись и фактические sha256 остальных
/// частей.
type Scan = (Option<Vec<u8>>, Option<String>, BTreeMap<String, String>);

fn scan(archive: &Path) -> Result<Scan> {
    let file =
        std::fs::File::open(archive).with_context(|| format!("открыть {}", archive.display()))?;
    let mut tar = tar::Archive::new(GzDecoder::new(std::io::BufReader::new(file)));

    let mut meta = None;
    let mut signature = None;
    let mut actual = BTreeMap::new();

    for entry in tar.entries().context("это не tar.gz")? {
        let mut entry = entry.context("повреждённая запись архива")?;
        let name = entry.path()?.to_string_lossy().replace('\\', "/");

        match name.as_str() {
            META => {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                meta = Some(buf);
            }
            SIGNATURE => {
                let mut buf = String::new();
                entry.read_to_string(&mut buf)?;
                signature = Some(buf.trim().to_string());
            }
            // `dump.sql`, `data/…` и всё, чего мы не ждали: постороннее
            // содержимое молча не проглатывается — его нет в `meta.json`, и
            // `compare` доложит о нём как о лишней части.
            _ => {
                actual.insert(name, hash_entry(&mut entry)?);
            }
        }
    }
    Ok((meta, signature, actual))
}

fn hash_entry<R: Read>(entry: &mut R) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = entry.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Что разошлось между паспортом и содержимым.
fn compare(expected: &BTreeMap<String, String>, actual: &BTreeMap<String, String>) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, want) in expected {
        match actual.get(name) {
            None => problems.push(format!("части {name} нет в архиве")),
            Some(got) if got != want => problems.push(format!("{name} не сходится по sha256")),
            Some(_) => {}
        }
    }
    for name in actual.keys() {
        if !expected.contains_key(name) {
            problems.push(format!("{name} есть в архиве, но не описан в meta.json"));
        }
    }
    // Список расхождений может быть огромным — целиком его всё равно никто не
    // читает, а сути «архиву верить нельзя» первые строки не меняют.
    problems.truncate(20);
    problems
}

/// Подпись — Ed25519 по sha256(meta.json). Один паспорт закрывает весь архив:
/// хеши остальных частей лежат внутри него.
fn check(signature_hex: &str, meta_bytes: &[u8], key: &VerifyingKey) -> bool {
    let Ok(raw) = hex::decode(signature_hex) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&raw) else {
        return false;
    };
    key.verify_strict(&Sha256::digest(meta_bytes), &sig).is_ok()
}
