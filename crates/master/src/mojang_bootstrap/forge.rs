//! Bootstrap Forge / NeoForge.
//!
//! Современные (1.13+) installer'ы не запускаются на клиенте: мастер скачивает
//! installer.jar, читает `install_profile.json` и `version.json`, материализует
//! библиотеки и встроенные файлы во временную maven-директорию, запускает
//! post-processors (нужна Java на мастере), а затем кладёт получившиеся jar в
//! FileStore. Клиент получает уже готовый набор без всякого установщика.

use super::maven::maven_to_path;
use super::BootstrapCtx;
use anyhow::{anyhow, bail, Context, Result};
use schema::{ArtifactKind, Modloader};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

pub async fn bootstrap_forge(
    ctx: &mut BootstrapCtx<'_>,
    loader: Modloader,
    _vanilla_vj: &Value,
) -> Result<()> {
    let version = ctx.modloader_version.clone().ok_or_else(|| {
        anyhow!(
            "для {} нужна modloader_version (версия установщика)",
            loader.as_str()
        )
    })?;

    let installer_url = installer_url(loader, &ctx.mc_version, &version);
    ctx.logf(format!("скачивание installer: {installer_url}"));

    let installer_bytes = ctx
        .state
        .http()
        .get(&installer_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    // Рабочая директория.
    let work = ctx
        .state
        .config
        .data_dir
        .join("tmp")
        .join(format!("forge-{}", ctx.base_build_id));
    let libs_dir = work.join("libraries");
    let _ = tokio::fs::remove_dir_all(&work).await;
    tokio::fs::create_dir_all(&libs_dir).await?;

    // Делаем пути абсолютными, так как процессоры меняют current_dir,
    // и относительные пути из config.data_dir сломают classpath.
    let work = tokio::fs::canonicalize(&work).await.unwrap_or(work);
    let libs_dir = tokio::fs::canonicalize(&libs_dir).await.unwrap_or(libs_dir);

    // Читаем install_profile.json и version.json из installer.
    let (install_profile, version_json, embedded) =
        read_installer(&installer_bytes).context("чтение installer")?;

    // mainClass и аргументы из version.json модлоадера.
    if let Some(mc) = version_json["mainClass"].as_str() {
        ctx.main_class = mc.to_string();
    }
    merge_arguments(ctx, &version_json);

    // Материализуем встроенные файлы installer'а в work/.
    write_embedded(&work, &embedded).await?;

    // Скачиваем/материализуем библиотеки install_profile (тулинг процессоров).
    materialize_libraries(ctx, &install_profile, &libs_dir, &embedded).await?;
    // И библиотеки version.json (то, что нужно для запуска).
    materialize_libraries(ctx, &version_json, &libs_dir, &embedded).await?;

    // Нужен ванильный client.jar как вход процессоров — он уже в FileStore.
    let vanilla_client = stage_vanilla_client(ctx, &work).await?;

    // Резолвим карту data (client-сторона).
    let data = resolve_data(&install_profile, &libs_dir, &work, &embedded)?;
    extend_ignore_list(&mut ctx.jvm_args);

    // Запускаем процессоры.
    run_processors(
        ctx,
        &install_profile,
        &libs_dir,
        &work,
        &vanilla_client,
        &data,
    )
    .await?;

    register_processed_outputs(ctx, &data, &libs_dir).await?;

    // version.json libs сначала (Library), потом runtime — чтобы NeoForge universal
    // и другие JAR'ы, загружаемые PathBasedLocator'ом, остались ArtifactKind::Runtime
    // и не попали в legacy classpath (иначе дублируются в module layer).
    register_version_libraries(ctx, &version_json, &libs_dir).await?;
    register_runtime_libraries(ctx, &install_profile, &libs_dir).await?;

    // Очистка рабочей директории.
    let _ = tokio::fs::remove_dir_all(&work).await;
    Ok(())
}

fn installer_url(loader: Modloader, mc: &str, version: &str) -> String {
    match loader {
        Modloader::NeoForge => format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar"
        ),
        _ => format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{version}/forge-{mc}-{version}-installer.jar"
        ),
    }
}

