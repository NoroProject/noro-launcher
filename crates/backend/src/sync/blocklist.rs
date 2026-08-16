//! Применение базы запрещённых файлов.
//!
//! Проверяется и внутри `unmanaged`, и внутри `user_managed`: в этом весь
//! смысл — папка ресурспаков не синхронизируется, но xray оттуда удаляется.
//! Поэтому обход идёт по всему инстансу, а не по managed-путям.
//!
//! `saves/` исключён жёстко: там гигабайты, а запрещённых файлов не бывает.

use schema::{BlockAction, BlockedFile, IntegrityFinding, IntegrityKind};
use std::path::Path;

/// Куда не ходим ни при каких правилах: дорого и бессмысленно.
const SKIP_DIRS: [&str; 4] = ["saves", "assets", "libraries", "logs"];

/// Что нашли и что сделали.
#[derive(Default)]
pub struct Report {
    pub findings: Vec<IntegrityFinding>,
    /// Нашёлся файл с действием `block_launch`.
    pub block_launch: bool,
}

/// Пройти инстанс и применить правила.
pub async fn enforce(instance_dir: &Path, rules: &[BlockedFile]) -> Report {
    let mut report = Report::default();
    if rules.is_empty() {
        return report;
    }

    for (rel, path) in candidates(instance_dir).await {
        let Ok(sha1) = super::integrity::sha1_file(&path).await else {
            continue;
        };
        let Some(rule) = schema::first_match(rules, &rel, &sha1) else {
            continue;
        };

        let repaired = match rule.action {
            BlockAction::Delete => tokio::fs::remove_file(&path).await.is_ok(),
            BlockAction::Flag => false,
            BlockAction::BlockLaunch => {
                report.block_launch = true;
                false
            }
        };
        tracing::warn!(path = %rel, reason = %rule.reason, "запрещённый файл");
        report.findings.push(IntegrityFinding {
            kind: IntegrityKind::ExtraFile,
            subject: rel,
            detail: Some(rule.reason.clone()),
            repaired,
        });
    }
    report
}

/// Файлы, которые вообще имеет смысл проверять.
async fn candidates(instance_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let root = instance_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Служебное лаунчера и заведомо тяжёлое — мимо.
                !name.starts_with(".noro") && !SKIP_DIRS.contains(&name.as_ref())
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let rel = e
                    .path()
                    .strip_prefix(&root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                Some((rel, e.path().to_path_buf()))
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
#[path = "blocklist_tests.rs"]
mod tests;
