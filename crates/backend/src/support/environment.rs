//! Окружение: то, чего нет в логах, но без чего они не читаются.

use schema::BuildManifest;
use std::path::Path;

/// Описание одним текстом — его же видит игрок в предпросмотре.
pub async fn describe(
    instance_dir: &Path,
    manifest: Option<&BuildManifest>,
    enabled_optional: &[String],
) -> String {
    let mut out = String::new();

    out.push_str(&format!("launcher: {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str(&format!(
        "os: {} {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));

    if let Some(m) = manifest {
        out.push_str(&format!(
            "build: {} ({} {})\n",
            m.version,
            m.modloader.as_str(),
            m.mc_version
        ));
        out.push_str(&format!("build_id: {}\n", m.build_id));
        out.push_str(&format!("server_id: {}\n", m.server_id));
        out.push_str(&format!(
            "memory: {}–{} МБ\n",
            m.recommended_client_settings.memory_min_mb,
            m.recommended_client_settings.memory_max_mb
        ));
    }

    out.push_str(&format!(
        "optional_enabled: {}\n",
        if enabled_optional.is_empty() {
            "—".to_string()
        } else {
            enabled_optional.join(", ")
        }
    ));

    out.push_str(&format!("mods:\n{}", list_mods(instance_dir).await));
    out
}

/// Что лежит в `mods/` на самом деле. Не то же, что список из манифеста:
/// расхождение между ними и есть половина разбираемых случаев.
async fn list_mods(instance_dir: &Path) -> String {
    let Ok(mut entries) = tokio::fs::read_dir(instance_dir.join("mods")).await else {
        return "  (каталог mods не читается)\n".into();
    };
    let mut names = Vec::new();
    while let Ok(Some(e)) = entries.next_entry().await {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
            let size = e.metadata().await.map(|m| m.len()).unwrap_or(0);
            names.push(format!("  {name} ({size} B)"));
        }
    }
    names.sort();
    if names.is_empty() {
        return "  (пусто)\n".into();
    }
    format!("{}\n", names.join("\n"))
}