/// Распаковать ключевые JSON и собрать карту встроенных файлов (path → bytes).
fn read_installer(bytes: &[u8]) -> Result<(Value, Value, HashMap<String, Vec<u8>>)> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;

    let mut install_profile = Value::Null;
    let mut version_json = Value::Null;
    let mut embedded: HashMap<String, Vec<u8>> = HashMap::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        match name.as_str() {
            "install_profile.json" => {
                install_profile = serde_json::from_slice(&buf)?;
            }
            "version.json" => {
                version_json = serde_json::from_slice(&buf)?;
            }
            _ => {
                embedded.insert(name, buf);
            }
        }
    }

    if install_profile.is_null() {
        bail!("install_profile.json не найден в installer");
    }
    // У некоторых installer'ов version.json лежит по пути из install_profile["json"].
    if version_json.is_null() {
        if let Some(json_path) = install_profile["json"].as_str() {
            let key = json_path.trim_start_matches('/');
            if let Some(data) = embedded.get(key) {
                version_json = serde_json::from_slice(data)?;
            }
        }
    }
    Ok((install_profile, version_json, embedded))
}

fn merge_arguments(ctx: &mut BootstrapCtx<'_>, vj: &Value) {
    if let Some(jvm) = vj["arguments"]["jvm"].as_array() {
        for a in jvm {
            if let Some(s) = a.as_str() {
                ctx.jvm_args.push(schema::ManifestArg::new_string(s));
            }
        }
    }
    if let Some(game) = vj["arguments"]["game"].as_array() {
        for a in game {
            if let Some(s) = a.as_str() {
                ctx.game_args.push(schema::ManifestArg::new_string(s));
            }
        }
    }
}

fn extend_ignore_list(jvm_args: &mut Vec<schema::ManifestArg>) {
    // client.jar = vanilla obfuscated client, must not enter the game module layer.
    let names: Vec<&str> = vec!["client.jar"];

    // Список приходит из version.json обычной строкой, без rules, — условным
    // он не бывает, поэтому ищем только среди строк.
    let existing = jvm_args.iter_mut().find_map(|arg| match arg {
        schema::ManifestArg::String(s) if s.starts_with("-DignoreList=") => Some(s),
        _ => None,
    });

    match existing {
        Some(list) => {
            for name in &names {
                if !list.contains(*name) {
                    list.push(',');
                    list.push_str(name);
                }
            }
        }
        None => jvm_args.push(schema::ManifestArg::new_string(format!(
            "-DignoreList={}",
            names.join(",")
        ))),
    }
}

/// Записать встроенные в installer файлы (data/*, maven/*) во временную директорию.
async fn write_embedded(work: &Path, embedded: &HashMap<String, Vec<u8>>) -> Result<()> {
    for (name, bytes) in embedded {
        // maven/<path> → libraries/<path>, остальное — как есть.
        let rel = name
            .strip_prefix("maven/")
            .map(|m| format!("libraries/{m}"))
            .unwrap_or_else(|| name.clone());
        let dst = work.join(&rel);
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&dst, bytes).await?;
    }
    Ok(())
}

