//! Сверка игрового каталога с манифестом непосредственно перед запуском.
//!
//! `clean_extra` работает во время синка, а между синком и запуском каталог
//! никто не перепроверяет. Здесь та же проверка повторяется в последний момент:
//! лишнее удаляется, расхождения уезжают мастеру как флаг.
//!
//! Реакция намеренно тихая — «восстановлены файлы сборки» и запуск. Честный
//! игрок (битый диск, антивирус съел файл) ничего не замечает, а тот, кто
//! подкладывал мод намеренно, не получает подсказки «здесь есть проверка,
//! обходи её вот тут».

mod cache;
mod scan;

use crate::directories::safe_join;
use cache::HashCache;
use schema::{BuildManifest, IntegrityFinding, IntegrityKind, IntegrityReport, UserProfile};
use std::path::Path;

/// Сверить каталог и починить то, что чинится удалением.
pub async fn verify_before_launch(
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> IntegrityReport {
    let mut cache = HashCache::load(instance_dir).await;
    let mut findings = Vec::new();
    let mut checked = 0u32;

    let expected = scan::expected_files(manifest, enabled_optional, user);
    for f in &expected {
        let Some(path) = safe_join(instance_dir, &f.path) else {
            continue;
        };
        match cache.sha1_of(&path).await {
            None => findings.push(finding(IntegrityKind::MissingFile, &f.path, None, false)),
            Some(actual) if actual != f.sha1 => {
                checked += 1;
                findings.push(finding(
                    IntegrityKind::ModifiedFile,
                    &f.path,
                    Some(format!("ожидался {}, на диске {}", f.sha1, actual)),
                    false,
                ));
            }
            Some(_) => checked += 1,
        }
    }

    findings.extend(scan::remove_extras(instance_dir, manifest, &expected).await);
    findings.extend(scan::forbidden_optionals(manifest, enabled_optional, user));

    cache.save(instance_dir).await;

    IntegrityReport {
        server_id: manifest.server_id,
        build_id: manifest.build_id,
        build_version: manifest.version.clone(),
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        enabled_optional: enabled_optional.to_vec(),
        findings,
        checked_files: checked,
    }
}

fn finding(
    kind: IntegrityKind,
    subject: &str,
    detail: Option<String>,
    repaired: bool,
) -> IntegrityFinding {
    IntegrityFinding {
        kind,
        subject: subject.to_string(),
        detail,
        repaired,
    }
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
