//! `bootstrap.json`: what this binary leaves for core next to the environment,
//! and the way a move of the master reaches a bootstrapper that can't update.
//!
//! When the master changes address, core learns it from the master and writes
//! `master_url` with the new address and `moved_from` with the old origins
//! here. The stamped address still wins over everything else, unless it is
//! one of those the master has left.

use reqwest::Url;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub fn path() -> Option<PathBuf> {
    Some(
        dirs::data_dir()?
            .join(schema::launcher_dir_name())
            .join("bootstrap.json"),
    )
}

/// The file as a JSON object. Missing, unreadable or not an object is `None`.
pub fn read(path: &Path) -> Option<Map<String, Value>> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str(&raw).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

/// A non-empty string field, trimmed.
pub fn field(doc: &Map<String, Value>, key: &str) -> Option<String> {
    doc.get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn origin(url: &str) -> Option<String> {
    let url = Url::parse(url.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }
    Some(url.origin().ascii_serialization())
}

/// Where the master went, if it left the address this binary was stamped
/// with. Anything short of a bare https origin is ignored, the same rule core
/// applies before it writes the file.
pub fn moved_master(stamped: &str, doc: &Map<String, Value>) -> Option<String> {
    let stamped = origin(stamped)?;
    let left = doc
        .get("moved_from")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .any(|o| origin(o).as_deref() == Some(stamped.as_str()));
    if !left {
        return None;
    }
    let target = Url::parse(&field(doc, "master_url")?).ok()?;
    let bare = target.scheme() == "https"
        && target.host_str().is_some()
        && target.username().is_empty()
        && target.password().is_none()
        && target.path() == "/"
        && target.query().is_none()
        && target.fragment().is_none();
    let target = target.origin().ascii_serialization();
    (bare && target != stamped).then_some(target)
}

/// Rewrite the file for core, keeping what core put there: dropping
/// `moved_from` would send this binary back to the old address next time.
pub fn write(path: &Path, master_url: &str, signing_pubkey: &str) {
    let mut doc = read(path).unwrap_or_default();
    doc.insert("master_url".into(), master_url.into());
    doc.insert("signing_pubkey".into(), signing_pubkey.into());
    if let Ok(json) = serde_json::to_string_pretty(&doc) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OLD: &str = "https://api.old.example.com";
    const NEW: &str = "https://api.new.example.com";

    fn doc(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(map) => map,
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_move_applies_when_the_stamp_is_what_the_master_left() {
        let d = doc(json!({ "master_url": NEW, "moved_from": [OLD] }));
        assert_eq!(moved_master(OLD, &d).as_deref(), Some(NEW));
        assert_eq!(moved_master(&format!("{OLD}/"), &d).as_deref(), Some(NEW));
    }

    #[test]
    fn a_stamp_the_master_never_left_wins() {
        let d = doc(json!({ "master_url": NEW, "moved_from": [OLD] }));
        assert_eq!(moved_master("https://api.third.example.com", &d), None);
        // No record of a move: the file only echoes the last stamp.
        let d = doc(json!({ "master_url": NEW }));
        assert_eq!(moved_master(OLD, &d), None);
        let d = doc(json!({ "master_url": NEW, "moved_from": [] }));
        assert_eq!(moved_master(OLD, &d), None);
    }

    #[test]
    fn a_target_that_is_no_bare_https_origin_is_ignored() {
        for bad in [
            "",
            "garbage",
            "http://api.new.example.com",
            "https://api.new.example.com/path",
            "https://user@api.new.example.com",
            OLD,
        ] {
            let d = doc(json!({ "master_url": bad, "moved_from": [OLD] }));
            assert_eq!(moved_master(OLD, &d), None, "{bad:?}");
        }
    }

    #[test]
    fn rewriting_keeps_the_move() {
        let dir = std::env::temp_dir().join(format!("noro-bootfile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bootstrap.json");
        std::fs::write(
            &path,
            json!({ "master_url": NEW, "moved_from": [OLD], "signing_pubkey": "old" }).to_string(),
        )
        .unwrap();

        write(&path, NEW, "abcd");
        let d = read(&path).unwrap();
        assert_eq!(d["master_url"], NEW);
        assert_eq!(d["moved_from"], json!([OLD]));
        assert_eq!(d["signing_pubkey"], "abcd");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
