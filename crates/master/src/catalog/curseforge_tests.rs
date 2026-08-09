//! Тесты CurseForge: разбор файлов и числовых справочников.

use super::curseforge_map;
use serde_json::json;

#[test]
fn curseforge_version_splits_loaders_from_game_versions() {
    // CurseForge валит версии игры и загрузчики в один список.
    let version = curseforge_map::version(&json!({
        "id": 5_432_100u64,
        "modId": 238_222u64,
        "displayName": "JEI 19.21.0",
        "releaseType": 1,
        "gameVersions": ["1.21.1", "NeoForge", "Forge"],
        "fileName": "jei.jar",
        "fileLength": 1024,
        "downloadUrl": "https://edge.forgecdn.net/jei.jar",
        "dependencies": [{ "modId": 1, "relationType": 5 }]
    }));
    assert_eq!(version.game_versions, vec!["1.21.1"]);
    assert_eq!(version.loaders, vec!["neoforge", "forge"]);
    assert_eq!(version.channel, "release");
    assert!(version.downloadable);
    assert_eq!(version.dependencies[0].kind, "incompatible");
}

#[test]
fn curseforge_version_without_download_url_is_not_installable() {
    // Автор мог запретить стороннюю загрузку — версию показываем, ставить не
    // даём. Иначе кнопка была бы, а установка падала с невнятной ошибкой.
    let version = curseforge_map::version(&json!({
        "id": 1u64, "modId": 2u64, "displayName": "x",
        "releaseType": 3, "gameVersions": [], "fileName": "x.jar", "fileLength": 0
    }));
    assert!(!version.downloadable);
    assert_eq!(version.channel, "alpha");
}
