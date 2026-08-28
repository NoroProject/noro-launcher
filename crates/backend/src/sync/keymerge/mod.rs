//! Merging configs key by key rather than whole-file.
//!
//! `merge.rs` answers "who changed this file". The question here is smaller: the
//! player edited one line and the server edited another, and both edits should
//! survive. Whole-file compare calls that a conflict when there's nothing to
//! argue about.
//!
//! Unlike `merge.rs` this needs the original file itself, not just its hash, so
//! `.noro/base/` only exists for the paths where the mode is on.
//!
//! `.properties` and line-oriented `.txt` like `options.txt` only. JSON and TOML
//! are left out on purpose: a value there can be a tree, and "merge by key"
//! stops being well-defined exactly where you'd want it.

use std::collections::BTreeMap;
use std::path::Path;

mod base;

pub use base::{base_copy_path, remember_base};

/// `None` when both sides changed the same key to different values — that file
/// stays an ordinary conflict.
pub fn merge_properties(mine: &str, base: &str, theirs: &str) -> Option<String> {
    let mine_map = parse(mine);
    let base_map = parse(base);
    let theirs_map = parse(theirs);

    let mut out: BTreeMap<&str, String> = BTreeMap::new();
    let keys: Vec<&str> = mine_map
        .keys()
        .chain(base_map.keys())
        .chain(theirs_map.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    for key in keys {
        let m = mine_map.get(key);
        let b = base_map.get(key);
        let t = theirs_map.get(key);

        match (m, b, t) {
            // Gone on both sides, or never there.
            (None, _, None) => {}
            // The player deleted it and the server left it alone. A deletion is
            // an edit too.
            (None, Some(b), Some(t)) if b == t => {}
            // The server deleted it and the player left it alone.
            (Some(m), Some(b), None) if m == b => {}
            // Both sides agree.
            (Some(m), _, Some(t)) if m == t => {
                out.insert(key, (*m).to_string());
            }
            // Only the player changed it.
            (Some(m), b, t) if b == t => {
                out.insert(key, (*m).to_string());
            }
            // Only the server changed it.
            (m, b, Some(t)) if m == b => {
                out.insert(key, (*t).to_string());
            }
            // Added by one side, unknown to the other.
            (Some(m), None, None) => {
                out.insert(key, (*m).to_string());
            }
            (None, None, Some(t)) => {
                out.insert(key, (*t).to_string());
            }
            // Both changed it, differently. This is where the automation ends.
            _ => return None,
        }
    }

    Some(
        out.into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// `key=value` per line. Comments and blank lines are dropped — a merge has no
/// way to keep their placement anyway, so they don't survive the round trip.
fn parse(text: &str) -> BTreeMap<&str, &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
        .filter_map(|l| l.split_once(&['=', ':'][..]))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

pub fn is_mergeable(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    lower.ends_with(".properties") || lower.ends_with("options.txt")
}

/// Try to resolve a conflict by merging keys.
///
/// The server's version has to be downloaded before the decision — there's
/// nothing to merge against otherwise. Affordable only because this runs on
/// configs, which are kilobytes, and only on a real conflict.
///
/// `None` means the conflict policy decides instead: wrong format, no base
/// copy, or one key changed differently on both sides.
pub async fn try_merge(
    client: &reqwest::Client,
    instance_dir: &Path,
    rel: &str,
    url: &str,
) -> Option<String> {
    if !is_mergeable(rel) {
        return None;
    }
    let base = tokio::fs::read_to_string(base_copy_path(instance_dir, rel))
        .await
        .ok()?;
    let mine = tokio::fs::read_to_string(instance_dir.join(rel))
        .await
        .ok()?;
    let theirs = client.get(url).send().await.ok()?.text().await.ok()?;

    let merged = merge_properties(&mine, &base, &theirs)?;
    tokio::fs::write(instance_dir.join(rel), &merged)
        .await
        .ok()?;
    // The new base is the server's text, not the merged result: next time we
    // want to know what changed relative to what they sent.
    let base_path = base_copy_path(instance_dir, rel);
    if let Some(parent) = base_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(base_path, &theirs).await;
    Some(merged)
}

#[cfg(test)]
#[path = "../keymerge_tests.rs"]
mod tests;
