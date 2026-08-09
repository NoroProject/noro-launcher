//! Modrinth JSON → нормализованные типы каталога.

use super::json::{items, num, opt_text, str_list, text};
use super::types::{Category, ModDependency, ModHit, ModProject, ModVersion, Provider};
use serde_json::Value;

pub fn hit(v: &Value) -> ModHit {
    // В поиске проект зовётся project_id, в карточке проекта — просто id.
    let project_id = opt_text(v, "project_id").unwrap_or_else(|| text(v, "id"));
    let slug = opt_text(v, "slug").unwrap_or_else(|| project_id.clone());
    ModHit {
        provider: Provider::Modrinth,
        page_url: Some(format!("https://modrinth.com/mod/{slug}")),
        project_id,
        title: text(v, "title"),
        description: opt_text(v, "description").unwrap_or_else(|| text(v, "summary")),
        author: text(v, "author"),
        icon_url: opt_text(v, "icon_url"),
        downloads: num(v, "downloads"),
        follows: num(v, "follows").max(num(v, "followers")),
        categories: str_list(v, "categories"),
        client_side: text(v, "client_side"),
        server_side: text(v, "server_side"),
        updated: opt_text(v, "date_modified").or_else(|| opt_text(v, "updated")),
        slug,
    }
}

pub fn project(v: &Value) -> ModProject {
    ModProject {
        hit: hit(v),
        body: text(v, "body"),
        gallery: items(v, "gallery")
            .iter()
            .filter_map(|g| opt_text(g, "url"))
            .collect(),
        source_url: opt_text(v, "source_url"),
        issues_url: opt_text(v, "issues_url"),
        wiki_url: opt_text(v, "wiki_url"),
        license: opt_text(&v["license"], "name").or_else(|| opt_text(&v["license"], "id")),
        game_versions: str_list(v, "game_versions"),
        loaders: str_list(v, "loaders"),
    }
}

pub fn version(v: &Value) -> Option<ModVersion> {
    // Primary-файл — то, что ставится; остальные это исходники и javadoc.
    let files = items(v, "files");
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())?;

    Some(ModVersion {
        provider: Provider::Modrinth,
        id: text(v, "id"),
        project_id: text(v, "project_id"),
        name: text(v, "name"),
        version_number: text(v, "version_number"),
        game_versions: str_list(v, "game_versions"),
        loaders: str_list(v, "loaders"),
        channel: text(v, "version_type"),
        downloads: num(v, "downloads"),
        published: opt_text(v, "date_published"),
        filename: text(file, "filename"),
        size: num(file, "size"),
        dependencies: items(v, "dependencies")
            .iter()
            .map(|d| ModDependency {
                project_id: opt_text(d, "project_id"),
                kind: text(d, "dependency_type"),
            })
            .collect(),
        downloadable: true,
    })
}

pub fn category(v: &Value) -> Category {
    let name = text(v, "name");
    Category {
        display: pretty(&name),
        // Modrinth кладёт сюда готовый inline SVG, а не ссылку.
        icon: opt_text(v, "icon"),
        name,
    }
}

/// `worldgen` → `Worldgen`, `game-mechanics` → `Game mechanics`.
fn pretty(name: &str) -> String {
    let spaced = name.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => spaced,
    }
}
