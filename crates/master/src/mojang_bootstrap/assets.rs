//! Скачивание индекса ассетов и всех объектов. Объекты кешируются по SHA1,
//! поэтому для второй версии MC с пересекающимися ассетами почти ничего не качается.

use super::BootstrapCtx;
use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use schema::ArtifactKind;

use serde_json::Value;

const RESOURCES_BASE: &str = "https://resources.download.minecraft.net";

pub async fn bootstrap_assets(
    ctx: &BootstrapCtx<'_>,
    index_name: &str,
    index_url: &str,
    index_sha1: Option<&str>,
) -> Result<()> {
    let http = ctx.state.http();

    // Индекс ассетов.
    let index_path = format!("assets/indexes/{index_name}.json");
    ctx.logf(format!("скачивание индекса ассетов {index_name}"));
    ctx.register(
        &index_path,
        ArtifactKind::AssetIndex,
        "both",
        index_url,
        index_sha1,
        "asset_index",
    )
    .await?;

    // Сам индекс читаем, чтобы перечислить объекты.
    let index: Value = http.get(index_url).send().await?.json().await?;
    let objects = match index["objects"].as_object() {
        Some(o) => o,
        None => return Ok(()),
    };

    // Собираем уникальные хеши (несколько ключей могут указывать на один хеш).
    let mut hashes: Vec<(String, u64)> = Vec::new();
    for (_name, obj) in objects {
        if let Some(hash) = obj["hash"].as_str() {
            let size = obj["size"].as_u64().unwrap_or(0);
            hashes.push((hash.to_string(), size));
        }
    }
    hashes.sort();
    hashes.dedup_by(|a, b| a.0 == b.0);

    ctx.logf(format!("скачивание {} ассет-объектов", hashes.len()));

    // Параллельно (с ограничением), но регистрация в build_files идёт последовательно
    // через ctx.register, который пишет в БД — DashMap-пул справляется.
    let results = stream::iter(hashes.into_iter().map(|(hash, size)| async move {
        let sub = &hash[0..2];
        let url = format!("{RESOURCES_BASE}/{sub}/{hash}");
        let path = format!("assets/objects/{sub}/{hash}");
        ctx.register(
            &path,
            ArtifactKind::Asset,
            "both",
            &url,
            Some(&hash),
            "asset",
        )
        .await
        .with_context(|| format!("ассет {hash} ({size} б)"))
    }))
    .buffer_unordered(12)
    .collect::<Vec<_>>()
    .await;

    for r in results {
        r?;
    }
    Ok(())
}
