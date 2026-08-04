use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Try to extract a mod icon from a local JAR file.
/// Returns a `data:image/...;base64,...` URL on success.
pub fn extract_jar_icon(jar_path: &Path) -> Option<String> {
    let file = std::fs::File::open(jar_path).ok()?;
    let mut zip = ZipArchive::new(file).ok()?;

    let bytes = try_fabric(&mut zip)
        .or_else(|| try_forge_toml(&mut zip, "META-INF/mods.toml"))
        .or_else(|| try_forge_toml(&mut zip, "META-INF/neoforge.mods.toml"))
        .or_else(|| read_entry(&mut zip, "pack.png"))?;

    Some(format!("data:image/png;base64,{}", B64.encode(&bytes)))
}

fn try_fabric(zip: &mut ZipArchive<std::fs::File>) -> Option<Vec<u8>> {
    let json_bytes = read_entry(zip, "fabric.mod.json")?;
    let json: serde_json::Value = serde_json::from_slice(&json_bytes).ok()?;
    let icon_val = json.get("icon")?;

    let icon_path = if let Some(s) = icon_val.as_str() {
        s.to_string()
    } else if let Some(obj) = icon_val.as_object() {
        // Pick the largest size variant.
        obj.keys()
            .filter_map(|k| k.parse::<u32>().ok().map(|n| (n, k.clone())))
            .max_by_key(|(n, _)| *n)
            .and_then(|(_, k)| obj[&k].as_str().map(str::to_string))?
    } else {
        return None;
    };

    read_entry(zip, &icon_path)
}

fn try_forge_toml(zip: &mut ZipArchive<std::fs::File>, entry: &str) -> Option<Vec<u8>> {
    let toml_bytes = read_entry(zip, entry)?;
    let content = std::str::from_utf8(&toml_bytes).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("logoFile") {
            if let Some(val) = trimmed.splitn(2, '=').nth(1) {
                let logo = val.trim().trim_matches('"');
                if !logo.is_empty() {
                    if let Some(bytes) = read_entry(zip, logo) {
                        return Some(bytes);
                    }
                }
            }
        }
    }
    None
}

fn read_entry(zip: &mut ZipArchive<std::fs::File>, name: &str) -> Option<Vec<u8>> {
    let mut entry = zip.by_name(name).ok()?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).ok()?;
    (!buf.is_empty()).then_some(buf)
}
