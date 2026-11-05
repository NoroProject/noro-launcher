//! Config stamped into the binary after it has been built.
//!
//! A release is compiled with a placeholder in here. When the master hands out
//! that release it looks for the `NORO_CFG_START:` and `:NORO_CFG_END` markers
//! and rewrites the JSON between them with its own address, public key and
//! instance name.

pub const CFG_START: &[u8] = b"NORO_CFG_START:";
pub const CFG_END: &[u8] = b":NORO_CFG_END";

pub const EMBEDDED_CONFIG_LEN: usize = 512;

const fn make_placeholder() -> [u8; EMBEDDED_CONFIG_LEN] {
    let prefix = b"NORO_CFG_START:{\"master_url\":\"__NORO_URL_PLACEHOLDER__\",\"pubkey\":\"__NORO_PUBKEY_PLACEHOLDER__\"}";
    let suffix = b":NORO_CFG_END\0";
    let mut buf = [b' '; EMBEDDED_CONFIG_LEN];
    let mut i = 0;
    while i < prefix.len() {
        buf[i] = prefix[i];
        i += 1;
    }
    let suffix_start = EMBEDDED_CONFIG_LEN - suffix.len();
    let mut j = 0;
    while j < suffix.len() {
        buf[suffix_start + j] = suffix[j];
        j += 1;
    }
    buf
}

// Has to end up in the data section, at a fixed size, for the stamper to find
// it and overwrite it in place.
#[used]
#[no_mangle]
pub static mut NORO_EMBEDDED_CONFIG: [u8; EMBEDDED_CONFIG_LEN] = make_placeholder();

#[derive(Debug, Clone)]
pub struct EmbeddedConfig {
    pub master_url: String,
    pub pubkey: String,
    /// The instance's name at build time. Older masters don't stamp it.
    pub name: Option<String>,
}

pub fn parse_embedded_config_slice(slice: &[u8]) -> Option<EmbeddedConfig> {
    let start_pos = slice
        .windows(CFG_START.len())
        .position(|w| w == CFG_START)?;
    let content_start = start_pos + CFG_START.len();
    let end_pos = slice[content_start..]
        .windows(CFG_END.len())
        .position(|w| w == CFG_END)?;
    let raw = &slice[content_start..content_start + end_pos];
    let v: serde_json::Value = serde_json::from_slice(raw).ok()?;
    let master_url = v.get("master_url")?.as_str()?.trim().to_string();
    let pubkey = v.get("pubkey")?.as_str()?.trim().to_string();
    let name = v
        .get("name")
        .and_then(|n| n.as_str())
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());

    if master_url.is_empty() || master_url.contains("__NORO_URL_PLACEHOLDER__") {
        return None;
    }

    Some(EmbeddedConfig {
        master_url,
        pubkey,
        name,
    })
}

pub fn get_embedded_config() -> Option<EmbeddedConfig> {
    parse_embedded_config_slice(unsafe { &NORO_EMBEDDED_CONFIG[..] })
}

/// The name the splash shows and core starts with. The product's name when the
/// build wasn't stamped — the same fallback as `bridge::brand`, which this
/// binary doesn't link.
pub fn instance_name() -> String {
    get_embedded_config()
        .and_then(|cfg| cfg.name)
        .unwrap_or_else(|| "Noro".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unstamped_returns_none() {
        assert!(get_embedded_config().is_none());
    }

    #[test]
    fn parses_stamped_payload() {
        let stamped = b"NORO_CFG_START:{\"master_url\":\"https://test.example.com\",\"pubkey\":\"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"}                :NORO_CFG_END\0";
        let mut buf = [b' '; EMBEDDED_CONFIG_LEN];
        buf[..stamped.len()].copy_from_slice(stamped);
        let cfg = parse_embedded_config_slice(&buf).expect("should parse");
        assert_eq!(cfg.master_url, "https://test.example.com");
        assert_eq!(cfg.name, None);
        assert_eq!(
            cfg.pubkey,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn reads_the_stamped_name() {
        let stamped = b"NORO_CFG_START:{\"master_url\":\"https://test.example.com\",\"pubkey\":\"ab\",\"name\":\" Craft Land \"}:NORO_CFG_END";
        let cfg = parse_embedded_config_slice(stamped).expect("should parse");
        assert_eq!(cfg.name.as_deref(), Some("Craft Land"));
    }
}
