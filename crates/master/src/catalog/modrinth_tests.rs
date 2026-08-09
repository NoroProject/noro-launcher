//! Тесты Modrinth: сборка фасетов и чтение выдачи. Сеть не нужна — ломается
//! молча как раз разбор, а не запрос.

use super::query::SearchQuery;
use super::types::Provider;
use super::{modrinth, modrinth_map};
use serde_json::json;

fn query() -> SearchQuery {
    SearchQuery {
        q: "sodium".into(),
        mc: None,
        loader: None,
        categories: String::new(),
        project_type: "mod".into(),
        sort: "relevance".into(),
        offset: 0,
        limit: 20,
        provider: "modrinth".into(),
        side: None,
    }
}

#[test]
fn facets_only_project_type_by_default() {
    assert_eq!(modrinth::facets(&query()), r#"[["project_type:mod"]]"#);
}

#[test]
fn facets_and_categories_separately() {
    // Каждая категория — своя группа: выбрав две, админ ищет мод, который и то
    // и другое. Одной группой это стало бы «или», то есть свалкой из двух.
    let mut q = query();
    q.categories = "magic, technology".into();
    assert_eq!(
        modrinth::facets(&q),
        r#"[["project_type:mod"],["categories:magic"],["categories:technology"]]"#
    );
}

#[test]
fn facets_side_matches_required_and_optional() {
    let mut q = query();
    q.side = Some("server".into());
    assert!(modrinth::facets(&q).contains(r#"["server_side:required","server_side:optional"]"#));
}

#[test]
fn empty_context_does_not_add_facets() {
    // Глобальный браузер шлёт пустые строки, а не отсутствие поля: `versions:`
    // без значения не нашёл бы ничего.
    let mut q = query();
    q.mc = Some(String::new());
    q.loader = Some(String::new());
    assert_eq!(modrinth::facets(&q), r#"[["project_type:mod"]]"#);
}

#[test]
fn modrinth_hit_reads_search_shape() {
    let hit = modrinth_map::hit(&json!({
        "project_id": "AANobbMI",
        "slug": "sodium",
        "title": "Sodium",
        "description": "Modern rendering engine",
        "author": "jellysquid3",
        "downloads": 202_411_472u64,
        "follows": 39_588,
        "categories": ["optimization", "fabric"],
        "client_side": "required",
        "server_side": "unsupported",
        "date_modified": "2026-08-07T00:57:35Z"
    }));
    assert_eq!(hit.provider, Provider::Modrinth);
    assert_eq!(hit.project_id, "AANobbMI");
    assert_eq!(hit.author, "jellysquid3");
    assert_eq!(hit.downloads, 202_411_472);
    assert_eq!(
        hit.page_url.as_deref(),
        Some("https://modrinth.com/mod/sodium")
    );
}

#[test]
fn modrinth_project_falls_back_to_id_and_followers() {
    // Карточка проекта зовёт поля иначе, чем выдача поиска: `id` вместо
    // `project_id`, `followers` вместо `follows`, `updated` вместо
    // `date_modified`. Автора в ней нет вообще.
    let project = modrinth_map::project(&json!({
        "id": "AANobbMI",
        "slug": "sodium",
        "title": "Sodium",
        "description": "Modern rendering engine",
        "followers": 39_588,
        "updated": "2026-08-07T00:57:35Z",
        "license": { "id": "LicenseRef-Polyform", "name": "" },
        "gallery": [{ "url": "https://cdn.modrinth.com/a.webp" }]
    }));
    assert_eq!(project.hit.project_id, "AANobbMI");
    assert_eq!(project.hit.follows, 39_588);
    assert_eq!(project.hit.author, "");
    // `name` пустое — берём `id`, иначе лицензия исчезла бы из карточки.
    assert_eq!(project.license.as_deref(), Some("LicenseRef-Polyform"));
    assert_eq!(project.gallery.len(), 1);
}

#[test]
fn modrinth_version_takes_primary_file() {
    let version = modrinth_map::version(&json!({
        "id": "QV48eyCs",
        "project_id": "AANobbMI",
        "name": "Sodium 0.8.13",
        "version_number": "mc1.21.1-0.8.13",
        "version_type": "beta",
        "game_versions": ["1.21.1"],
        "loaders": ["fabric"],
        "files": [
            { "filename": "sources.jar", "size": 10, "primary": false },
            { "filename": "sodium.jar", "size": 1_574_596u64, "primary": true }
        ],
        "dependencies": [{ "project_id": "P7dR8mSH", "dependency_type": "required" }]
    }))
    .expect("у версии есть файлы");
    assert_eq!(version.filename, "sodium.jar");
    assert_eq!(version.size, 1_574_596);
    assert_eq!(version.dependencies[0].kind, "required");
}

#[test]
fn modrinth_version_without_files_is_skipped() {
    assert!(modrinth_map::version(&json!({ "id": "x", "files": [] })).is_none());
}
