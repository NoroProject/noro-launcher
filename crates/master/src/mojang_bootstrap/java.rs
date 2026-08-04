//! Скачивание Java-рантайма Mojang под платформу клиента. Кешируется по SHA1.

use super::BootstrapCtx;
use anyhow::{anyhow, Context, Result};
use futures::stream::{self, StreamExt};
use schema::ArtifactKind;
use serde_json::Value;

/// Стабильный «манифест манифестов» java-рантаймов Mojang.
const JAVA_ALL_MANIFEST: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

pub async fn bootstrap_java(ctx: &BootstrapCtx<'_>, component: &str) -> Result<()> {
    let http = ctx.state.http();
    ctx.logf(format!("получение java-рантайма {component}"));

    let all: Value = http.get(JAVA_ALL_MANIFEST).send().await?.json().await?;
    let os_key = ctx.platform.java_os_key();

    let comp_list = all
        .get(os_key)
        .and_then(|o| o.get(component))
        .and_then(|c| c.as_array())
        .filter(|a| !a.is_empty());

    // Если для платформы нет нужного компонента — попробуем дефолтный gamma.
    let comp_list = match comp_list {
        Some(list) => list,
        None => all
            .get(os_key)
            .and_then(|o| o.get("java-runtime-gamma"))
            .and_then(|c| c.as_array())
            .filter(|a| !a.is_empty())
            .ok_or_else(|| anyhow!("нет java-рантайма для платформы {os_key}/{component}"))?,
    };

    let manifest_url = comp_list[0]["manifest"]["url"]
        .as_str()
        .ok_or_else(|| anyhow!("нет url манифеста java"))?;

    let manifest: Value = http.get(manifest_url).send().await?.json().await?;
    let files = manifest["files"]
        .as_object()
        .ok_or_else(|| anyhow!("манифест java без files"))?;

    // Собираем задачи скачивания файлов (директории/симлинки пропускаем).
    let mut tasks: Vec<(String, String, Option<String>)> = Vec::new();
    for (key, entry) in files {
        if entry["type"].as_str() != Some("file") {
            continue;
        }
        let raw = &entry["downloads"]["raw"];
        let url = match raw["url"].as_str() {
            Some(u) => u.to_string(),
            None => continue,
        };
        let sha1 = raw["sha1"].as_str().map(String::from);
        let path = format!("runtime/{key}");
        tasks.push((path, url, sha1));
    }

    ctx.logf(format!("скачивание {} файлов java", tasks.len()));

    let results = stream::iter(tasks.into_iter().map(|(path, url, sha1)| async move {
        ctx.register(
            &path,
            ArtifactKind::Java,
            "both",
            &url,
            sha1.as_deref(),
            "java",
        )
        .await
        .with_context(|| format!("java-файл {path}"))
    }))
    .buffer_unordered(12)
    .collect::<Vec<_>>()
    .await;

    for r in results {
        r?;
    }
    Ok(())
}
