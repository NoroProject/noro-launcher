//! The master's server list, written into the client's `servers.dat`.
//!
//! Only ever written before launch. Minecraft reads the file at startup and
//! rewrites it on exit, so editing it under a running client accomplishes
//! nothing.
//!
//! The player's own entries are carried across whole, fields we don't recognise
//! included — they are moved, not rebuilt from the keys we know about.

use anyhow::{Context, Result};
use fastnbt::Value;
use schema::ServerEntry;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// Sits next to `servers.dat` and remembers what we put there last time.
const STAMP: &str = ".noro-servers";

#[derive(Serialize, Deserialize, Default)]
struct ServersDat {
    servers: Vec<HashMap<String, Value>>,
}

/// Brings `servers.dat` in line with the build's game servers. Returns whether
/// the file was rewritten.
pub fn sync(instance_dir: &Path, server: &ServerEntry) -> Result<bool> {
    let desired = desired_entries(server);

    let stamp_path = instance_dir.join(STAMP);
    let path = instance_dir.join("servers.dat");
    let stamp = fingerprint(&desired);
    // First line of the stamp is the fingerprint; the rest are addresses.
    let known = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    // The stamp only says whether the master's list changed, hence the separate
    // existence check: if servers.dat disappeared while the stamp survived we'd
    // keep answering "nothing to do" and the list would never come back. An
    // entry the player deleted by hand is a different matter — the file is
    // still there and gets merged below.
    if known.lines().next() == Some(stamp.as_str()) && path.exists() {
        return Ok(false);
    }

    let existing = read(&path)?;

    // Ours means an address the master gave us, now or last time. Without the
    // previous set, a server pulled from the build would stay in the list
    // forever, looking like something the player added.
    let mut ours: HashSet<String> = desired.iter().map(|(_, ip)| ip.clone()).collect();
    ours.extend(previous(&stamp_path));

    let mut out: Vec<HashMap<String, Value>> =
        desired.iter().map(|(name, ip)| record(name, ip)).collect();
    for entry in existing.servers {
        let ip = entry.get("ip").and_then(as_str).unwrap_or_default();
        if !ours.contains(&ip) {
            out.push(entry);
        }
    }

    std::fs::create_dir_all(instance_dir).ok();
    let bytes =
        fastnbt::to_bytes(&ServersDat { servers: out }).context("serializing servers.dat")?;
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    // Deliberately not `.ok()`: with no stamp, the next launch rewrites
    // servers.dat from scratch and loses whatever the player added.
    std::fs::write(&stamp_path, stamp_with_ips(&stamp, &desired))
        .with_context(|| format!("writing {}", stamp_path.display()))?;
    Ok(true)
}

/// If the build has a proxy, only the proxy goes in the list. The nodes behind
/// it are internal, and connecting straight to one skips the proxy's checks.
/// Without a proxy every game server is an entry point in its own right.
fn desired_entries(server: &ServerEntry) -> Vec<(String, String)> {
    let has_proxy = server.game_servers.iter().any(|node| node.proxy);
    server
        .game_servers
        .iter()
        .filter(|node| !has_proxy || node.proxy)
        .map(|node| (node.name.clone(), address(&node.mc_host, node.mc_port)))
        .collect()
}

/// Minecraft shows the address verbatim, so the default port is left off.
fn address(host: &str, port: u16) -> String {
    if port == 25565 {
        host.to_string()
    } else {
        format!("{host}:{port}")
    }
}

fn record(name: &str, ip: &str) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String(name.to_string()));
    map.insert("ip".to_string(), Value::String(ip.to_string()));
    // The build's resource pack comes from the master anyway; no reason to make
    // the player answer a prompt about it.
    map.insert("acceptTextures".to_string(), Value::Byte(1));
    map
}

fn read(path: &Path) -> Result<ServersDat> {
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(ServersDat::default());
    };
    // A corrupt or foreign-format file is no reason to fail a launch: start the
    // list over and lose at most the player's own entries. Worth a warning
    // though, or a vanished server list looks like something we did on purpose.
    match fastnbt::from_bytes(&bytes) {
        Ok(dat) => Ok(dat),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "servers.dat unreadable, list rebuilt from scratch");
            Ok(ServersDat::default())
        }
    }
}

fn as_str(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Name and address of every entry, in a stable order.
fn fingerprint(desired: &[(String, String)]) -> String {
    let sorted: BTreeMap<&str, &str> = desired
        .iter()
        .map(|(name, ip)| (ip.as_str(), name.as_str()))
        .collect();
    let joined = sorted
        .iter()
        .map(|(ip, name)| format!("{ip}\u{1}{name}"))
        .collect::<Vec<_>>()
        .join("\u{2}");
    hex::encode(<sha1::Sha1 as sha1::Digest>::digest(joined.as_bytes()))
}

/// The addresses after the fingerprint are what tells the next run which
/// entries to drop once a server is gone from the master.
fn stamp_with_ips(stamp: &str, desired: &[(String, String)]) -> String {
    std::iter::once(stamp.to_string())
        .chain(desired.iter().map(|(_, ip)| ip.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn previous(stamp_path: &Path) -> Vec<String> {
    std::fs::read_to_string(stamp_path)
        .unwrap_or_default()
        .lines()
        .skip(1)
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
#[path = "servers_dat_tests.rs"]
mod tests;
