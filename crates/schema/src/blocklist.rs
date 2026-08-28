//! Banned files.
//!
//! Ships inside the signed manifest, or the list could just be swapped out on
//! the client.
//!
//! These rules outrank every path rule, `unmanaged` included — that's the point:
//! the resourcepacks folder isn't synced, but xray still gets deleted from it.
//!
//! Both matchers are cheap to defeat: a hash by changing one byte, a name mask
//! by renaming. They catch the lazy. Real coverage comes from "file is not in
//! the manifest" (§10.4), which lives elsewhere.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlockAction {
    /// Delete and carry on. The player sees a neutral message.
    #[default]
    Delete,
    /// Leave it, but tell an admin. For cases where deleting costs more than a
    /// false positive.
    Flag,
    /// Refuse to launch while the file is there.
    BlockLaunch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedFile {
    /// Name mask, e.g. `*xray*`. `None` means match on hash alone.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Exact SHA1. `None` means match on the mask alone.
    #[serde(default)]
    pub sha1: Option<String>,
    /// Goes into the admin flag.
    pub reason: String,
    #[serde(default)]
    pub action: BlockAction,
}

impl BlockedFile {
    /// When both a mask and a hash are set, both have to match — "this hash
    /// under this name" is narrower than either half and misfires less.
    pub fn matches(&self, rel_path: &str, sha1: &str) -> bool {
        let by_name = match &self.pattern {
            Some(p) => glob_match(&rel_path.to_lowercase(), &p.to_lowercase()),
            None => true,
        };
        let by_hash = match &self.sha1 {
            Some(h) => h.eq_ignore_ascii_case(sha1),
            None => true,
        };
        // An empty rule would otherwise match everything.
        (self.pattern.is_some() || self.sha1.is_some()) && by_name && by_hash
    }
}

/// `*` anywhere: `*xray*`, `mods/x*.jar`.
fn glob_match(text: &str, pattern: &str) -> bool {
    // With no star it's an exact path, not a prefix — otherwise
    // `mods/banned.jar` would also take out `mods/banned.jar.bak`.
    if !pattern.contains('*') {
        return text == pattern;
    }

    let mut parts = pattern.split('*');
    let Some(first) = parts.next() else {
        return true;
    };
    if !text.starts_with(first) {
        return false;
    }
    let mut rest = &text[first.len()..];

    let parts: Vec<&str> = parts.collect();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        // A trailing segment with no `*` after it has to end the string.
        if i == parts.len() - 1 && !pattern.ends_with('*') {
            return rest.ends_with(part);
        }
        match rest.find(part) {
            Some(pos) => rest = &rest[pos + part.len()..],
            None => return false,
        }
    }
    true
}

pub fn first_match<'a>(
    rules: &'a [BlockedFile],
    rel_path: &str,
    sha1: &str,
) -> Option<&'a BlockedFile> {
    rules.iter().find(|r| r.matches(rel_path, sha1))
}

#[cfg(test)]
#[path = "blocklist_tests.rs"]
mod tests;
