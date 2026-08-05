//! Ванильный bootstrap: version_manifest → version.json → client.jar, библиотеки,
//! natives, затем assets и java.

use super::platform::Platform;
use super::{assets, java, BootstrapCtx};
use anyhow::{anyhow, Context, Result};
use schema::ArtifactKind;
use serde_json::Value;

const VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

/// Скачать и зарегистрировать ванильные артефакты; вернуть version.json (нужен Forge).
pub async fn bootstrap_vanilla(ctx: &mut BootstrapCtx<'_>) -> Result<Value> {
    let http = ctx.state.http();

    // 1. Манифест версий → URL нужной версии.
    ctx.logf("получение version_manifest");
    let manifest: Value = http.get(VERSION_MANIFEST).send().await?.json().await?;
    let versions = manifest["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("version_manifest без versions"))?;
    let entry = versions
        .iter()
        .find(|v| v["id"].as_str() == Some(ctx.mc_version.as_str()))
        .ok_or_else(|| anyhow!("версия {} не найдена в манифесте Mojang", ctx.mc_version))?;
    let version_url = entry["url"]
        .as_str()
        .ok_or_else(|| anyhow!("нет url версии"))?;

    // 2. version.json.
    let vj: Value = http.get(version_url).send().await?.json().await?;

    // 3. client.jar.
    if let Some(client) = vj["downloads"]["client"].as_object() {
        let url = client["url"].as_str().unwrap_or_default();
        let sha1 = client["sha1"].as_str();
        ctx.logf("скачивание client.jar");
        ctx.register(
            &format!("versions/{}/client.jar", ctx.mc_version),
            ArtifactKind::ClientJar,
            "both",
            url,
            sha1,
            "client",
        )
        .await?;
    }

    // 4. Библиотеки и natives — под каждую платформу клиента.
    //
    // Раньше и здесь, и в java стояла платформа мастера: сборка уезжала на
    // Windows с линуксовым рантаймом, и JVM не стартовала. Пути natives уже
    // различаются классификатором Mojang, так что коллизий не будет.
    if let Some(libs) = vj["libraries"].as_array() {
        ctx.logf(format!("обработка {} библиотек", libs.len()));
        for platform in Platform::ALL {
            ctx.platform = platform;
            for lib in libs {
                process_library(ctx, lib).await?;
            }
        }
    }

    // mainClass и аргументы.
    ctx.main_class = vj["mainClass"].as_str().unwrap_or_default().to_string();
    extract_arguments(ctx, &vj);

    // 6-7. Assets.
    let assets_index_name = vj["assets"].as_str().unwrap_or(&ctx.mc_version).to_string();
    ctx.assets_index_name = assets_index_name.clone();
    if let Some(ai) = vj["assetIndex"].as_object() {
        let url = ai["url"].as_str().unwrap_or_default();
        let sha1 = ai["sha1"].as_str();
        assets::bootstrap_assets(ctx, &assets_index_name, url, sha1).await?;
    }

    // 8. Java — свой рантайм каждой платформе, они лежат в runtime/{платформа}/.
    if let Some(jv) = vj["javaVersion"]["component"].as_str() {
        for platform in Platform::ALL {
            ctx.platform = platform;
            java::bootstrap_java(ctx, jv).await?;
        }
    }

    Ok(vj)
}

/// Обработать одну библиотеку: проверить rules, скачать artifact и natives.
async fn process_library(ctx: &BootstrapCtx<'_>, lib: &Value) -> Result<()> {
    if !rules_allow(lib.get("rules"), ctx.platform) {
        return Ok(());
    }

    let name = lib["name"].as_str().unwrap_or_default();
    let is_native = name.contains(":natives") || name.contains("natives-");

    // Современный формат: downloads.artifact.
    if let Some(artifact) = lib["downloads"]["artifact"].as_object() {
        if let Some(path) = artifact["path"].as_str() {
            let url = artifact["url"].as_str().unwrap_or_default();
            let sha1 = artifact["sha1"].as_str();
            if !url.is_empty() {
                let kind = if is_native {
                    ArtifactKind::Native
                } else {
                    ArtifactKind::Library
                };
                ctx.register(
                    &format!("libraries/{path}"),
                    kind,
                    "both",
                    url,
                    sha1,
                    "library",
                )
                .await
                .with_context(|| format!("библиотека {name}"))?;
            }
        }
    }

    // Легаси natives через classifiers.
    if let Some(natives) = lib.get("natives").and_then(|n| n.as_object()) {
        let os_key = match ctx.platform.mojang_os_name() {
            "osx" => "osx",
            "windows" => "windows",
            _ => "linux",
        };
        if let Some(classifier) = natives.get(os_key).and_then(|c| c.as_str()) {
            // classifier может содержать ${arch}
            let classifier = classifier.replace("${arch}", "64");
            if let Some(c) = lib["downloads"]["classifiers"][&classifier].as_object() {
                if let Some(path) = c["path"].as_str() {
                    let url = c["url"].as_str().unwrap_or_default();
                    let sha1 = c["sha1"].as_str();
                    if !url.is_empty() {
                        ctx.register(
                            &format!("libraries/{path}"),
                            ArtifactKind::Native,
                            "both",
                            url,
                            sha1,
                            "native",
                        )
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Проверить rules библиотеки/аргумента для текущей платформы.
/// Feature-правила (is_demo_user и т.п.) считаются не выполненными.
pub fn rules_allow(rules: Option<&Value>, platform: Platform) -> bool {
    let Some(rules) = rules.and_then(|r| r.as_array()) else {
        return true;
    };
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        let action_allow = rule["action"].as_str() == Some("allow");

        // Если есть feature-условия — мы их не включаем, правило не применяется.
        if rule.get("features").is_some() {
            continue;
        }

        let os_matches = match rule.get("os").and_then(|o| o.as_object()) {
            None => true,
            Some(os) => {
                let name_ok = os
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == platform.mojang_os_name())
                    .unwrap_or(true);
                let arch_ok = os
                    .get("arch")
                    .and_then(|a| a.as_str())
                    .map(|a| a == platform.mojang_arch())
                    .unwrap_or(true);
                name_ok && arch_ok
            }
        };

        if os_matches {
            allowed = action_allow;
        }
    }
    allowed
}

/// Извлечь jvm/game аргументы (с placeholder'ами) из version.json.
pub fn extract_arguments(ctx: &mut BootstrapCtx<'_>, vj: &Value) {
    if let Some(args) = vj["arguments"].as_object() {
        ctx.jvm_args = extract_arg_list(args.get("jvm"), ctx.platform);
        ctx.game_args = extract_arg_list(args.get("game"), ctx.platform);
    } else if let Some(legacy) = vj["minecraftArguments"].as_str() {
        // Версии до 1.13.
        ctx.game_args = legacy.split_whitespace().map(String::from).collect();
        ctx.jvm_args = vec![
            "-Djava.library.path=${natives_directory}".to_string(),
            "-cp".to_string(),
            "${classpath}".to_string(),
        ];
    }
}

fn extract_arg_list(list: Option<&Value>, platform: Platform) -> Vec<String> {
    let Some(arr) = list.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        match item {
            Value::String(s) => out.push(s.clone()),
            Value::Object(obj) => {
                if rules_allow(obj.get("rules"), platform) {
                    match &item["value"] {
                        Value::String(s) => out.push(s.clone()),
                        Value::Array(vs) => {
                            for v in vs {
                                if let Some(s) = v.as_str() {
                                    out.push(s.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}
