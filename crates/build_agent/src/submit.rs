//! `submit`: pack this working copy and queue a build of it.

use crate::args::Args;
use crate::master::Master;
use crate::source;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(args: Args) -> Result<()> {
    let master = Master::from_env()?;
    let repo = match args.repo.clone() {
        Some(r) => r,
        None => repo_root()?,
    };
    // The tag comes from Cargo.toml rather than a flag: the launcher compares
    // its own CARGO_PKG_VERSION against the tag, and a mismatch leaves the
    // "update available" banner up forever.
    let tag = format!("v{}", source::workspace_version(&repo)?);

    let packed = {
        let repo = repo.clone();
        tokio::task::spawn_blocking(move || source::pack(&repo)).await??
    };
    let (archive, count) = packed;
    println!(
        "{tag}: {count} files, {:.1} MB from {}",
        archive.len() as f64 / 1e6,
        repo.display()
    );

    let job_id = master
        .submit(&tag, &args.targets, args.deploy.as_deref(), archive)
        .await?;
    println!("job {job_id} queued");
    if args.no_wait {
        return Ok(());
    }
    follow(&master, &job_id).await
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse; pass --repo")?;
    anyhow::ensure!(
        out.status.success(),
        "not inside a git checkout; pass --repo"
    );
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// Print the job's log as it grows until the job closes.
async fn follow(master: &Master, job_id: &str) -> Result<()> {
    let mut shown = 0;
    loop {
        let v = master.job_log(job_id).await?;
        let log = v["log"].as_str().unwrap_or_default();
        if log.len() > shown {
            print!("{}", &log[shown..]);
            shown = log.len();
        }
        match v["status"].as_str() {
            Some("done") => return Ok(()),
            Some("failed") => anyhow::bail!("build {job_id} failed"),
            _ => tokio::time::sleep(Duration::from_secs(3)).await,
        }
    }
}
