//! Числовые справочники CurseForge.
//!
//! У этого API вместо слов числа: тип релиза, вид зависимости, загрузчик,
//! раздел каталога. Публичного словаря к ним нет, поэтому таблицы собраны
//! здесь — чтобы «6» не встречалось по коду без объяснения, что это NeoForge.

pub fn loader_name(id: Option<u64>) -> Option<&'static str> {
    match id? {
        1 => Some("Forge"),
        4 => Some("Fabric"),
        5 => Some("Quilt"),
        6 => Some("NeoForge"),
        _ => None,
    }
}

pub fn channel(release_type: u64) -> String {
    match release_type {
        2 => "beta".into(),
        3 => "alpha".into(),
        _ => "release".into(),
    }
}

pub fn relation(kind: u64) -> String {
    match kind {
        1 | 6 => "embedded".into(),
        2 => "optional".into(),
        3 => "required".into(),
        5 => "incompatible".into(),
        _ => "optional".into(),
    }
}

/// Раздел каталога. Числа — внутренние id CurseForge, словаря для них нет.
pub fn class_id(project_type: &str) -> u32 {
    match project_type {
        "modpack" => 4471,
        "resourcepack" => 12,
        "shader" => 6552,
        "datapack" => 6945,
        "plugin" => 5,
        _ => 6,
    }
}

pub fn loader_type(loader: &str) -> Option<u8> {
    match loader.to_ascii_lowercase().as_str() {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

pub fn sort_field(sort: &str) -> u8 {
    match sort {
        "downloads" => 6,
        "follows" => 2,
        "newest" | "updated" => 3,
        _ => 1,
    }
}
