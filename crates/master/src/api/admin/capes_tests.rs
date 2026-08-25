use super::*;

fn png(extra: usize) -> Bytes {
    let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
    v.extend(std::iter::repeat_n(0u8, extra));
    Bytes::from(v)
}

#[test]
fn имя_нужной_длины_проходит() {
    assert_eq!(validate_name(Some("  Cape  ".into())).unwrap(), "Cape");
}

/// Длина считается символами. По байтам «Плащ» — это восемь, и кириллическое
/// название упиралось в потолок вдвое раньше, чем обещает сообщение об ошибке.
#[test]
fn кириллица_меряется_символами() {
    let long = "п".repeat(48);
    assert!(validate_name(Some(long)).is_ok());
    let too_long = "п".repeat(49);
    assert!(validate_name(Some(too_long)).is_err());
}

#[test]
fn слишком_короткое_и_пустое_имя_отвергаются() {
    assert!(validate_name(Some("a".into())).is_err());
    assert!(validate_name(Some("   ".into())).is_err());
    assert!(validate_name(None).is_err());
}

#[test]
fn не_png_отвергается() {
    assert!(validate_png(None).is_err());
    assert!(validate_png(Some(Bytes::from_static(b"GIF89a"))).is_err());
    // Правильная сигнатура, но обрезанная: восьми байт ещё нет.
    assert!(validate_png(Some(Bytes::from_static(b"\x89PNG"))).is_err());
}

#[test]
fn png_проходит_и_режется_по_размеру() {
    assert!(validate_png(Some(png(16))).is_ok());
    assert!(validate_png(Some(png(MAX_CAPE_BYTES))).is_err());
}
