//! База запрещённых файлов.
//!
//! Едет внутри подписанного манифеста — иначе список подменяется на клиенте, и
//! вся затея теряет смысл.
//!
//! Приоритет выше всех правил путей, включая `unmanaged`: в этом и смысл —
//! папка ресурспаков не синхронизируется, но xray оттуда удаляется.
//!
//! Честно о границах: SHA1 обходится изменением одного байта, маска имени —
//! переименованием. Обе ловят ленивых, а не мотивированных. Настоящий охват
//! даёт «файла нет в манифесте» (§10.4), и это отдельная механика.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlockAction {
    /// Удалить и продолжить. Умолчание: игрок видит нейтральное сообщение.
    #[default]
    Delete,
    /// Оставить, но сообщить админу. Для случаев, где удаление дороже ошибки.
    Flag,
    /// Не пускать в игру, пока файл на месте.
    BlockLaunch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedFile {
    /// Маска имени: `*xray*`. Пусто — правило только по хешу.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Точный SHA1. Пусто — правило только по маске.
    #[serde(default)]
    pub sha1: Option<String>,
    /// Зачем запрещён — попадает в флаг админу.
    pub reason: String,
    #[serde(default)]
    pub action: BlockAction,
}

impl BlockedFile {
    /// Подпадает ли файл под правило.
    ///
    /// Оба условия, если заданы оба: правило «этот хеш под этим именем» строже
    /// каждого по отдельности и даёт меньше ложных срабатываний.
    pub fn matches(&self, rel_path: &str, sha1: &str) -> bool {
        let by_name = match &self.pattern {
            Some(p) => glob_match(&rel_path.to_lowercase(), &p.to_lowercase()),
            None => true,
        };
        let by_hash = match &self.sha1 {
            Some(h) => h.eq_ignore_ascii_case(sha1),
            None => true,
        };
        // Пустое правило не должно матчить всё подряд.
        (self.pattern.is_some() || self.sha1.is_some()) && by_name && by_hash
    }
}

/// Маска с `*` в любом месте: `*xray*`, `mods/x*.jar`.
fn glob_match(text: &str, pattern: &str) -> bool {
    // Без звёзд это точный путь, а не префикс: иначе `mods/banned.jar` удаляло
    // бы и `mods/banned.jar.bak`.
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
        // Последний кусок без `*` на конце обязан завершать строку.
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

/// Первое сработавшее правило.
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
