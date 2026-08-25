//! Подмена содержимого тома — шаг, после которого текущих файлов уже нет.
//! Здесь проверяется, что нового не потеряли, старое не удалили, а наружу тома
//! ничего не уехало.

use super::swap;
use std::path::PathBuf;

fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("noro-swap-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

#[test]
fn путь_наружу_тома_не_распаковывается() {
    assert_eq!(
        swap::safe_rel("data/skins/a.png"),
        Some("skins/a.png".into())
    );
    assert_eq!(swap::safe_rel("data/../../etc/passwd"), None);
    assert_eq!(swap::safe_rel("data//../x"), None);
    assert_eq!(swap::safe_rel("/etc/passwd"), None);
    // meta.json и signature разворачивать некуда — они не под data/.
    assert_eq!(swap::safe_rel("meta.json"), None);
    assert_eq!(swap::safe_rel("data/"), None);
}

#[test]
fn подмена_переносит_новое_и_откладывает_старое() {
    let root = root("swap");
    let data = root.join("data");
    let staged = super::work_dir(&data).join("restore-X");
    std::fs::create_dir_all(data.join("builds")).unwrap();
    std::fs::create_dir_all(staged.join("builds")).unwrap();

    std::fs::write(data.join("builds/old.jar"), b"old").unwrap();
    std::fs::write(data.join("skin.png"), b"old skin").unwrap();
    std::fs::write(staged.join("builds/new.jar"), b"new").unwrap();

    swap::swap(&data, &staged, "X").unwrap();

    // Новое встало на место.
    assert_eq!(std::fs::read(data.join("builds/new.jar")).unwrap(), b"new");
    // Прежнего в томе нет, но оно не удалено — лежит рядом и откатывается руками.
    assert!(!data.join("skin.png").exists());
    let old = swap::old_dir(&data, "X");
    assert_eq!(std::fs::read(old.join("skin.png")).unwrap(), b"old skin");
    assert_eq!(std::fs::read(old.join("builds/old.jar")).unwrap(), b"old");
    // Каталог сборок подменён целиком, а не слит с прежним.
    assert!(!data.join("builds/old.jar").exists());
    // Staging убран за собой.
    assert!(!staged.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn служебный_каталог_переживает_подмену() {
    let root = root("work");
    let data = root.join("data");
    let work = super::work_dir(&data);
    let staged = work.join("restore-Y");
    std::fs::create_dir_all(&staged).unwrap();
    std::fs::write(work.join("upload-keep.tar.gz"), b"archive").unwrap();
    std::fs::write(data.join("a.txt"), b"a").unwrap();
    std::fs::write(staged.join("b.txt"), b"b").unwrap();

    swap::swap(&data, &staged, "Y").unwrap();

    // Сам архив, из которого идёт восстановление, лежит в служебном каталоге:
    // уехал бы он в old-*, и распаковывать было бы уже нечего.
    assert_eq!(
        std::fs::read(work.join("upload-keep.tar.gz")).unwrap(),
        b"archive"
    );
    assert!(data.join("b.txt").exists());
    assert!(!data.join(super::WORK).join(super::WORK).exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn уборка_сносит_только_протухшее() {
    let root = root("sweep");
    let work = super::work_dir(&root.join("data"));
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("fresh.tar.gz"), b"x").unwrap();

    // Только что созданный файл ещё нужен — его как раз разбирают.
    swap::sweep(&work, 6).unwrap();
    assert!(work.join("fresh.tar.gz").exists());

    // При нулевом сроке (нижняя граница — час) свежий файл всё равно остаётся:
    // иначе уборка сносила бы загрузку прямо во время разбора.
    swap::sweep(&work, 0).unwrap();
    assert!(work.join("fresh.tar.gz").exists());

    let _ = std::fs::remove_dir_all(&root);
}
