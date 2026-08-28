//! Вшитая в бинарник конфигурация (In-Place Binary Stamping).
//!
//! При сборке релиза этот блок заполнен плейсхолдером. Мастер-сервер при
//! скачивании релиза находит маркеры `NORO_CFG_START:` и `:NORO_CFG_END`
//! и перезаписывает JSON реальным адресом и публичным ключом инстанса.

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

// 512 байт в секции данных
#[used]
#[no_mangle]
pub static mut NORO_EMBEDDED_CONFIG: [u8; EMBEDDED_CONFIG_LEN] = make_placeholder();

#[derive(Debug, Clone)]
pub struct EmbeddedConfig {
    pub master_url: String,
    pub pubkey: String,
}

pub fn parse_embedded_config_slice(slice: &[u8]) -> Option<EmbeddedConfig> {
    let start_pos = slice.windows(CFG_START.len()).position(|w| w == CFG_START)?;
    let content_start = start_pos + CFG_START.len();
    let end_pos = slice[content_start..].windows(CFG_END.len()).position(|w| w == CFG_END)?;
    let raw = &slice[content_start..content_start + end_pos];
    let v: serde_json::Value = serde_json::from_slice(raw).ok()?;
    let master_url = v.get("master_url")?.as_str()?.trim().to_string();
    let pubkey = v.get("pubkey")?.as_str()?.trim().to_string();

    if master_url.is_empty() || master_url.contains("__NORO_URL_PLACEHOLDER__") {
        return None;
    }

    Some(EmbeddedConfig { master_url, pubkey })
}

pub fn get_embedded_config() -> Option<EmbeddedConfig> {
    parse_embedded_config_slice(unsafe { &NORO_EMBEDDED_CONFIG[..] })
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
        let stamped = b"NORO_CFG_START:{\"master_url\":\"https://test.noro.dev\",\"pubkey\":\"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"}                :NORO_CFG_END\0";
        let mut buf = [b' '; EMBEDDED_CONFIG_LEN];
        buf[..stamped.len()].copy_from_slice(stamped);
        let cfg = parse_embedded_config_slice(&buf).expect("должен распарсить");
        assert_eq!(cfg.master_url, "https://test.noro.dev");
        assert_eq!(cfg.pubkey, "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    }
}

