//! Правила обращения с путями внутри игрового каталога.
//!
//! Раньше это были три независимых списка (`unmanaged_paths`,
//! `user_managed_paths` и всё остальное по умолчанию) с неочевидным
//! приоритетом. Схлопнуты в один упорядоченный список, как в `.gitignore`:
//! **побеждает последнее совпавшее правило**. Так исключения выражаются
//! естественно — `config/**` в одном режиме, `config/xaero*` в другом, — а не
//! разносятся по трём массивам.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PathMode {
    /// Файл принадлежит сборке: качается, обновляется по хешу, лишнее удаляется.
    #[default]
    Managed,
    /// Ставится один раз, дальше принадлежит игроку. Не обновляется никогда —
    /// именно в этом главный дефект, который чинит `Merged`.
    UserManaged,
    /// Не трогается вовсе: не качается, не обновляется, не удаляется.
    Unmanaged,
    /// Three-way по хешам: правки игрока не затираются, но обновления с сервера
    /// доезжают. См. [`MergeDecision`].
    Merged,
}

/// Что делать при конфликте в режиме `Merged`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    /// Оставить версию игрока и поднять флаг админу.
    #[default]
    KeepMine,
    /// Взять серверную, положив прежнюю в `.noro/conflicts/<timestamp>/`.
    TakeTheirs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathRule {
    /// Маска: `config/**`, `saves/`, `options.txt`, `config/xaero*`.
    pub pattern: String,
    pub mode: PathMode,
    #[serde(default)]
    pub conflict: ConflictPolicy,
}

/// Режим для пути. Побеждает последнее совпавшее правило.
pub fn mode_for(path: &str, rules: &[PathRule]) -> PathMode {
    rule_for(path, rules).map(|r| r.mode).unwrap_or_default()
}

/// Само правило — нужно ещё и для политики конфликта.
pub fn rule_for<'a>(path: &str, rules: &'a [PathRule]) -> Option<&'a PathRule> {
    let lower = path.to_lowercase();
    rules
        .iter()
        .rev()
        .find(|r| matches(&lower, &r.pattern.to_lowercase()))
}

/// Совпадение пути с маской.
///
/// Три формы, все три уже встречаются в существующих сборках:
/// `dir/` — каталог целиком, `name*` — префикс, `path` — точное совпадение.
/// `**` внутри работает как «любой остаток»: `config/**` — всё под config.
fn matches(path: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return path.starts_with(prefix);
    }
    if let Some(dir) = pattern.strip_suffix('/') {
        return path == dir || path.starts_with(&format!("{dir}/"));
    }
    path == pattern
}

/// Перевести старые списки в правила.
///
/// Порядок важен: `unmanaged` идёт последним, потому что он строже всех —
/// `config/xaero*` обязан побеждать общее правило для `config/`.
pub fn from_legacy(unmanaged: &[String], user_managed: &[String]) -> Vec<PathRule> {
    let rule = |pattern: &String, mode: PathMode| PathRule {
        pattern: pattern.clone(),
        mode,
        conflict: ConflictPolicy::default(),
    };
    user_managed
        .iter()
        .map(|p| rule(p, PathMode::UserManaged))
        .chain(unmanaged.iter().map(|p| rule(p, PathMode::Unmanaged)))
        .collect()
}

#[cfg(test)]
#[path = "path_rules_tests.rs"]
mod tests;
