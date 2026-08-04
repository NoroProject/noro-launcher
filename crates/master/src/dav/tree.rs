//! Виртуальное дерево сборки поверх `build_files`.
//!
//! Каталогов на диске нет: в базе лежат только пути файлов, а папки существуют
//! ровно постольку, поскольку в них что-то лежит. Отсюда две особенности,
//! которые видны наружу: пустой каталог создать нельзя (MKCOL — no-op), а
//! удаление последнего файла убирает и папку.

use crate::db::models::BuildFileRow;
use std::collections::BTreeSet;

/// Узел дерева в том виде, в каком его ждёт PROPFIND.
pub struct Node {
    pub name: String,
    pub is_dir: bool,
    pub size: i64,
    pub sha1: String,
}

/// Нормализует путь из URL: без ведущих и хвостовых слэшей, без `..`.
pub fn normalize(raw: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for part in raw.split('/') {
        match part {
            "" | "." => continue,
            // Наружу отдаём None: подниматься выше корня сборки нельзя.
            ".." => return None,
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// Существует ли такой каталог, то есть есть ли под ним хоть один файл.
pub fn is_dir(files: &[BuildFileRow], path: &str) -> bool {
    if path.is_empty() {
        return true;
    }
    let prefix = format!("{path}/");
    files.iter().any(|f| f.path.starts_with(&prefix))
}

pub fn find<'a>(files: &'a [BuildFileRow], path: &str) -> Option<&'a BuildFileRow> {
    files.iter().find(|f| f.path == path)
}

/// Прямые потомки каталога: файлы как есть, вложенные папки — по первому
/// сегменту, без повторов.
pub fn children(files: &[BuildFileRow], dir: &str) -> Vec<Node> {
    let prefix = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };

    let mut dirs: BTreeSet<String> = BTreeSet::new();
    let mut nodes: Vec<Node> = Vec::new();

    for file in files {
        let Some(rest) = file.path.strip_prefix(&prefix) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        match rest.split_once('/') {
            Some((head, _)) => {
                dirs.insert(head.to_string());
            }
            None => nodes.push(Node {
                name: rest.to_string(),
                is_dir: false,
                size: file.size,
                sha1: file.sha1.clone(),
            }),
        }
    }

    let mut out: Vec<Node> = dirs
        .into_iter()
        .map(|name| Node {
            name,
            is_dir: true,
            size: 0,
            sha1: String::new(),
        })
        .collect();
    out.append(&mut nodes);
    out
}

/// Все файлы внутри каталога — для DELETE и MOVE целой папки.
pub fn under<'a>(files: &'a [BuildFileRow], dir: &str) -> Vec<&'a BuildFileRow> {
    let prefix = format!("{dir}/");
    files
        .iter()
        .filter(|f| f.path == dir || f.path.starts_with(&prefix))
        .collect()
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
