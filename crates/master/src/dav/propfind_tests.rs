use super::*;

fn file(path: &str, size: i64) -> Resource {
    Resource {
        path: path.into(),
        is_dir: false,
        size,
        etag: "abc123".into(),
    }
}

fn dir(path: &str) -> Resource {
    Resource {
        path: path.into(),
        is_dir: true,
        size: 0,
        etag: String::new(),
    }
}

#[test]
fn collection_has_no_content_length() {
    let xml = multistatus("/dav/b1", &[dir("config")]);
    assert!(xml.contains("<D:collection/>"));
    // Finder принимает папку за пустой файл, если у неё есть длина.
    assert!(!xml.contains("getcontentlength"));
    // Каталог обязан заканчиваться слэшем.
    assert!(xml.contains("<D:href>/dav/b1/config/</D:href>"), "{xml}");
}

#[test]
fn file_carries_size_and_etag() {
    let xml = multistatus("/dav/b1", &[file("options.txt", 42)]);
    assert!(xml.contains("<D:getcontentlength>42</D:getcontentlength>"));
    assert!(xml.contains("&quot;abc123&quot;"));
    assert!(xml.contains("<D:href>/dav/b1/options.txt</D:href>"));
}

#[test]
fn root_href_is_the_base_with_slash() {
    let xml = multistatus("/dav/b1", &[dir("")]);
    assert!(xml.contains("<D:href>/dav/b1/</D:href>"), "{xml}");
}

#[test]
fn special_characters_are_encoded_not_injected() {
    // Имя от админа может содержать что угодно; в XML оно не должно ломать разметку.
    let xml = multistatus("/dav/b1", &[file("config/a&b<c>.toml", 1)]);
    assert!(!xml.contains("a&b<c>"), "сырые спецсимволы в XML: {xml}");
    assert!(xml.contains("a%26b%3Cc%3E.toml"), "{xml}");
}

#[test]
fn spaces_in_names_are_percent_encoded() {
    let xml = multistatus("/dav/b1", &[file("config/my mod.toml", 1)]);
    assert!(xml.contains("my%20mod.toml"), "{xml}");
}
