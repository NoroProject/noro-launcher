//! Тесты имени файла. Оно приходит из чужих рук и становится путём в манифесте
//! сборки и путём на диске игрового сервера — ошибка здесь стоит дорого.

use super::resolve::safe_filename;

#[test]
fn percent_encoding_from_a_url_is_decoded() {
    // Плюс в версии Modrinth в ссылке закодирован. Без разбора в манифест
    // уезжало `sodium%2Bmc1.21.1.jar`, и лаунчер качал файл с таким именем.
    assert_eq!(
        safe_filename("sodium-fabric-0.8.13%2Bmc1.21.1.jar"),
        "sodium-fabric-0.8.13+mc1.21.1.jar"
    );
}

#[test]
fn path_separators_are_stripped() {
    assert_eq!(safe_filename("../../etc/passwd"), "passwd");
    assert_eq!(safe_filename("mods/nested/thing.jar"), "thing.jar");
    assert_eq!(safe_filename("C:\\windows\\evil.jar"), "evil.jar");
}

#[test]
fn encoded_separators_are_stripped_too() {
    // Декодирование идёт первым, иначе `%2F` проскочил бы проверку.
    assert_eq!(safe_filename("a%2F..%2Fboot.jar"), "boot.jar");
}

#[test]
fn empty_and_dotted_names_fall_back() {
    assert_eq!(safe_filename(""), "mod.jar");
    assert_eq!(safe_filename("..."), "mod.jar");
    assert_eq!(safe_filename("mods/"), "mod.jar");
}
