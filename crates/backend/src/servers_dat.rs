//! Список серверов мастера → `servers.dat` игрового клиента.
//!
//! Пишется только перед запуском: Minecraft читает файл на старте и сам
//! перезаписывает его при выходе, поэтому править на работающем клиенте
//! бессмысленно.
//!
//! Свои записи игрока сохраняются как есть — вместе с полями, которых мы не
//! знаем: запись переносится целиком, а не пересобирается по известным ключам.

use anyhow::{Context, Result};
use fastnbt::Value;
use schema::ServerEntry;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// Имя рядом с `servers.dat`: помнит, что мы записали в прошлый раз.
const STAMP: &str = ".noro-servers";

#[derive(Serialize, Deserialize, Default)]
struct ServersDat {
    servers: Vec<HashMap<String, Value>>,
}

/// Приводит `servers.dat` к игровым серверам сборки.
///
/// @return `true`, если файл переписан
pub fn sync(instance_dir: &Path, server: &ServerEntry) -> Result<bool> {
    let desired = desired_entries(server);

    let stamp_path = instance_dir.join(STAMP);
    let path = instance_dir.join("servers.dat");
    let stamp = fingerprint(&desired);
    // В штампе первая строка — отпечаток, дальше адреса; сверяем только её.
    let known = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    // Наличие файла проверяем отдельно: штамп говорит лишь о том, менялся ли
    // список у мастера. Пропади сам servers.dat — а штамп останься, — мы бы
    // вечно отвечали «всё сделано», и список серверов не вернулся бы никогда.
    // Удалённые игроком отдельные записи это по-прежнему не трогает: там файл
    // на месте, и его содержимое разбирается ниже.
    if known.lines().next() == Some(stamp.as_str()) && path.exists() {
        return Ok(false);
    }

    let existing = read(&path)?;

    // Чужими считаем всё, чей адрес не принадлежит мастеру — ни сейчас, ни в
    // прошлый раз. Иначе снятый со сборки сервер остался бы в списке навсегда.
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
        fastnbt::to_bytes(&ServersDat { servers: out }).context("сериализация servers.dat")?;
    std::fs::write(&path, bytes).with_context(|| format!("запись {}", path.display()))?;
    // Не `.ok()`: без штампа следующий запуск перепишет servers.dat заново и
    // затрёт записи, которые игрок успел добавить сам.
    std::fs::write(&stamp_path, stamp_with_ips(&stamp, &desired))
        .with_context(|| format!("запись {}", stamp_path.display()))?;
    Ok(true)
}

/// Какие серверы сборки показать игроку.
///
/// Если у сборки есть прокси, в список идёт только он: бэкенды за ним —
/// внутренние, и прямой коннект к ним обошёл бы и прокси, и его проверки.
/// Без прокси показываем все игровые серверы — они и есть точки входа.
fn desired_entries(server: &ServerEntry) -> Vec<(String, String)> {
    let has_proxy = server.game_servers.iter().any(|node| node.proxy);
    server
        .game_servers
        .iter()
        .filter(|node| !has_proxy || node.proxy)
        .map(|node| (node.name.clone(), address(&node.mc_host, node.mc_port)))
        .collect()
}

/// Порт опускаем на 25565: Minecraft показывает адрес как есть, и `:25565`
/// в списке выглядит мусором.
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
    // Ресурспак сборки едет с мастера, спрашивать про него незачем.
    map.insert("acceptTextures".to_string(), Value::Byte(1));
    map
}

fn read(path: &Path) -> Result<ServersDat> {
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(ServersDat::default());
    };
    // Битый или чужого формата файл не повод падать перед запуском игры:
    // начнём список заново, потеряв разве что записи игрока. Но молчать об этом
    // нельзя — иначе пропавший список серверов выглядит как наша самодеятельность.
    match fastnbt::from_bytes(&bytes) {
        Ok(dat) => Ok(dat),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "servers.dat не разобран, список пересоздан");
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

/// Отпечаток списка: имя+адрес каждой записи в стабильном порядке.
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

/// В штампе после отпечатка лежат адреса прошлой записи — по ним видно, что
/// убрать, когда сервер сняли с мастера.
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
