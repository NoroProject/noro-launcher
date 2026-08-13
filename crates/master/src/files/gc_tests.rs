//! Разбор имени объекта. Ошибка здесь означает либо удаление живого файла,
//! либо вечно растущий мусор, поэтому границы проверяются явно.

use super::*;

#[test]
fn accepts_only_lowercase_forty_hex_names() {
    assert!(is_sha1("da39a3ee5e6b4b0d3255bfef95601890afd80709"));

    assert!(
        !is_sha1("da39a3ee5e6b4b0d3255bfef95601890afd807"),
        "39 символов"
    );
    assert!(
        !is_sha1("da39a3ee5e6b4b0d3255bfef95601890afd8070900"),
        "42 символа"
    );
    assert!(
        !is_sha1("DA39A3EE5E6B4B0D3255BFEF95601890AFD80709"),
        "верхний регистр"
    );
    assert!(
        !is_sha1("da39a3ee5e6b4b0d3255bfef95601890afd8070g"),
        "не hex"
    );
    assert!(!is_sha1(""), "пустое имя");
}

/// Временные файлы и всё, что не похоже на объект хранилища, обходятся
/// стороной — их сборщик трогать не должен.
#[test]
fn ignores_names_that_are_not_objects() {
    assert!(!is_sha1("tmp-upload.part"));
    assert!(!is_sha1("mods"));
    assert!(!is_sha1("da39a3ee5e6b4b0d3255bfef95601890afd80709.tmp"));
}

#[test]
fn identifiers_are_escaped_for_sql() {
    assert_eq!(quote_ident("build_files"), "\"build_files\"");
    assert_eq!(quote_ident("we\"ird"), "\"we\"\"ird\"");
}
