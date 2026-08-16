//! Ошибка здесь удаляет файлы игрока или, наоборот, пропускает подложенный
//! мод, поэтому проверяются обе стороны: что лишнее уходит и что своё остаётся.

use super::fixtures::*;
use super::verify_before_launch;
use schema::{ArtifactKind, IntegrityKind};

#[tokio::test]
async fn a_matching_directory_produces_no_findings() {
    let dir = Scratch::new("matching");
    write(dir.path(), "mods/core.jar", "ok").await;
    let m = manifest(vec![entry("mods/core.jar", OK_SHA1, ArtifactKind::Mod)]);

    let report = verify_before_launch(dir.path(), &m, &[], &player()).await;

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert_eq!(report.checked_files, 1);
}

#[tokio::test]
async fn a_planted_mod_is_deleted_and_reported() {
    let dir = Scratch::new("planted");
    write(dir.path(), "mods/core.jar", "ok").await;
    write(dir.path(), "mods/xray.jar", "нет в манифесте").await;
    let m = manifest(vec![entry("mods/core.jar", OK_SHA1, ArtifactKind::Mod)]);

    let report = verify_before_launch(dir.path(), &m, &[], &player()).await;

    assert_eq!(
        subjects(&report, IntegrityKind::ExtraFile),
        ["mods/xray.jar"]
    );
    assert!(report.findings[0].repaired);
    assert!(!dir.path().join("mods/xray.jar").exists());
    // Файл сборки не тронут.
    assert!(dir.path().join("mods/core.jar").exists());
}

#[tokio::test]
async fn a_replaced_mod_is_reported_but_kept() {
    let dir = Scratch::new("replaced");
    write(dir.path(), "mods/core.jar", "подменён").await;
    let m = manifest(vec![entry("mods/core.jar", OK_SHA1, ArtifactKind::Mod)]);

    let report = verify_before_launch(dir.path(), &m, &[], &player()).await;

    assert_eq!(
        subjects(&report, IntegrityKind::ModifiedFile),
        ["mods/core.jar"]
    );
    // Удалять нельзя: следующий синк докачает правильный, а до тех пор файл
    // нужен игре.
    assert!(dir.path().join("mods/core.jar").exists());
}

#[tokio::test]
async fn a_missing_file_is_reported() {
    let dir = Scratch::new("missing");
    let m = manifest(vec![entry("mods/core.jar", OK_SHA1, ArtifactKind::Mod)]);

    let report = verify_before_launch(dir.path(), &m, &[], &player()).await;

    assert_eq!(
        subjects(&report, IntegrityKind::MissingFile),
        ["mods/core.jar"]
    );
}

#[tokio::test]
async fn a_disabled_optional_mod_is_not_extra() {
    let dir = Scratch::new("disabled-optional");
    write(dir.path(), "mods/optifine.jar", "ok").await;
    let mut m = manifest(vec![entry("mods/optifine.jar", OK_SHA1, ArtifactKind::Mod)]);
    m.optional_mods
        .push(optional("optifine", false, &["mods/optifine.jar"]));

    // Мод выключен: файл лежит и ждёт включения, это не находка.
    let report = verify_before_launch(dir.path(), &m, &["sodium".into()], &player()).await;

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[tokio::test]
async fn a_selection_left_over_from_an_older_build_is_not_flagged() {
    // Админ убрал мод из сборки, а выбор игрока сохранён с прошлого запуска.
    // Флаг тут был бы ложным: файлов мода у игрока всё равно нет.
    let dir = Scratch::new("stale-selection");
    let m = manifest(vec![]);

    let report = verify_before_launch(dir.path(), &m, &["удалённый".into()], &player()).await;

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[tokio::test]
async fn unmanaged_paths_are_left_alone() {
    let dir = Scratch::new("unmanaged");
    write(dir.path(), "config/xaero_waypoints.txt", "точки игрока").await;
    let mut m = manifest(vec![]);
    m.unmanaged_paths.push("config/xaero*".into());

    let report = verify_before_launch(dir.path(), &m, &[], &player()).await;

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(dir.path().join("config/xaero_waypoints.txt").exists());
}

#[tokio::test]
async fn an_enabled_limited_mod_without_permission_is_flagged() {
    let dir = Scratch::new("limited-no-permission");
    let mut m = manifest(vec![]);
    m.optional_mods
        .push(optional("staff", true, &["mods/staff.jar"]));

    let report = verify_before_launch(dir.path(), &m, &["staff".into()], &player()).await;

    assert_eq!(
        subjects(&report, IntegrityKind::ForbiddenOptionalMod),
        ["staff"]
    );
}
