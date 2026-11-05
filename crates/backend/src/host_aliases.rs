//! Server aliases: the name the game shows instead of a server's address.
//!
//! The master gives every game server an alias such as `survival.example`
//! next to its real address. The game only ever sees the alias — in quick play,
//! `servers.dat`, its own log — while authlib-injector resolves it to the real
//! host inside the JVM, from a table this module writes. That keeps the address
//! off a stream or a screenshot; it is not a defence against anyone holding the
//! machine, who can read it from `netstat`.
//!
//! Only an authlib-injector build that can resolve aliases may be handed them:
//! with any other the game would join a name nothing resolves. Such a build
//! says so in its manifest.

use anyhow::{Context, Result};
use schema::ServerEntry;
use std::io::Read;
use std::path::{Path, PathBuf};

/// System property with the table's path, read by the agent in `premain`.
pub const PROPERTY: &str = "authlibinjector.hostAliases";

/// Manifest attribute of an authlib-injector build that resolves aliases.
const MANIFEST_MARK: &str = "Noro-Host-Aliases:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostAlias {
    pub alias: String,
    pub host: String,
    pub port: u16,
}

/// Every alias the player can reach, not just the launched build's: the game's
/// server list holds the other builds' servers too.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostAliases(Vec<HostAlias>);

impl HostAliases {
    pub fn from_servers(servers: &[ServerEntry]) -> Self {
        let mut out: Vec<HostAlias> = Vec::new();
        for node in servers.iter().flat_map(|s| &s.game_servers) {
            let Some(alias) = node.alias.as_deref() else {
                continue;
            };
            // The table is whitespace-separated; a name that would break a
            // line, or one already taken, is left out rather than trusted.
            let fits = |s: &str| !s.is_empty() && !s.contains(char::is_whitespace);
            if !fits(alias) || !fits(&node.mc_host) || out.iter().any(|a| a.alias == alias) {
                continue;
            }
            out.push(HostAlias {
                alias: alias.to_string(),
                host: node.mc_host.clone(),
                port: node.mc_port,
            });
        }
        Self(out)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The alias standing in for `host:port`, if there is one.
    pub fn alias_of(&self, host: &str, port: u16) -> Option<&str> {
        self.0
            .iter()
            .find(|a| a.host.eq_ignore_ascii_case(host) && a.port == port)
            .map(|a| a.alias.as_str())
    }

    /// One `<alias> <host> <port>` line per alias.
    pub fn table(&self) -> String {
        self.0
            .iter()
            .map(|a| format!("{} {} {}\n", a.alias, a.host, a.port))
            .collect()
    }

    /// Writes the table for the agent and returns where it is.
    pub async fn write(&self, instance_dir: &Path) -> Result<PathBuf> {
        let path = instance_dir.join(".noro").join("host-aliases.txt");
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        crate::fsutil::write_atomic(&path, self.table())
            .await
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }
}

/// Whether this authlib-injector jar resolves aliases. Anything unreadable
/// counts as no: the game then gets real addresses and still joins.
pub fn agent_resolves_aliases(jar: &Path) -> bool {
    let read = || -> Option<String> {
        let file = std::fs::File::open(jar).ok()?;
        let mut zip = zip::ZipArchive::new(file).ok()?;
        let mut manifest = zip.by_name("META-INF/MANIFEST.MF").ok()?;
        let mut text = String::new();
        manifest.read_to_string(&mut text).ok()?;
        Some(text)
    };
    read().is_some_and(|text| text.lines().any(|l| l.starts_with(MANIFEST_MARK)))
}

#[cfg(test)]
#[path = "host_aliases_tests.rs"]
mod tests;
