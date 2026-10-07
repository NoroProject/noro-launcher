use super::*;

#[test]
fn store_path_formats_correctly() {
    let root = Path::new("/test/store");
    assert_eq!(
        store_path(root, "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        Some(PathBuf::from(
            "/test/store/da/da39a3ee5e6b4b0d3255bfef95601890afd80709"
        ))
    );
    assert_eq!(store_path(root, "bad"), None);
    assert_eq!(store_path(root, "zzzz"), None);
}

#[test]
fn cacheable_kinds_classified() {
    assert!(is_cacheable(ArtifactKind::Mod));
    assert!(is_cacheable(ArtifactKind::Java));
    assert!(is_cacheable(ArtifactKind::Library));
    assert!(is_cacheable(ArtifactKind::Asset));
    assert!(is_cacheable(ArtifactKind::Config));
    assert!(!is_cacheable(ArtifactKind::Other));
}

#[tokio::test]
async fn links_and_verifies_from_store() {
    let dir = crate::test_http::TempDir::new("store-test");
    let store_root = dir.path().join("store");
    let instance_dir = dir.path().join("instance");
    tokio::fs::create_dir_all(&store_root).await.unwrap();

    let sha1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    let stored_file = store_path(&store_root, sha1).unwrap();
    tokio::fs::create_dir_all(stored_file.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&stored_file, b"hello world")
        .await
        .unwrap();

    let dest = instance_dir.join("mods/test.jar");
    let linked = try_link_from_store(&store_root, sha1, 11, &dest, false).await;
    assert!(linked);
    assert_eq!(tokio::fs::read(&dest).await.unwrap(), b"hello world");

    // Size mismatch returns false
    let bad_dest = instance_dir.join("mods/bad.jar");
    let bad_linked = try_link_from_store(&store_root, sha1, 999, &bad_dest, false).await;
    assert!(!bad_linked);
    assert!(!bad_dest.exists());
}

#[tokio::test]
async fn overwrites_existing_dest_and_sets_executable() {
    let dir = crate::test_http::TempDir::new("store-overwrite-test");
    let store_root = dir.path().join("store");
    let instance_dir = dir.path().join("instance");
    let sha1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    let stored_file = store_path(&store_root, sha1).unwrap();
    tokio::fs::create_dir_all(stored_file.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&stored_file, b"executable content")
        .await
        .unwrap();

    let dest = instance_dir.join("bin/java");
    tokio::fs::create_dir_all(dest.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&dest, b"old partial").await.unwrap();

    let linked = try_link_from_store(&store_root, sha1, 18, &dest, true).await;
    assert!(linked);
    assert_eq!(tokio::fs::read(&dest).await.unwrap(), b"executable content");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = tokio::fs::metadata(&dest).await.unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111);
    }
}

#[tokio::test]
async fn lock_for_sha1_returns_shared_lock() {
    let sha1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    let lock1 = lock_for_sha1(sha1);
    let lock2 = lock_for_sha1(sha1);
    assert!(Arc::ptr_eq(&lock1, &lock2));

    let other_sha1 = "0123456789abcdef0123456789abcdef01234567";
    let lock3 = lock_for_sha1(other_sha1);
    assert!(!Arc::ptr_eq(&lock1, &lock3));
}
