// Over 150 lines: the tests take up a third; the download and its checks are
// one function.
use crate::config::LauncherConfig;
use crate::directories::LauncherDirectories;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Downloads authlib-injector from the master server so the launcher, wrapper
/// and node all run the exact same version. Before this the launcher fetched
/// the jar straight from upstream, causing version mismatches whenever the
/// upstream released a new build.
///
/// The copy on disk is revalidated on every launch with its ETag, so a new
/// version on the master reaches players without a launcher update; a 304 is
/// one small request. With the master unreachable the cached copy is used.
pub async fn ensure_authlib_injector(
    client: &reqwest::Client,
    config: &LauncherConfig,
    dirs: &LauncherDirectories,
) -> Result<PathBuf> {
    let path = dirs.authlib_injector();
    let etag_path = etag_path(&path);
    let cached = is_jar(&path).await;
    let etag = if cached {
        tokio::fs::read_to_string(&etag_path).await.ok()
    } else {
        None
    };

    let url = format!(
        "{}/api/agent/authlib-injector.jar",
        config.master_url.trim_end_matches('/')
    );
    let mut req = client.get(&url);
    if let Some(etag) = etag.as_deref() {
        req = req.header(reqwest::header::IF_NONE_MATCH, etag.trim());
    }
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) if cached => {
            tracing::warn!("authlib-injector: master unreachable ({e}), using the cached copy");
            return Ok(path);
        }
        Err(e) => return Err(e).context("authlib-injector: request failed"),
    };
    if resp.status() == reqwest::StatusCode::NOT_MODIFIED && cached {
        return Ok(path);
    }
    let resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(e) if cached => {
            tracing::warn!("authlib-injector: {e}, using the cached copy");
            return Ok(path);
        }
        Err(e) => return Err(e).context("authlib-injector: bad status"),
    };
    let new_etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp
        .bytes()
        .await
        .context("authlib-injector: reading body")?;

    // An error page or a cut-off body cached as the agent would break every
    // launch after it, with the JVM refusing to start.
    if !bytes.starts_with(b"PK\x03\x04") {
        if cached {
            tracing::warn!("authlib-injector: the master sent something that isn't a jar, keeping the cached copy");
            return Ok(path);
        }
        bail!("authlib-injector: the master sent something that isn't a jar");
    }
    crate::fsutil::write_atomic(&path, &bytes)
        .await
        .context("authlib-injector: writing the jar")?;
    match new_etag {
        Some(tag) => {
            let _ = crate::fsutil::write_atomic(&etag_path, tag).await;
        }
        None => {
            let _ = tokio::fs::remove_file(&etag_path).await;
        }
    }
    Ok(path)
}

fn etag_path(jar: &Path) -> PathBuf {
    let mut name = jar.as_os_str().to_os_string();
    name.push(".etag");
    PathBuf::from(name)
}

/// A jar is a zip; anything else on disk is a broken earlier download.
async fn is_jar(path: &Path) -> bool {
    use tokio::io::AsyncReadExt;
    let Ok(mut file) = tokio::fs::File::open(path).await else {
        return false;
    };
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).await.is_ok() && &magic == b"PK\x03\x04"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_http::{serve, TempDir};

    fn setup(master: &str, root: &Path) -> (LauncherConfig, LauncherDirectories) {
        let config = LauncherConfig {
            master_url: master.to_string(),
            ..LauncherConfig::default()
        };
        let dirs = LauncherDirectories {
            root: root.to_path_buf(),
        };
        (config, dirs)
    }

    #[tokio::test]
    async fn downloads_a_jar_and_refuses_an_error_page() {
        let dir = TempDir::new("authlib");
        let good = serve(vec![(
            "/api/agent/authlib-injector.jar",
            200,
            b"PK\x03\x04rest-of-jar".to_vec(),
        )])
        .await;
        let (config, dirs) = setup(&good.base, dir.path());
        let path = ensure_authlib_injector(&reqwest::Client::new(), &config, &dirs)
            .await
            .unwrap();
        assert!(tokio::fs::read(&path).await.unwrap().starts_with(b"PK"));

        // The master now answers with an HTML error: the good copy stays.
        let bad = serve(vec![(
            "/api/agent/authlib-injector.jar",
            200,
            b"<html>oops</html>".to_vec(),
        )])
        .await;
        let (config, dirs) = setup(&bad.base, dir.path());
        let path = ensure_authlib_injector(&reqwest::Client::new(), &config, &dirs)
            .await
            .unwrap();
        assert!(tokio::fs::read(&path).await.unwrap().starts_with(b"PK"));
    }

    #[tokio::test]
    async fn no_copy_and_no_master_is_an_error() {
        let dir = TempDir::new("authlib");
        let (config, dirs) = setup("http://127.0.0.1:9", dir.path());
        assert!(
            ensure_authlib_injector(&reqwest::Client::new(), &config, &dirs)
                .await
                .is_err()
        );
    }
}
