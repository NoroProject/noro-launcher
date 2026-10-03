//! What went wrong, in words a player can act on.
//!
//! A sync error is a chain of contexts ending in an io or network error. The
//! window used to show that chain as is — "download of https://… failed: error
//! sending request" — which says nothing to someone who only wants to play.
//! The window gets a translation key for the cause it can name, and the chain
//! separately, for the console and support.

/// The cause, as a translation key.
pub fn sync_failure_key(e: &anyhow::Error) -> &'static str {
    for cause in e.chain() {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            if is_disk_full(io) {
                return "sync-error-disk-full";
            }
            if io.kind() == std::io::ErrorKind::PermissionDenied {
                return "sync-error-permission";
            }
        }
        if let Some(http) = cause.downcast_ref::<reqwest::Error>() {
            if http.is_connect() || http.is_timeout() || http.is_request() || http.is_body() {
                return "sync-error-network";
            }
        }
    }
    let text = format!("{e:#}").to_lowercase();
    if text.contains("signature") {
        "sync-error-signature"
    } else if text.contains("sha1 mismatch") {
        "sync-error-corrupt"
    } else if text.contains("authlib") {
        "sync-error-authlib"
    } else if text.contains("interrupted") || text.contains("connection") {
        "sync-error-network"
    } else {
        "sync-error-unknown"
    }
}

/// The JVM didn't start: a broken runtime, a bad flag, an antivirus.
pub const LAUNCH_FAILED: &str = "sync-error-launch";
pub const NOT_SIGNED_IN: &str = "sync-error-signed-out";
pub const LAUNCH_BLOCKED: &str = "notif-launch-blocked";

fn is_disk_full(e: &std::io::Error) -> bool {
    // ENOSPC on Unix, ERROR_DISK_FULL / ERROR_HANDLE_DISK_FULL on Windows.
    e.kind() == std::io::ErrorKind::StorageFull
        || matches!(e.raw_os_error(), Some(28) | Some(112) | Some(39))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn a_full_disk_is_named_whatever_the_context() {
        let io = std::io::Error::from_raw_os_error(28);
        let e = Err::<(), _>(io)
            .context("writing mods/a.jar")
            .context("download of https://example.com/a.jar failed")
            .unwrap_err();
        assert_eq!(sync_failure_key(&e), "sync-error-disk-full");
    }

    #[test]
    fn known_messages_map_to_their_keys() {
        let e = anyhow::anyhow!("manifest signature is invalid, sync aborted");
        assert_eq!(sync_failure_key(&e), "sync-error-signature");
        let e = anyhow::anyhow!("SHA1 mismatch for x: expected a, got b");
        assert_eq!(sync_failure_key(&e), "sync-error-corrupt");
        let e = anyhow::anyhow!("something else entirely");
        assert_eq!(sync_failure_key(&e), "sync-error-unknown");
    }
}
