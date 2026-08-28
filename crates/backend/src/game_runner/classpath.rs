use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest, Modloader};
use std::path::Path;

pub fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

/// Собрать classpath из манифеста.
///
/// Для Forge/NeoForge ванильный client jar и patched loader client jar не
/// добавляются в legacy classpath: FML сам собирает game layer через
/// ProductionClientProvider.
pub fn build_classpath(instance_dir: &Path, manifest: &BuildManifest) -> String {
    let mut game_jar = None;
    let mut libs = Vec::new();
    let forge_like = is_forge_like(manifest);

    for f in &manifest.verified_files {
        // Natives лежат в сборке под все платформы; чужие не скачивались, и
        // ссылки на них в classpath указывали бы в пустоту.
        if !f.side.needed_on_client() || !f.matches_platform() {
            continue;
        }
        let kind = manifest.kind_of(&f.path);
        let Some(path) = safe_join(instance_dir, &f.path) else {
            continue;
        };
        let path_str = path.to_string_lossy().into_owned();
        match kind {
            ArtifactKind::ClientJar if !forge_like => game_jar = Some(path_str),
            ArtifactKind::Library if forge_like && is_loader_client_path(&f.path) => {}
            ArtifactKind::Library | ArtifactKind::Native => libs.push(path_str),
            _ => {}
        }
    }

    let mut entries = Vec::new();
    if let Some(gj) = game_jar {
        entries.push(gj);
    }
    entries.extend(libs);
    entries.join(classpath_separator())
}

pub fn primary_game_artifact(instance_dir: &Path, manifest: &BuildManifest) -> Option<String> {
    loader_client_path(manifest)
        .or_else(|| {
            manifest
                .verified_files
                .iter()
                .find(|f| manifest.kind_of(&f.path) == ArtifactKind::ClientJar)
                .map(|f| f.path.clone())
        })
        .and_then(|path| safe_join(instance_dir, &path))
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn loader_client_name(manifest: &BuildManifest) -> Option<String> {
    loader_client_path(manifest)
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

pub fn remove_from_ignore_list(arg: &mut String, filename: Option<&str>) {
    let Some(filename) = filename else {
        return;
    };
    let Some(values) = arg.strip_prefix("-DignoreList=") else {
        return;
    };
    if !values.contains(filename) {
        return;
    }
    let kept = values
        .split(',')
        .filter(|item| *item != filename)
        .collect::<Vec<_>>()
        .join(",");
    *arg = format!("-DignoreList={kept}");
}

pub fn standard_ignore_list() -> &'static str {
    "-DignoreList=bootstraplauncher,cpw.mods.securejarhandler,asm-7.2.jar,asm-commons-7.2.jar,asm-tree-7.2.jar,asm-util-7.2.jar,asm-analysis-7.2.jar"
}

fn loader_client_path(manifest: &BuildManifest) -> Option<String> {
    manifest
        .verified_files
        .iter()
        .find(|f| is_loader_client_path(&f.path))
        .map(|f| f.path.clone())
}

fn is_loader_client_path(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    name.ends_with("-client.jar") && is_loader_path(path)
}

fn is_loader_path(path: &str) -> bool {
    path.contains("/neoforged/neoforge/") || path.contains("/minecraftforge/forge/")
}

fn is_forge_like(manifest: &BuildManifest) -> bool {
    matches!(manifest.modloader, Modloader::Forge | Modloader::NeoForge)
}
