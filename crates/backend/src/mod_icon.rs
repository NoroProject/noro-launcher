use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use std::io::{Read, Seek};
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ModMetadata {
    pub mod_id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

/// Try to extract mod metadata (ID, title, version) from a JAR file.
pub fn extract_jar_metadata(jar_path: &Path) -> Option<ModMetadata> {
    let file = std::fs::File::open(jar_path).ok()?;
    let mut zip = ZipArchive::new(file).ok()?;

    if let Some((_, json_bytes)) = read_entry(&mut zip, "fabric.mod.json") {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&json_bytes) {
            let mod_id = json.get("id").and_then(|v| v.as_str()).map(String::from);
            let name = json.get("name").and_then(|v| v.as_str()).map(String::from);
            let version = json
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from);
            if mod_id.is_some() || name.is_some() || version.is_some() {
                return Some(ModMetadata {
                    mod_id,
                    name,
                    version,
                });
            }
        }
    }

    if let Some((_, json_bytes)) = read_entry(&mut zip, "quilt.mod.json") {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&json_bytes) {
            let meta = json.pointer("/quilt_loader/metadata");
            let mod_id = meta
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let name = meta
                .and_then(|m| m.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let version = meta
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .map(String::from);
            if mod_id.is_some() || name.is_some() || version.is_some() {
                return Some(ModMetadata {
                    mod_id,
                    name,
                    version,
                });
            }
        }
    }

    for entry in &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Some((_, toml_bytes)) = read_entry(&mut zip, entry) {
            if let Ok(content) = std::str::from_utf8(&toml_bytes) {
                let mut mod_id = None;
                let mut name = None;
                let mut version = None;

                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("modId") && mod_id.is_none() {
                        if let Some(val) = trimmed.split_once('=').map(|x| x.1) {
                            mod_id =
                                Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                        }
                    } else if trimmed.starts_with("displayName") && name.is_none() {
                        if let Some(val) = trimmed.split_once('=').map(|x| x.1) {
                            name =
                                Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                        }
                    } else if trimmed.starts_with("version") && version.is_none() {
                        if let Some(val) = trimmed.split_once('=').map(|x| x.1) {
                            let v = val.trim().trim_matches('"').trim_matches('\'').to_string();
                            if v != "${file.jarVersion}" {
                                version = Some(v);
                            }
                        }
                    }
                }
                if mod_id.is_some() || name.is_some() || version.is_some() {
                    return Some(ModMetadata {
                        mod_id,
                        name,
                        version,
                    });
                }
            }
        }
    }

    None
}

/// Try to extract a mod icon from a local JAR file.
/// Returns a `data:image/...;base64,...` URL on success.
pub fn extract_jar_icon(jar_path: &Path) -> Option<String> {
    let file = std::fs::File::open(jar_path).ok()?;
    extract_jar_icon_from_reader(file)
}

/// Try to extract a mod icon from any seekable reader (e.g. byte buffer or file).
pub fn extract_jar_icon_from_reader<R: Read + Seek>(reader: R) -> Option<String> {
    let mut zip = ZipArchive::new(reader).ok()?;

    let (mime, bytes) = try_fabric(&mut zip)
        .or_else(|| try_quilt(&mut zip))
        .or_else(|| try_forge_toml(&mut zip, "META-INF/mods.toml"))
        .or_else(|| try_forge_toml(&mut zip, "META-INF/neoforge.mods.toml"))
        .or_else(|| try_common_paths(&mut zip))?;

    Some(format!("data:{};base64,{}", mime, B64.encode(&bytes)))
}

fn try_fabric<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Option<(String, Vec<u8>)> {
    let (_, json_bytes) = read_entry(zip, "fabric.mod.json")?;
    let json: serde_json::Value = serde_json::from_slice(&json_bytes).ok()?;
    let icon_val = json.get("icon")?;

    // `icon` — либо строка с путём, либо объект «размер → путь»; берём самый
    // крупный размер. Всё остальное трактуется как отсутствие иконки.
    let icon_path = if let Some(s) = icon_val.as_str() {
        s.to_string()
    } else {
        let obj = icon_val.as_object()?;
        obj.keys()
            .filter_map(|k| k.parse::<u32>().ok().map(|n| (n, k.clone())))
            .max_by_key(|(n, _)| *n)
            .and_then(|(_, k)| obj[&k].as_str().map(str::to_string))?
    };

    read_entry(zip, &icon_path)
}

fn try_quilt<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Option<(String, Vec<u8>)> {
    let (_, json_bytes) = read_entry(zip, "quilt.mod.json")?;
    let json: serde_json::Value = serde_json::from_slice(&json_bytes).ok()?;
    let icon_path = json
        .pointer("/quilt_loader/metadata/icon")
        .and_then(|v| v.as_str())?;

    read_entry(zip, icon_path)
}

fn try_forge_toml<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    entry: &str,
) -> Option<(String, Vec<u8>)> {
    let (_, toml_bytes) = read_entry(zip, entry)?;
    let content = std::str::from_utf8(&toml_bytes).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("logoFile")
            || trimmed.starts_with("logo_file")
            || trimmed.starts_with("icon")
            || trimmed.starts_with("iconFile")
        {
            if let Some(val) = trimmed.split_once('=').map(|x| x.1) {
                let logo = val.trim().trim_matches('"').trim_matches('\'');
                if !logo.is_empty() {
                    if let Some(res) = read_entry(zip, logo) {
                        return Some(res);
                    }
                }
            }
        }
    }
    None
}

fn try_common_paths<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Option<(String, Vec<u8>)> {
    let candidates = [
        "icon.png",
        "logo.png",
        "mod_icon.png",
        "pack.png",
        "assets/icon.png",
        "assets/logo.png",
    ];

    for path in candidates {
        if let Some(res) = read_entry(zip, path) {
            return Some(res);
        }
    }

    let mut found_index = None;
    for i in 0..zip.len() {
        if let Ok(file) = zip.by_index(i) {
            let name = file.name().to_lowercase();
            if (name.ends_with("icon.png") || name.ends_with("logo.png"))
                && !name.contains("__macosx")
            {
                found_index = Some(i);
                break;
            }
        }
    }

    if let Some(idx) = found_index {
        if let Ok(mut entry) = zip.by_index(idx) {
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                let name = entry.name().to_string();
                let mime = get_mime(&name);
                return Some((mime.to_string(), buf));
            }
        }
    }

    None
}

fn read_entry<R: Read + Seek>(zip: &mut ZipArchive<R>, name: &str) -> Option<(String, Vec<u8>)> {
    let clean_name = name.trim_start_matches('/').trim_start_matches("./");

    if let Ok(mut entry) = zip.by_name(clean_name) {
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
            let mime = get_mime(clean_name);
            return Some((mime.to_string(), buf));
        }
    }

    let mut found_index = None;
    for i in 0..zip.len() {
        if let Ok(f) = zip.by_index(i) {
            let fn_name = f.name().trim_start_matches('/').trim_start_matches("./");
            if fn_name.eq_ignore_ascii_case(clean_name) {
                found_index = Some(i);
                break;
            }
        }
    }

    if let Some(idx) = found_index {
        if let Ok(mut entry) = zip.by_index(idx) {
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                let mime = get_mime(clean_name);
                return Some((mime.to_string(), buf));
            }
        }
    }

    None
}

fn get_mime(filename: &str) -> &'static str {
    if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".svg") {
        "image/svg+xml"
    } else if filename.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    }
}