/// Скачать (или взять из встроенных) все библиотеки из массива `libraries`.
async fn materialize_libraries(
    ctx: &BootstrapCtx<'_>,
    profile: &Value,
    libs_dir: &Path,
    embedded: &HashMap<String, Vec<u8>>,
) -> Result<()> {
    let Some(libs) = profile["libraries"].as_array() else {
        return Ok(());
    };
    for lib in libs {
        let Some(name) = lib["name"].as_str() else {
            continue;
        };
        // Prefer downloads.artifact.path over maven_to_path(name): some loaders
        // (NeoForge 21.x) list "net.neoforged:neoforge:21.1.233" without the
        // :universal classifier in `name`, but the artifact.path correctly points
        // to neoforge-21.1.233-universal.jar. Using maven_to_path here would save
        // the jar under the wrong filename and later register it as a Library.
        let rel = lib["downloads"]["artifact"]["path"]
            .as_str()
            .map(str::to_string)
            .or_else(|| maven_to_path(name));
        let Some(rel) = rel else { continue };

        let dst = libs_dir.join(&rel);
        if dst.exists() {
            continue;
        }
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 1) уже извлечён из installer (maven/<rel>)?
        if let Some(data) = embedded.get(&format!("maven/{rel}")) {
            tokio::fs::write(&dst, data).await?;
            continue;
        }
        // 2) downloads.artifact.url?
        let url = lib["downloads"]["artifact"]["url"]
            .as_str()
            .filter(|u| !u.is_empty());
        let url = match url {
            Some(u) => u.to_string(),
            None => {
                // 3) собрать из base url (часто пусто → дефолтные maven).
                let base = lib["url"]
                    .as_str()
                    .unwrap_or("https://libraries.minecraft.net/");
                format!("{}/{}", base.trim_end_matches('/'), rel)
            }
        };
        let resp = ctx.state.http().get(&url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let bytes = r.bytes().await?;
                tokio::fs::write(&dst, &bytes).await?;
            }
            _ => {
                ctx.logf(format!(
                    "предупреждение: не удалось получить {name} ({url})"
                ));
            }
        }
    }
    Ok(())
}

async fn stage_vanilla_client(ctx: &BootstrapCtx<'_>, work: &Path) -> Result<PathBuf> {
    let path = format!("versions/{}/client.jar", ctx.mc_version);
    let files = crate::db::base_build_files(&ctx.state.db, ctx.base_build_id).await?;
    let file = files.into_iter().find(|f| f.path == path).ok_or_else(|| {
        anyhow!("ванильный client.jar не найден (запусти vanilla bootstrap раньше)")
    })?;
    let src = ctx.state.files.path_for(&file.sha1);
    let dst = work.join("client.jar");
    tokio::fs::copy(&src, &dst).await?;
    Ok(dst)
}

/// Резолв карты `data`: [maven] → путь в libs_dir, /path → файл в work, иначе литерал.
fn resolve_data(
    profile: &Value,
    libs_dir: &Path,
    work: &Path,
    embedded: &HashMap<String, Vec<u8>>,
) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    if let Some(data) = profile["data"].as_object() {
        for (key, val) in data {
            let raw = val["client"].as_str().unwrap_or_default();
            let resolved = resolve_token(raw, libs_dir, work, embedded);
            out.insert(key.clone(), resolved);
        }
    }
    Ok(out)
}

fn resolve_token(
    raw: &str,
    libs_dir: &Path,
    work: &Path,
    _embedded: &HashMap<String, Vec<u8>>,
) -> String {
    if let Some(coord) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        if let Some(rel) = maven_to_path(coord) {
            return libs_dir.join(rel).to_string_lossy().into_owned();
        }
    }
    if let Some(path) = raw.strip_prefix('/') {
        return work.join(path).to_string_lossy().into_owned();
    }
    raw.to_string()
}

