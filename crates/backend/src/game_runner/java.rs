/// Archives and some mirrors drop the executable bit; the JVM then fails to
/// start with a bare "permission denied". Windows has no such bit.
#[cfg(unix)]
pub async fn ensure_executable(path: &std::path::Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = tokio::fs::metadata(path).await?.permissions();
    if perms.mode() & 0o111 == 0 {
        perms.set_mode(0o755);
        tokio::fs::set_permissions(path, perms).await?;
    }
    Ok(())
}

/// The rest of the runtime needs the bit too, not just `bin/java`. Without it
/// on `lib/jspawnhelper` every process the game starts fails with
/// "posix_spawn failed": a link opened in the browser, a crash reporter, a
/// mod's own helper. `java` is `<home>/bin/java`.
#[cfg(unix)]
pub async fn ensure_runtime_executable(java: &std::path::Path) -> anyhow::Result<()> {
    ensure_executable(java).await?;
    let Some(home) = java.parent().and_then(std::path::Path::parent) else {
        return Ok(());
    };
    let mut helpers = vec![home.join("lib/jspawnhelper"), home.join("lib/jexec")];
    if let Ok(mut bin) = tokio::fs::read_dir(home.join("bin")).await {
        while let Ok(Some(entry)) = bin.next_entry().await {
            helpers.push(entry.path());
        }
    }
    for helper in helpers {
        if tokio::fs::metadata(&helper)
            .await
            .is_ok_and(|m| m.is_file())
        {
            ensure_executable(&helper).await?;
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    async fn mode(path: &std::path::Path) -> u32 {
        tokio::fs::metadata(path)
            .await
            .unwrap()
            .permissions()
            .mode()
            & 0o777
    }

    #[tokio::test]
    async fn marks_the_spawn_helper_and_bin_tools() {
        let home = std::env::temp_dir().join(format!("noro-java-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(home.join("bin")).await.unwrap();
        tokio::fs::create_dir_all(home.join("lib")).await.unwrap();
        for f in [
            "bin/java",
            "bin/keytool",
            "lib/jspawnhelper",
            "lib/libjava.dylib",
        ] {
            tokio::fs::write(home.join(f), b"x").await.unwrap();
            let perms = std::fs::Permissions::from_mode(0o644);
            tokio::fs::set_permissions(home.join(f), perms)
                .await
                .unwrap();
        }

        ensure_runtime_executable(&home.join("bin/java"))
            .await
            .unwrap();

        assert_eq!(mode(&home.join("bin/java")).await, 0o755);
        assert_eq!(mode(&home.join("bin/keytool")).await, 0o755);
        assert_eq!(mode(&home.join("lib/jspawnhelper")).await, 0o755);
        // Libraries are loaded, not run: left as they are.
        assert_eq!(mode(&home.join("lib/libjava.dylib")).await, 0o644);
        tokio::fs::remove_dir_all(&home).await.ok();
    }
}
