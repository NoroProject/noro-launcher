//! Проверка архива — единственное, что стоит между «восстановили» и «затёрли
//! живую базу мусором». Тесты бьют именно по ней.

use super::*;
use crate::signing::Signer25519;
use std::io::Write;
use std::path::{Path, PathBuf};

fn signer() -> Signer25519 {
    Signer25519::from_config(&None).unwrap()
}

/// Каталог с данными и дампом; возвращает (корень, data_dir, dump).
fn fixture(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("noro-backup-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let data = root.join("data");
    std::fs::create_dir_all(data.join("skins")).unwrap();
    std::fs::write(data.join("skins/a.png"), b"first").unwrap();
    std::fs::write(data.join("b.bin"), b"second").unwrap();
    let dump = root.join("dump.sql");
    std::fs::write(&dump, b"-- dump").unwrap();
    (root.clone(), data, dump)
}

fn build(data: &Path, dump: &Path, to: &Path, signed: bool) {
    let file = std::fs::File::create(to).unwrap();
    let now = chrono::Utc::now();
    let fast = flate2::Compression::fast();
    build::write_archive(file, dump, data, &signer(), signed, now, fast).unwrap();
}

#[test]
fn собранный_архив_проходит_проверку() {
    let (root, data, dump) = fixture("ok");
    let archive = root.join("a.tar.gz");
    build(&data, &dump, &archive, true);

    let v = inspect::verify(&archive, &signer().verifying_key(), 999).unwrap();
    assert!(v.parts_ok, "{:?}", v.problems);
    assert!(v.signature_ok);
    assert!(v.schema_ok);
    assert_eq!(v.refusal(), None);
    assert_eq!(v.meta.data_files, 2);
    // dump.sql плюс два файла данных; сам meta.json себя не хеширует.
    assert_eq!(v.meta.parts.len(), 3);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn подпись_чужим_ключом_отвергается() {
    let (root, data, dump) = fixture("foreign");
    let archive = root.join("a.tar.gz");
    build(&data, &dump, &archive, true);

    let other = Signer25519::from_config(&Some(hex::encode([7u8; 32]))).unwrap();
    let v = inspect::verify(&archive, &other.verifying_key(), 999).unwrap();
    assert!(v.parts_ok);
    assert!(!v.signature_ok);
    assert!(v.refusal().unwrap().contains("подпись"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn без_ключа_архив_помечен_неподписанным_но_годен() {
    let (root, data, dump) = fixture("unsigned");
    let archive = root.join("a.tar.gz");
    build(&data, &dump, &archive, false);

    let v = inspect::verify(&archive, &signer().verifying_key(), 999).unwrap();
    assert!(!v.meta.signed);
    assert!(!v.signature_ok);
    // Неподписанный — не значит негодный: без NORO_SIGNING_KEY подписывать
    // нечем, и отказ запер бы dev-контур целиком. Админка это показывает.
    assert_eq!(v.refusal(), None);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn подмена_содержимого_видна_по_sha256() {
    let (root, data, dump) = fixture("tamper");
    let archive = root.join("a.tar.gz");

    // Паспорт от честного архива, содержимое — подменённое: ровно то, что
    // сделал бы подсунувший свой файл в чужой архив.
    let honest = root.join("honest.tar.gz");
    build(&data, &dump, &honest, true);
    let meta_bytes = read_entry(&honest, META);
    let signature = read_entry(&honest, SIGNATURE);

    std::fs::write(data.join("b.bin"), b"EVIL!!").unwrap();
    let gz = flate2::write::GzEncoder::new(
        std::fs::File::create(&archive).unwrap(),
        flate2::Compression::fast(),
    );
    let mut tar = tar::Builder::new(gz);
    let mut parts = tar_out::Parts::new();
    tar_out::append_file(&mut tar, &mut parts, &dump, DUMP).unwrap();
    tar_out::append_file(&mut tar, &mut parts, &data.join("b.bin"), "data/b.bin").unwrap();
    tar_out::append_file(
        &mut tar,
        &mut parts,
        &data.join("skins/a.png"),
        "data/skins/a.png",
    )
    .unwrap();
    tar_out::append_bytes(&mut tar, META, &meta_bytes).unwrap();
    tar_out::append_bytes(&mut tar, SIGNATURE, &signature).unwrap();
    tar.into_inner().unwrap().finish().unwrap().flush().unwrap();

    let v = inspect::verify(&archive, &signer().verifying_key(), 999).unwrap();
    // Подпись цела — паспорт не трогали. Спасает именно sha256 частей.
    assert!(v.signature_ok);
    assert!(!v.parts_ok);
    assert!(v.refusal().unwrap().contains("data/b.bin"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn архив_новее_бинарника_отвергается() {
    let (root, data, dump) = fixture("schema");
    let archive = root.join("a.tar.gz");
    build(&data, &dump, &archive, true);

    let known = crate::db::known_schema_version() - 1;
    let v = inspect::verify(&archive, &signer().verifying_key(), known).unwrap();
    assert!(!v.schema_ok);
    let reason = v.refusal().unwrap();
    // В тексте обязаны стоять обе версии: без них админу нечего делать.
    assert!(reason.contains(&v.meta.schema_version.to_string()));
    assert!(reason.contains(&known.to_string()));
    let _ = std::fs::remove_dir_all(&root);
}

fn read_entry(archive: &Path, want: &str) -> Vec<u8> {
    use std::io::Read;
    let file = std::fs::File::open(archive).unwrap();
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    for entry in tar.entries().unwrap() {
        let mut entry = entry.unwrap();
        if entry.path().unwrap().to_string_lossy() == want {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();
            return buf;
        }
    }
    panic!("в архиве нет {want}");
}
