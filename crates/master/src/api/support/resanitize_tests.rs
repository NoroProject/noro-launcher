//! Смысл второго прохода в том, что клиент может быть подменён: тесты бьют
//! именно по этому — архив, собранный «неправильным» лаунчером.

use super::*;

const TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.Qm9ndXM";

fn zip_with(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for (name, body) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn read_entry(bytes: &[u8], name: &str) -> Option<String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut file = archive.by_name(name).ok()?;
    let mut out = String::new();
    file.read_to_string(&mut out).ok()?;
    Some(out)
}

#[test]
fn a_token_from_a_patched_client_is_stripped_here() {
    // Лаунчер обязан был вырезать это сам; предполагаем, что он этого не сделал.
    let raw = zip_with(&[("logs/latest.log", &format!("--accessToken {TOKEN}"))]);

    let clean = resanitize_zip(&raw).unwrap();

    let text = read_entry(&clean, "logs/latest.log").unwrap();
    assert!(!text.contains(TOKEN), "{text}");
    assert!(text.contains("--accessToken *****"), "{text}");
}

#[test]
fn ordinary_content_survives_the_second_pass() {
    let raw = zip_with(&[("logs/latest.log", "[main/INFO]: Loading 148 mods")]);

    let clean = resanitize_zip(&raw).unwrap();

    assert_eq!(
        read_entry(&clean, "logs/latest.log").unwrap(),
        "[main/INFO]: Loading 148 mods"
    );
}

#[test]
fn a_path_traversal_name_cannot_escape() {
    let raw = zip_with(&[("../../etc/passwd", "root:x:0:0")]);

    let clean = resanitize_zip(&raw).unwrap();

    let mut archive = ZipArchive::new(Cursor::new(&clean[..])).unwrap();
    let name = archive.by_index(0).unwrap().name().to_string();
    assert!(!name.contains(".."), "{name}");
}

#[test]
fn a_bundle_with_too_many_files_is_rejected() {
    let names: Vec<String> = (0..MAX_ENTRIES + 1).map(|i| format!("f{i}.log")).collect();
    let entries: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), "x")).collect();

    assert!(resanitize_zip(&zip_with(&entries)).is_err());
}

#[test]
fn garbage_is_rejected_rather_than_stored() {
    // Принять непонятный архив и положить его в хранилище значит потерять
    // смысл проверки.
    assert!(resanitize_zip(b"not a zip at all").is_err());
}

#[test]
fn a_binary_entry_is_dropped() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("screenshot.png", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(&[0x89, b'P', b'N', b'G', 0x00, 0xFF])
        .unwrap();
    let raw = zip.finish().unwrap().into_inner();

    let clean = resanitize_zip(&raw).unwrap();

    let archive = ZipArchive::new(Cursor::new(&clean[..])).unwrap();
    assert_eq!(archive.len(), 0);
}
