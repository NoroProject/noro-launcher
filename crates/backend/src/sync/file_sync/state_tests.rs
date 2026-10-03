use super::*;
use crate::sync::verify::fixtures::{entry, manifest, OK_SHA1};
use bridge::BuildState;

fn build(mod_sha1: &str) -> BuildManifest {
    let mut m = manifest(vec![entry("mods/a.jar", mod_sha1, ArtifactKind::Mod)]);
    m.version = "1.0.0".into();
    m
}

fn instance(marker: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noro-state-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    if let Some(marker) = marker {
        std::fs::write(version_marker(&dir), marker).unwrap();
    }
    dir
}

#[test]
fn a_synced_build_is_ready() {
    let m = build(OK_SHA1);
    let dir = instance(Some(&marker_contents(&m)));
    assert_eq!(build_state(&dir, &m), BuildState::Ready);
}

/// Re-imported in place: same version, other files. The version alone said
/// «Play» here.
#[test]
fn same_version_with_other_files_is_outdated() {
    let dir = instance(Some(&marker_contents(&build(OK_SHA1))));
    let reimported = build("0000000000000000000000000000000000000000");
    assert_eq!(build_state(&dir, &reimported), BuildState::Outdated);
}

/// Written before the fingerprint existed: only the version to go on.
#[test]
fn an_old_marker_compares_the_version() {
    let m = build(OK_SHA1);
    assert_eq!(build_state(&instance(Some("1.0.0")), &m), BuildState::Ready);
    assert_eq!(
        build_state(&instance(Some("0.9.0")), &m),
        BuildState::Outdated
    );
}

#[test]
fn without_a_marker_nothing_is_installed() {
    let dir = instance(None);
    assert_eq!(build_state(&dir, &build(OK_SHA1)), BuildState::Missing);
    assert_eq!(installed_state(&dir), BuildState::Missing);
    assert_eq!(installed_state(&instance(Some("1.0.0"))), BuildState::Ready);
}
