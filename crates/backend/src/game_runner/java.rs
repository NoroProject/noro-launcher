use anyhow::Result;
use std::path::Path;

#[cfg(unix)]
pub async fn ensure_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = tokio::fs::metadata(path).await?.permissions();
    if perms.mode() & 0o111 == 0 {
        perms.set_mode(0o755);
        tokio::fs::set_permissions(path, perms).await?;
    }
    Ok(())
}
