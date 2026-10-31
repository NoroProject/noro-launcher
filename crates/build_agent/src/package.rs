//! Artifacts named and shaped exactly as `release-launcher.yml` makes them:
//! the master tells core from installer and platform from platform by name.

use crate::compile::{command, run, Sink};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// The files to upload for `target`.
pub async fn package(
    src: &Path,
    out: &Path,
    dist: &Path,
    target: &str,
    tag: &str,
    sink: &Sink,
) -> Result<Vec<PathBuf>> {
    if dist.exists() {
        std::fs::remove_dir_all(dist)?;
    }
    std::fs::create_dir_all(dist)?;
    let exe = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };
    let launcher = dist.join(format!("noro-launcher-{target}{exe}"));
    let core = dist.join(format!("noro-launcher-core-{target}{exe}"));
    std::fs::copy(out.join(format!("noro-launcher{exe}")), &launcher)?;
    std::fs::copy(out.join(format!("noro-launcher-core{exe}")), &core)?;

    if target.contains("apple") {
        mac(src, dist, &launcher, &core, target, tag, sink).await
    } else if target.contains("linux") {
        strip(src, &[&launcher, &core], target, sink).await?;
        Ok(vec![launcher, core])
    } else {
        Ok(vec![launcher, core])
    }
}

/// The installer for macOS is the .app; the master wraps it in a DMG, signs
/// and notarises it. Core goes up as a bare binary.
async fn mac(
    src: &Path,
    dist: &Path,
    launcher: &Path,
    core: &Path,
    target: &str,
    tag: &str,
    sink: &Sink,
) -> Result<Vec<PathBuf>> {
    let identity = std::env::var("MACOS_SIGN_IDENTITY").unwrap_or_else(|_| "-".into());
    for bin in [launcher, core] {
        let mut sign = command("codesign", src);
        sign.args(["--force", "--sign", &identity]);
        // The runtime either way: the master keeps these flags when it
        // re-signs, and notarisation refuses a binary without it.
        sign.args(["--options", "runtime"]);
        if identity == "-" {
            sign.arg("--timestamp=none");
        } else {
            sign.arg("--timestamp");
        }
        sign.arg(bin);
        run(sign, sink).await?;
    }

    // Both tag shapes, v1.2.3 and launcher-v1.2.3, give 1.2.3.
    let version = tag.trim_start_matches("launcher-").trim_start_matches('v');
    let app = dist.join("NoroLauncher.app");
    let mut make = command("bash", src);
    make.arg(".github/scripts/make-app.sh")
        .arg(launcher)
        .arg(&app)
        .arg(version);
    run(make, sink).await?;

    let bundle = dist.join(format!("noro-launcher-{target}.app.tar.gz"));
    // Without these bsdtar stores extended attributes (com.apple.provenance)
    // as AppleDouble `._*` entries. The master unpacks on Linux, gets them as
    // real files inside the bundle, seals them into the signature — and
    // notarisation then rejects the main executable's signature as invalid.
    let mut tar = command("tar", dist);
    tar.env("COPYFILE_DISABLE", "1")
        .args(["--no-xattrs", "--no-mac-metadata", "-czf"])
        .arg(&bundle)
        .arg("NoroLauncher.app");
    run(tar, sink).await?;
    Ok(vec![bundle, core.to_path_buf()])
}

/// On Linux debug info lives inside the ELF. Sentry already has it, so players
/// don't need to carry it — unstripped, the aarch64 build is about three times
/// the size. Fails rather than warns: a silent skip ships a much larger binary.
async fn strip(src: &Path, files: &[&Path], target: &str, sink: &Sink) -> Result<()> {
    let Some(tool) = strip_tool(target) else {
        bail!("no strip that handles {target}: install llvm-tools (rustup component add llvm-tools) or binutils for it");
    };
    let mut cmd = command(&tool.to_string_lossy(), src);
    cmd.arg("--strip-debug").args(files);
    run(cmd, sink).await
}

/// `llvm-strip` from the rustup sysroot handles every ELF; a native Linux host
/// can fall back to its own binutils.
fn strip_tool(target: &str) -> Option<PathBuf> {
    let sysroot = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    let host = std::process::Command::new("rustc")
        .args(["-vV"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find_map(|l| l.strip_prefix("host: ").map(String::from))
        });
    if let (Some(root), Some(host)) = (sysroot, host) {
        let llvm = root.join("lib/rustlib").join(host).join("bin/llvm-strip");
        if llvm.exists() {
            return Some(llvm);
        }
    }
    let native = crate::compile::host_targets().iter().any(|t| t == target);
    native.then(|| PathBuf::from("strip"))
}
