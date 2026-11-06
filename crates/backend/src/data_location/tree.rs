//! The list of what moves: every folder, file and link under the data
//! entries, with what is needed to copy it faithfully and check the copy.

use super::{io_failure, Refusal};
use crate::directories::DATA_ENTRIES;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Dir,
    File,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct Entry {
    /// Relative to the data root, starting with one of [`DATA_ENTRIES`].
    pub rel: PathBuf,
    pub kind: Kind,
    pub len: u64,
    pub modified: Option<SystemTime>,
    pub permissions: std::fs::Permissions,
    /// Device and inode of a file with more than one name. The store hands
    /// files to instances as hard links; copied as separate files they would
    /// take their space twice on the new disk.
    pub link: Option<(u64, u64)>,
}

#[derive(Debug, Default)]
pub struct Tree {
    /// Parents before their children, so creating them in order works.
    pub entries: Vec<Entry>,
    /// Each hard-linked file counted once.
    pub bytes: u64,
    pub files: u64,
}

/// Walks the data entries under `root`. Links are listed, not followed: a
/// link to a folder elsewhere would otherwise pull that folder into the move.
pub fn scan(root: &Path) -> Result<Tree, Refusal> {
    let mut tree = Tree::default();
    let mut seen_links = HashSet::new();
    for name in DATA_ENTRIES {
        let top = root.join(name);
        if std::fs::symlink_metadata(&top).is_err() {
            continue;
        }
        let walk = walkdir::WalkDir::new(&top)
            .follow_links(false)
            .sort_by_file_name();
        for item in walk {
            let item = item.map_err(|e| {
                let path = e.path().unwrap_or(&top).to_path_buf();
                io_failure(&path, &std::io::Error::from(e))
            })?;
            let meta = item
                .metadata()
                .map_err(|e| io_failure(item.path(), &std::io::Error::from(e)))?;
            let kind = if item.file_type().is_symlink() {
                Kind::Symlink
            } else if item.file_type().is_dir() {
                Kind::Dir
            } else {
                Kind::File
            };
            let link = (kind == Kind::File).then(|| link_key(&meta)).flatten();
            if kind == Kind::File && link.is_none_or(|key| seen_links.insert(key)) {
                tree.bytes += meta.len();
            }
            if kind != Kind::Dir {
                tree.files += 1;
            }
            tree.entries.push(Entry {
                rel: item
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(item.path())
                    .to_path_buf(),
                kind,
                len: if kind == Kind::File { meta.len() } else { 0 },
                modified: meta.modified().ok(),
                permissions: meta.permissions(),
                link,
            });
        }
    }
    Ok(tree)
}

#[cfg(unix)]
fn link_key(meta: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    (meta.nlink() > 1).then(|| (meta.dev(), meta.ino()))
}

/// The link count and file index are unstable API on Windows. There a hard
/// link is copied as a file of its own: it costs space, never correctness, and
/// the free-space check counted it twice already.
#[cfg(not(unix))]
fn link_key(_meta: &std::fs::Metadata) -> Option<(u64, u64)> {
    None
}
