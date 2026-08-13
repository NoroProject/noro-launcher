//! Проверки самого каталога миграций.
//!
//! Однажды два файла получили номер 0021. sqlx применяет первый, на втором
//! отдаёт `VersionMismatch` и **обрывает цикл** — всё, что идёт следом, не
//! применяется уже никогда. Ошибка при этом уходила в лог как info, так что
//! мастер поднимался молча, а схема отставала от кода на несколько миграций.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn migrations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

/// Номер и имя каждой миграции, по возрастанию номера.
fn migrations() -> Vec<(u32, String)> {
    let mut found: Vec<(u32, String)> = std::fs::read_dir(migrations_dir())
        .expect("каталог migrations читается")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".sql"))
        .map(|name| {
            let version = name
                .split('_')
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or_else(|| panic!("имя миграции должно начинаться с номера: {name}"));
            (version, name)
        })
        .collect();
    found.sort();
    found
}

#[test]
fn migration_versions_are_unique() {
    let mut by_version: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for (version, name) in migrations() {
        by_version.entry(version).or_default().push(name);
    }

    let duplicates: Vec<_> = by_version
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();

    assert!(
        duplicates.is_empty(),
        "номера миграций должны быть уникальны, иначе цепочка обрывается на дубле: {duplicates:?}"
    );
}

#[test]
fn migration_directory_is_not_empty() {
    assert!(
        !migrations().is_empty(),
        "миграции не найдены — путь до каталога изменился?"
    );
}
