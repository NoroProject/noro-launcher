use super::*;

fn png(extra: usize) -> Bytes {
    let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
    v.extend(std::iter::repeat_n(0u8, extra));
    Bytes::from(v)
}

#[test]
fn name_of_valid_length_passes() {
    assert_eq!(validate_name(Some("  Cape  ".into())).unwrap(), "Cape");
}

/// The limit is in characters, not bytes — Cyrillic names used to hit it at
/// half the length the error message promises.
#[test]
fn cyrillic_is_measured_in_chars() {
    let long = "п".repeat(48);
    assert!(validate_name(Some(long)).is_ok());
    let too_long = "п".repeat(49);
    assert!(validate_name(Some(too_long)).is_err());
}

#[test]
fn short_and_empty_names_are_rejected() {
    assert!(validate_name(Some("a".into())).is_err());
    assert!(validate_name(Some("   ".into())).is_err());
    assert!(validate_name(None).is_err());
}

#[test]
fn non_png_is_rejected() {
    assert!(validate_png(None).is_err());
    assert!(validate_png(Some(Bytes::from_static(b"GIF89a"))).is_err());
    // Right signature, truncated: still short of the full eight bytes.
    assert!(validate_png(Some(Bytes::from_static(b"\x89PNG"))).is_err());
}

#[test]
fn png_passes_and_is_size_capped() {
    assert!(validate_png(Some(png(16))).is_ok());
    assert!(validate_png(Some(png(MAX_CAPE_BYTES))).is_err());
}