/// Запустить процессоры install_profile по порядку.
async fn run_processors(
    ctx: &BootstrapCtx<'_>,
    profile: &Value,
    libs_dir: &Path,
    work: &Path,
    vanilla_client: &Path,
    data: &HashMap<String, String>,
) -> Result<()> {
    let Some(processors) = profile["processors"].as_array() else {
        return Ok(());
    };
    let java = super::processor_java::find(ctx).await?;

    for (idx, proc) in processors.iter().enumerate() {
        // Пропустить процессоры не для client-стороны.
        if let Some(sides) = proc["sides"].as_array() {
            let for_client = sides.iter().any(|s| s.as_str() == Some("client"));
            if !for_client {
                continue;
            }
        }

        let jar_coord = proc["jar"]
            .as_str()
            .ok_or_else(|| anyhow!("процессор без jar"))?;
        let jar_path =
            libs_dir.join(maven_to_path(jar_coord).ok_or_else(|| anyhow!("плохой jar coord"))?);
        let main_class = read_jar_main_class(&jar_path)
            .with_context(|| format!("Main-Class из {}", jar_path.display()))?;

        // classpath = jar + classpath-зависимости.
        let mut cp = vec![jar_path.to_string_lossy().into_owned()];
        if let Some(cps) = proc["classpath"].as_array() {
            for c in cps {
                if let Some(coord) = c.as_str() {
                    if let Some(rel) = maven_to_path(coord) {
                        cp.push(libs_dir.join(rel).to_string_lossy().into_owned());
                    }
                }
            }
        }

        // Аргументы с подстановкой.
        let mut args: Vec<String> = Vec::new();
        if let Some(arglist) = proc["args"].as_array() {
            for a in arglist {
                if let Some(s) = a.as_str() {
                    args.push(substitute_arg(s, data, libs_dir, work, vanilla_client));
                }
            }
        }

        ctx.logf(format!(
            "процессор {}/{}: {main_class} (jar: {})",
            idx + 1,
            processors.len(),
            jar_path.display()
        ));
        let sep = if cfg!(windows) { ";" } else { ":" };
        let cp_joined = cp.join(sep);
        ctx.logf(format!("classpath: {cp_joined}"));

        let mut cmd = tokio::process::Command::new(&java);
        cmd.arg("-cp").arg(&cp_joined).arg(&main_class).args(&args);
        cmd.current_dir(work);
        let output = cmd
            .output()
            .await
            .with_context(|| format!("запуск процессора {main_class} через {}", java.display()))?;
        if !output.status.success() {
            bail!(
                "процессор {main_class} завершился с ошибкой:\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
    Ok(())
}

fn substitute_arg(
    arg: &str,
    data: &HashMap<String, String>,
    libs_dir: &Path,
    _work: &Path,
    vanilla_client: &Path,
) -> String {
    let mut s = arg.to_string();
    // {KEY} из data.
    for (k, v) in data {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    // Спец-плейсхолдеры.
    s = s.replace("{MINECRAFT_JAR}", &vanilla_client.to_string_lossy());
    s = s.replace("{SIDE}", "client");
    s = s.replace("{LIBRARY_DIR}", &libs_dir.to_string_lossy());
    // [maven] координата как аргумент.
    if let Some(coord) = s.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
        if let Some(rel) = maven_to_path(coord) {
            return libs_dir.join(rel).to_string_lossy().into_owned();
        }
    }
    s
}

/// Прочитать Main-Class из MANIFEST.MF jar-файла.
fn read_jar_main_class(jar: &Path) -> Result<String> {
    let file = std::fs::File::open(jar).with_context(|| format!("открытие {}", jar.display()))?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut manifest = zip.by_name("META-INF/MANIFEST.MF")?;
    let mut content = String::new();
    manifest.read_to_string(&mut content)?;
    for line in content.lines() {
        if let Some(mc) = line.strip_prefix("Main-Class:") {
            return Ok(mc.trim().to_string());
        }
    }
    bail!("Main-Class не найден в {}", jar.display())
}

async fn register_processed_outputs(
    ctx: &BootstrapCtx<'_>,
    data: &HashMap<String, String>,
    libs_dir: &Path,
) -> Result<()> {
    for key in ["MC_SRG", "MC_EXTRA", "PATCHED"] {
        let Some(path) = data.get(key) else {
            continue;
        };
        register_processed_output(ctx, key, Path::new(path), libs_dir).await?;
    }
    Ok(())
}

async fn register_processed_output(
    ctx: &BootstrapCtx<'_>,
    name: &str,
    path: &Path,
    libs_dir: &Path,
) -> Result<()> {
    if !path.exists() {
        bail!(
            "процессоры завершились, но {name} jar не найден: {}",
            path.display()
        );
    }
    let rel = path.strip_prefix(libs_dir).with_context(|| {
        format!(
            "{name} jar должен лежать внутри libraries: {}",
            path.display()
        )
    })?;
    let data = tokio::fs::read(path)
        .await
        .with_context(|| format!("чтение {name} jar {}", path.display()))?;
    ctx.register_bytes(
        &format!("libraries/{}", rel.to_string_lossy()),
        ArtifactKind::Runtime,
        "both",
        &data,
        "library",
    )
    .await?;
    Ok(())
}

async fn register_runtime_libraries(
    ctx: &BootstrapCtx<'_>,
    profile: &Value,
    libs_dir: &Path,
) -> Result<()> {
    let Some(libs) = profile["libraries"].as_array() else {
        return Ok(());
    };
    for lib in libs {
        let Some(name) = lib["name"].as_str() else {
            continue;
        };
        let is_loader_universal = (name.starts_with("net.neoforged:neoforge:")
            || name.starts_with("net.minecraftforge:forge:"))
            && name.ends_with(":universal");
        if !is_loader_universal {
            continue;
        }
        let rel = lib["downloads"]["artifact"]["path"]
            .as_str()
            .map(str::to_string)
            .or_else(|| maven_to_path(name));
        let Some(rel) = rel else { continue };
        let path = libs_dir.join(&rel);
        if !path.exists() {
            bail!("runtime library не материализована: {name}");
        }
        let data = tokio::fs::read(&path).await?;
        ctx.register_bytes(
            &format!("libraries/{rel}"),
            ArtifactKind::Runtime,
            "both",
            &data,
            "library",
        )
        .await?;
    }
    Ok(())
}

/// Зарегистрировать все библиотеки version.json как файлы сборки, читая их с диска.
async fn register_version_libraries(
    ctx: &BootstrapCtx<'_>,
    vj: &Value,
    libs_dir: &Path,
) -> Result<()> {
    let Some(libs) = vj["libraries"].as_array() else {
        return Ok(());
    };
    for lib in libs {
        let Some(name) = lib["name"].as_str() else {
            continue;
        };
        // Use artifact path when available: NeoForge 21.x lists the universal jar
        // as "net.neoforged:neoforge:21.1.233" (no :universal classifier), but
        // artifact.path correctly points to neoforge-21.1.233-universal.jar.
        let rel = lib["downloads"]["artifact"]["path"]
            .as_str()
            .map(str::to_string)
            .or_else(|| maven_to_path(name));
        let Some(rel) = rel else { continue };

        let path = libs_dir.join(&rel);
        if !path.exists() {
            ctx.logf(format!(
                "предупреждение: библиотека {name} не материализована"
            ));
            continue;
        }
        let data = tokio::fs::read(&path).await?;
        // PathBasedLocator discovers the NeoForge/Forge universal jar by filename
        // (neoforge-{v}-universal.jar / forge-{mc}-{v}-universal.jar). If it also
        // appears in legacyClassPath, ModLauncher sees two modules with the same
        // name and throws ResolutionException. Register as Runtime so it stays off
        // the classpath. Detect by filename to catch both coordinate formats:
        //   - "net.neoforged:neoforge:21.1.233:universal" (install_profile style)
        //   - "net.neoforged:neoforge:21.1.233" (version.json style, no classifier)
        let is_loader_universal = rel.ends_with("-universal.jar")
            && (name.starts_with("net.neoforged:neoforge:")
                || name.starts_with("net.minecraftforge:forge:"));
        let kind = if is_loader_universal {
            ArtifactKind::Runtime
        } else {
            ArtifactKind::Library
        };
        ctx.register_bytes(&format!("libraries/{rel}"), kind, "both", &data, "library")
            .await?;

        // Remove any stale entry from a previous bootstrap that saved this jar
        // under the wrong maven path (without the -universal suffix). The upsert
        // above is keyed on (base_build_id, path), so the old entry would silently
        // remain and end up in legacyClassPath, causing a duplicate neoforge module.
        if is_loader_universal {
            if let Some(stale_rel) = maven_to_path(name) {
                if stale_rel != rel {
                    let _ = sqlx::query(
                        "DELETE FROM base_build_files WHERE base_build_id = $1 AND path = $2",
                    )
                    .bind(ctx.base_build_id)
                    .bind(format!("libraries/{stale_rel}"))
                    .execute(&ctx.state.db)
                    .await;
                }
            }
        }
    }
    Ok(())
}
