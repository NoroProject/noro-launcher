//! The game's `screenshots` folder, for the launcher's gallery tab.
//!
//! The window names a screenshot by file name only. Everything that touches the
//! disk resolves that name here, inside the one folder, so a crafted name can't
//! reach a file elsewhere.

use bridge::ScreenshotInfo;
use std::path::{Path, PathBuf};

/// More than anyone scrolls through; past it the oldest are left out rather
/// than stat'ing a folder somebody has been filling for years.
const MAX_LISTED: usize = 1000;

pub fn folder(instance: &Path) -> PathBuf {
    instance.join("screenshots")
}

fn is_picture(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")
}

/// Newest first. A missing folder is an empty gallery, not an error: the game
/// creates it with the first screenshot.
pub fn list(folder: &Path) -> Vec<ScreenshotInfo> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut items: Vec<ScreenshotInfo> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name().to_str()?.to_string();
            if !is_picture(&file_name) {
                return None;
            }
            // `file_type` doesn't follow links, so a link planted in the folder
            // is skipped instead of showing whatever it points at.
            if !entry.file_type().ok()?.is_file() {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let taken_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            Some(ScreenshotInfo {
                path: entry.path(),
                file_name,
                taken_at,
                size_bytes: meta.len(),
            })
        })
        .collect();
    items.sort_by(|a, b| {
        b.taken_at
            .cmp(&a.taken_at)
            .then_with(|| b.file_name.cmp(&a.file_name))
    });
    items.truncate(MAX_LISTED);
    items
}

/// The file `name` in `folder`, if it is a plain picture right there.
///
/// Anything with a separator, `..`, or a name the listing would not have shown
/// is refused, and so is a link.
pub fn resolve(folder: &Path, name: &str) -> Option<PathBuf> {
    let plain = Path::new(name).file_name().and_then(|n| n.to_str()) == Some(name);
    if !plain || name.contains(['/', '\\']) || !is_picture(name) {
        return None;
    }
    let path = folder.join(name);
    let meta = std::fs::symlink_metadata(&path).ok()?;
    meta.is_file().then_some(path)
}

#[cfg(test)]
#[path = "screenshots_tests.rs"]
mod tests;
