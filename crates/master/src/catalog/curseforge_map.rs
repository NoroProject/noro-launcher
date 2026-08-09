//! CurseForge JSON → нормализованные типы каталога.

use super::curseforge_ids::{channel, loader_name, relation};
use super::json::{items, num, opt_text, text};
use super::types::{Category, ModDependency, ModHit, ModProject, ModVersion, Provider};
use serde_json::Value;

/// Загрузчики, которые CurseForge валит в один список с версиями игры.
const LOADERS: [&str; 8] = [
    "Forge",
    "NeoForge",
    "Fabric",
    "Quilt",
    "Rift",
    "LiteLoader",
    "Bukkit",
    "Paper",
];

pub fn hit(v: &Value) -> ModHit {
    ModHit {
        provider: Provider::Curseforge,
        project_id: num(v, "id").to_string(),
        slug: text(v, "slug"),
        title: text(v, "name"),
        description: text(v, "summary"),
        author: items(v, "authors")
            .first()
            .map(|a| text(a, "name"))
            .unwrap_or_default(),
        icon_url: opt_text(&v["logo"], "url"),
        downloads: num(v, "downloadCount"),
        follows: num(v, "thumbsUpCount"),
        categories: items(v, "categories")
            .iter()
            .map(|c| text(c, "name"))
            .collect(),
        // CurseForge про стороны не рассказывает — врать «required» нельзя.
        client_side: "unknown".into(),
        server_side: "unknown".into(),
        updated: opt_text(v, "dateModified"),
        page_url: opt_text(&v["links"], "websiteUrl"),
    }
}

pub fn project(v: &Value, body: String) -> ModProject {
    let indexes = items(v, "latestFilesIndexes");
    let mut game_versions: Vec<String> = indexes.iter().map(|i| text(i, "gameVersion")).collect();
    game_versions.sort();
    game_versions.dedup();
    let mut loaders: Vec<String> = indexes
        .iter()
        .filter_map(|i| loader_name(i["modLoader"].as_u64()))
        .map(str::to_lowercase)
        .collect();
    loaders.sort();
    loaders.dedup();

    ModProject {
        hit: hit(v),
        body,
        gallery: items(v, "screenshots")
            .iter()
            .filter_map(|s| opt_text(s, "url"))
            .collect(),
        source_url: opt_text(&v["links"], "sourceUrl"),
        issues_url: opt_text(&v["links"], "issuesUrl"),
        wiki_url: opt_text(&v["links"], "wikiUrl"),
        license: None,
        game_versions,
        loaders,
    }
}

pub fn version(v: &Value) -> ModVersion {
    let tags = super::json::str_list(v, "gameVersions");
    let (loaders, game_versions): (Vec<String>, Vec<String>) = tags
        .into_iter()
        .partition(|t| LOADERS.iter().any(|l| l.eq_ignore_ascii_case(t)));

    ModVersion {
        provider: Provider::Curseforge,
        id: num(v, "id").to_string(),
        project_id: num(v, "modId").to_string(),
        name: text(v, "displayName"),
        version_number: text(v, "displayName"),
        game_versions,
        loaders: loaders.iter().map(|l| l.to_lowercase()).collect(),
        channel: channel(num(v, "releaseType")),
        downloads: num(v, "downloadCount"),
        published: opt_text(v, "fileDate"),
        filename: text(v, "fileName"),
        size: num(v, "fileLength"),
        dependencies: items(v, "dependencies")
            .iter()
            .map(|d| ModDependency {
                project_id: Some(num(d, "modId").to_string()),
                kind: relation(num(d, "relationType")),
            })
            .collect(),
        // Автор мог запретить стороннюю загрузку — тогда downloadUrl пустой.
        downloadable: opt_text(v, "downloadUrl").is_some(),
    }
}

pub fn category(v: &Value) -> Category {
    Category {
        // Фильтр CurseForge принимает числовой id, не slug: он и есть значение.
        name: num(v, "id").to_string(),
        display: text(v, "name"),
        icon: opt_text(v, "iconUrl"),
    }
}
