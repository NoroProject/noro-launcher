//! Преобразование строковых значений из БД в типы схемы и обратно.

use schema::{ArtifactKind, FileSide};

/// Преобразовать строковый kind из БД в ArtifactKind.
pub fn kind_from_str(s: &str) -> ArtifactKind {
    match s {
        "client_jar" => ArtifactKind::ClientJar,
        "library" => ArtifactKind::Library,
        "runtime" => ArtifactKind::Runtime,
        "native" => ArtifactKind::Native,
        "asset" => ArtifactKind::Asset,
        "asset_index" => ArtifactKind::AssetIndex,
        "java" => ArtifactKind::Java,
        "mod" => ArtifactKind::Mod,
        "config" => ArtifactKind::Config,
        _ => ArtifactKind::Other,
    }
}

pub fn kind_to_str(k: ArtifactKind) -> &'static str {
    match k {
        ArtifactKind::ClientJar => "client_jar",
        ArtifactKind::Library => "library",
        ArtifactKind::Runtime => "runtime",
        ArtifactKind::Native => "native",
        ArtifactKind::Asset => "asset",
        ArtifactKind::AssetIndex => "asset_index",
        ArtifactKind::Java => "java",
        ArtifactKind::Mod => "mod",
        ArtifactKind::Config => "config",
        ArtifactKind::Other => "other",
    }
}

pub fn side_from_str(s: &str) -> FileSide {
    match s {
        "client" => FileSide::Client,
        "server" => FileSide::Server,
        _ => FileSide::Both,
    }
}
