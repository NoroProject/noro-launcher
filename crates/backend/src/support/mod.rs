//! Support bundles: logs and environment, for working out what went wrong.
//!
//! The contents are a fixed allowlist in code rather than "whatever is in the
//! folder", so the set can't grow on its own the day a new file appears in the
//! directory.
//!
//! Never collected: `saves/`, `screenshots/`, `servers.dat`.
//!
//! Everything goes through [`crate::redact`] before packing, and the redacted
//! text is exactly what the preview shows — the player sees `C:\Users\*****`
//! rather than their own name.

mod collect;
mod environment;
mod pack;
mod send;

pub use collect::collect;
pub use pack::pack;
pub use send::{send, send_for_request};

use serde::{Deserialize, Serialize};

/// One file of the bundle, already redacted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFile {
    /// Name inside the archive, e.g. `logs/latest.log`.
    pub name: String,
    pub text: String,
    /// Size of the file on disk, shown to the player before sending. Not the
    /// length of `text`, which is clamped.
    pub original_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub files: Vec<BundleFile>,
    /// OS, launcher version, build, mod set — what the logs don't say.
    pub environment: String,
}

impl Bundle {
    /// The same text the archive carries, so the preview can't understate it.
    pub fn preview(&self) -> String {
        let mut out = format!("=== environment ===\n{}\n", self.environment);
        for f in &self.files {
            out.push_str(&format!(
                "\n=== {} ({} bytes on disk) ===\n{}\n",
                f.name, f.original_bytes, f.text
            ));
        }
        out
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.text.len() as u64).sum()
    }
}
