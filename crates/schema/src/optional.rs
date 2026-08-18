//! Правила включения опциональных модов: зависимости и конфликты.
//!
//! Живут в `schema`, потому что решают один и тот же вопрос в двух местах:
//! лаунчер гасит галочку и объясняет почему, мастер проверяет присланный
//! список. Разъедься эти проверки — и лаунчер разрешал бы то, что мастер
//! отвергает, а игрок видел бы «мод включён», которого в игре нет.
//!
//! Конфликт здесь запрещает включение, а не выключает соседа молча: игрок сам
//! решает, чем из двух несовместимых модов пожертвовать, и подмена его выбора
//! выглядит как сбой лаунчера.

use crate::build::OptionalMod;
use serde::{Deserialize, Serialize};

/// Что мешает включить мод.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionIssue {
    /// Мод несовместим с уже включённым.
    Conflict { mod_name: String, with: String },
    /// Моду нужен другой мод, которого нет в выборе.
    MissingDependency { mod_name: String, needs: String },
}

impl SelectionIssue {
    /// Имя мода, из-за которого выбор не сходится.
    pub fn mod_name(&self) -> &str {
        match self {
            SelectionIssue::Conflict { mod_name, .. } => mod_name,
            SelectionIssue::MissingDependency { mod_name, .. } => mod_name,
        }
    }
}

/// Можно ли включить мод поверх текущего выбора.
///
/// Конфликт проверяется в обе стороны: несовместимость записана у одного из
/// пары, и требовать её у обоих значило бы ловить половину случаев.
pub fn can_enable(
    mods: &[OptionalMod],
    enabled: &[String],
    name: &str,
) -> Result<(), SelectionIssue> {
    let Some(target) = find(mods, name) else {
        return Ok(());
    };
    for other in enabled.iter().filter(|n| n.as_str() != name) {
        if conflicting(target, other) || find(mods, other).is_some_and(|m| conflicting(m, name)) {
            return Err(SelectionIssue::Conflict {
                mod_name: name.to_string(),
                with: other.clone(),
            });
        }
    }
    for needed in &target.dependencies {
        if !enabled.iter().any(|n| n == needed) {
            return Err(SelectionIssue::MissingDependency {
                mod_name: name.to_string(),
                needs: needed.clone(),
            });
        }
    }
    Ok(())
}

/// Всё, что не сходится в готовом выборе.
///
/// Нужна там, где список приходит целиком: при загрузке сохранённого выбора и
/// на мастере. Одной проверки «можно ли включить» тут мало — сборку могли
/// изменить после того, как игрок собрал набор.
pub fn issues(mods: &[OptionalMod], enabled: &[String]) -> Vec<SelectionIssue> {
    let mut found = Vec::new();
    for name in enabled {
        let Some(m) = find(mods, name) else {
            continue;
        };
        for other in enabled.iter().filter(|n| n.as_str() != name.as_str()) {
            if conflicting(m, other) && !already(&found, name, other) {
                found.push(SelectionIssue::Conflict {
                    mod_name: name.clone(),
                    with: other.clone(),
                });
            }
        }
        for needed in &m.dependencies {
            if !enabled.iter().any(|n| n == needed) {
                found.push(SelectionIssue::MissingDependency {
                    mod_name: name.clone(),
                    needs: needed.clone(),
                });
            }
        }
    }
    found
}

/// Пара уже отмечена — с другой стороны. Иначе один конфликт давал бы две
/// одинаковые по смыслу записи и два сообщения игроку.
fn already(found: &[SelectionIssue], name: &str, other: &str) -> bool {
    found.iter().any(|issue| match issue {
        SelectionIssue::Conflict { mod_name, with } => {
            (mod_name == name && with == other) || (mod_name == other && with == name)
        }
        SelectionIssue::MissingDependency { .. } => false,
    })
}

fn conflicting(m: &OptionalMod, other: &str) -> bool {
    m.conflicts.iter().any(|c| c == other)
}

fn find<'a>(mods: &'a [OptionalMod], name: &str) -> Option<&'a OptionalMod> {
    mods.iter().find(|m| m.name == name)
}

#[cfg(test)]
#[path = "optional_tests.rs"]
mod tests;
