use super::*;

/// Свой каталог на каждый тест: стор адресуется по содержимому, и общий
/// временный каталог сделал бы тесты зависимыми друг от друга.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noro-store-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[tokio::test]
async fn put_bytes_is_content_addressed() {
    let dir = scratch("addressed");
    let store = FileStore::new(&dir);

    let first = store.put_bytes(b"hello").await.expect("сохранение");
    let again = store.put_bytes(b"hello").await.expect("повторное");

    assert_eq!(first.sha1, again.sha1);
    assert_eq!(first.size, 5);
    assert!(store.exists(&first.sha1));

    let _ = std::fs::remove_dir_all(&dir);
}

/// Обычная запись не перезаписывает уже лежащее — и потому не лечит побитый
/// блоб. Тест закрепляет это как ожидаемое поведение: именно из-за него
/// пересбору «с нуля» нужен отдельный, перезаписывающий путь.
#[tokio::test]
async fn plain_put_leaves_a_corrupted_blob_alone() {
    let dir = scratch("corrupt-kept");
    let store = FileStore::new(&dir);

    let stored = store.put_bytes(b"real content").await.expect("сохранение");
    std::fs::write(store.path_for(&stored.sha1), b"garbage").expect("порча блоба");

    store.put_bytes(b"real content").await.expect("повторное");

    let on_disk = std::fs::read(store.path_for(&stored.sha1)).expect("чтение");
    assert_eq!(on_disk, b"garbage", "обычная запись не должна была тронуть блоб");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn overwriting_put_repairs_a_corrupted_blob() {
    let dir = scratch("corrupt-fixed");
    let store = FileStore::new(&dir);

    let stored = store.put_bytes(b"real content").await.expect("сохранение");
    std::fs::write(store.path_for(&stored.sha1), b"garbage").expect("порча блоба");

    let fixed = store
        .put_bytes_overwriting(b"real content")
        .await
        .expect("перезапись");

    assert_eq!(fixed.sha1, stored.sha1, "путь берётся из содержимого");
    let on_disk = std::fs::read(store.path_for(&stored.sha1)).expect("чтение");
    assert_eq!(on_disk, b"real content");

    let _ = std::fs::remove_dir_all(&dir);
}
