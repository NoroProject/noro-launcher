//! Проверка путей внутри сборки.
//!
//! Эти пути лаунчер раскладывает по диску у игрока через `instance_dir.join()`,
//! поэтому каждый пропущенный сюда случай — это запись мимо инстанса на чужой
//! машине. Отсюда перечисление всех форм, а не один пример с `..`.

use super::safe_path;

#[test]
fn обычный_путь_проходит() {
    assert_eq!(
        safe_path("mods/sodium.jar").as_deref(),
        Some("mods/sodium.jar")
    );
    assert_eq!(
        safe_path("config/a/b.toml").as_deref(),
        Some("config/a/b.toml")
    );
}

#[test]
fn ведущее_точка_слэш_срезается() {
    assert_eq!(safe_path("./mods/x.jar").as_deref(), Some("mods/x.jar"));
}

/// Windows принимает `\` как разделитель наравне с `/`. Пока проверка искала
/// `..` только между `/`, путь `..\..\evil.exe` выглядел одним безобидным
/// именем файла — и на Linux им и был, а на машине игрока уезжал на два уровня
/// вверх.
#[test]
fn обратный_слэш_не_прячет_выход_вверх() {
    assert_eq!(safe_path(r"..\..\evil.exe"), None);
    assert_eq!(safe_path(r"mods\..\..\evil.exe"), None);
}

#[test]
fn выход_вверх_отвергается() {
    assert_eq!(safe_path("../evil"), None);
    assert_eq!(safe_path("mods/../../evil"), None);
    assert_eq!(safe_path(".."), None);
}

#[test]
fn абсолютный_путь_отвергается() {
    assert_eq!(safe_path("/etc/passwd"), None);
    assert_eq!(safe_path(r"\windows\system32\x.dll"), None);
}

/// `C:` — это не имя папки, а корень диска: на Windows `join("C:/x")` заменяет
/// весь путь целиком, а не дописывается к нему.
#[test]
fn буква_диска_отвергается() {
    assert_eq!(safe_path("C:/Windows/x.dll"), None);
    assert_eq!(safe_path(r"C:\Windows\x.dll"), None);
}

#[test]
fn пустые_и_пробельные_сегменты_отвергаются() {
    assert_eq!(safe_path(""), None);
    assert_eq!(safe_path("   "), None);
    assert_eq!(safe_path("mods//x.jar"), None);
    assert_eq!(safe_path("mods/ /x.jar"), None);
}
