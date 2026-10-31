// Over 150 lines: running a tool with its output forwarded and its lifetime
// tied to the claim, plus the two cargo invocations that use it.
//! Building one target with the same commands the release workflow runs.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

/// Where build output goes, and whether anyone still wants it.
#[derive(Clone)]
pub struct Sink {
    pub lines: UnboundedSender<String>,
    /// Set when the master took the target away. Whatever is running then is
    /// killed: its result would be refused anyway.
    pub lost: Arc<AtomicBool>,
}

impl Sink {
    pub fn say(&self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        let _ = self.lines.send(line);
    }

    fn lost(&self) -> bool {
        self.lost.load(Ordering::Relaxed)
    }
}

/// Environment the build must not see. The token is a secret, and a
/// compile-time `NORO_MASTER_URL` would become the binary's fallback address —
/// CI never sets it, and the master stamps the real one in anyway.
const SCRUBBED: &[&str] = &["NORO_ADMIN_TOKEN", "NORO_MASTER_URL"];

pub fn command(program: &str, dir: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir);
    for var in SCRUBBED {
        cmd.env_remove(var);
    }
    cmd
}

/// Run to completion with stdout and stderr forwarded line by line.
pub async fn run(mut cmd: Command, sink: &Sink) -> Result<()> {
    let shown = format!("{:?}", cmd.as_std());
    sink.say(format!("$ {shown}"));
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = cmd.spawn().with_context(|| format!("starting {shown}"))?;
    let out = forward(child.stdout.take(), sink.clone());
    let err = forward(child.stderr.take(), sink.clone());

    let status = loop {
        tokio::select! {
            status = child.wait() => break status?,
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                if sink.lost() {
                    let _ = child.kill().await;
                    bail!("the master took this target away");
                }
            }
        }
    };
    let _ = tokio::join!(out, err);
    if !status.success() {
        bail!("{shown} exited with {status}");
    }
    Ok(())
}

fn forward<R>(stream: Option<R>, sink: Sink) -> tokio::task::JoinHandle<()>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let Some(stream) = stream else { return };
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            sink.say(line);
        }
    })
}

/// Natively buildable targets of this host. Linux targets elsewhere go through
/// `cross` and need Docker, so they are opt-in via `--targets`.
pub fn host_targets() -> Vec<String> {
    let list: &[&str] = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", _) => &["aarch64-apple-darwin", "x86_64-apple-darwin"],
        ("windows", "x86_64") => &["x86_64-pc-windows-msvc"],
        ("linux", "x86_64") => &["x86_64-unknown-linux-gnu"],
        ("linux", "aarch64") => &["aarch64-unknown-linux-gnu"],
        _ => &[],
    };
    list.iter().map(|t| t.to_string()).collect()
}

fn needs_cross(target: &str) -> bool {
    target.contains("-linux-") && !host_targets().iter().any(|t| t == target)
}

/// Both binaries for `target`; returns the directory they land in.
pub async fn build(src: &Path, target: &str, tag: &str, sink: &Sink) -> Result<PathBuf> {
    // rust-toolchain.toml pins the toolchain; the target has to be added to
    // that one, not to whatever `rustup default` is.
    let mut add = command("rustup", src);
    add.args(["target", "add", target]);
    if let Err(e) = run(add, sink).await {
        sink.say(format!(
            "warning: rustup target add failed ({e:#}), trying the build anyway"
        ));
    }

    let tool = if needs_cross(target) {
        "cross"
    } else {
        "cargo"
    };
    for (package, bin) in [
        ("noro_launcher", "noro-launcher"),
        ("noro_core", "noro-launcher-core"),
    ] {
        let mut cmd = command(tool, src);
        // release-symbols, not release: same optimised code plus the string
        // tables Sentry needs. See [profile.release-symbols] in Cargo.toml.
        cmd.args([
            "build",
            "--locked",
            "--profile",
            "release-symbols",
            "-p",
            package,
            "--bin",
            bin,
            "--target",
            target,
        ])
        // From the tag, so crashes are filed against this release.
        .env("NORO_RELEASE", tag);
        run(cmd, sink).await?;
    }
    Ok(src.join("target").join(target).join("release-symbols"))
}

/// Debug files to Sentry, when this machine is set up for it. Optional by
/// design: a build without telemetry is still a complete build.
pub async fn upload_symbols(src: &Path, out: &Path, target: &str, sink: &Sink) -> Result<()> {
    let (Ok(org), Ok(_)) = (
        std::env::var("SENTRY_ORG"),
        std::env::var("SENTRY_AUTH_TOKEN"),
    ) else {
        sink.say("SENTRY_ORG/SENTRY_AUTH_TOKEN not set — symbols are not uploaded");
        return Ok(());
    };
    // On macOS debug info stays in the .o files and the binary alone is
    // useless to Sentry; dsymutil gathers it into a .dSYM.
    if target.contains("apple") {
        let mut dsym = command("dsymutil", src);
        dsym.arg(out.join("noro-launcher-core"))
            .arg(out.join("noro-launcher"));
        run(dsym, sink).await?;
    }
    let mut up = command("sentry-cli", src);
    up.args([
        "debug-files",
        "upload",
        "--org",
        &org,
        "--project",
        "noro-launcher",
    ])
    .args(["--include-sources", "--wait"])
    .arg(out);
    if std::env::var_os("SENTRY_URL").is_none() {
        up.env("SENTRY_URL", "https://de.sentry.io");
    }
    run(up, sink).await
}
