// Over 150 lines: the polling loop and one job's lifecycle, with the log pump
// that doubles as the claim's heartbeat.
//! `run`: poll the master, build whatever target it hands over, repeat.

use crate::args::Args;
use crate::compile::{self, Sink};
use crate::master::{Beat, Claim, Master};
use crate::{package, source};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// How often an idle agent asks for work. Also how the master knows it is
/// online — see `ONLINE_SECS` on the master, which is several of these.
const POLL: Duration = Duration::from_secs(10);
/// Log is sent in batches; a heartbeat goes out at least this often even when
/// the build prints nothing, as during a long link.
const FLUSH: Duration = Duration::from_secs(3);
const HEARTBEAT: Duration = Duration::from_secs(30);

pub async fn run(args: Args) -> Result<()> {
    let master = Arc::new(Master::from_env()?);
    let name = args.name.clone().unwrap_or_else(default_name);
    let targets = if args.targets.is_empty() {
        compile::host_targets()
    } else {
        args.targets.clone()
    };
    anyhow::ensure!(
        !targets.is_empty(),
        "this host has no default targets; pass --targets"
    );
    let work = match args.work.clone() {
        Some(w) => w,
        None => dirs::home_dir()
            .context("no home directory; pass --work")?
            .join(".noro-build-agent"),
    };
    std::fs::create_dir_all(&work)?;
    let work = work.canonicalize()?;
    let host = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

    println!(
        "agent {name}: targets {}, work dir {}",
        targets.join(", "),
        work.display()
    );
    loop {
        match master.claim(&name, &targets, &host).await {
            Ok(Some(claim)) => {
                println!("job {} → {} ({})", claim.job_id, claim.target, claim.tag);
                job(&master, &name, &work, claim).await;
            }
            Ok(None) => tokio::time::sleep(POLL).await,
            Err(e) => {
                eprintln!("claim failed: {e:#}");
                tokio::time::sleep(POLL).await;
            }
        }
    }
}

/// Lower-case host name, so the agent shows up as something recognisable in
/// the admin panel without configuration.
fn default_name() -> String {
    let raw = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|| "agent".into());
    let name: String = raw
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .take(64)
        .collect();
    name.trim_end_matches(".local").to_string()
}

async fn job(master: &Arc<Master>, agent: &str, work: &Path, claim: Claim) {
    let claim = Arc::new(claim);
    let (tx, rx) = mpsc::unbounded_channel();
    let lost = Arc::new(AtomicBool::new(false));
    let sink = Sink {
        lines: tx,
        lost: lost.clone(),
    };
    let pump = tokio::spawn(pump(
        master.clone(),
        claim.clone(),
        agent.to_string(),
        rx,
        lost.clone(),
    ));

    let result = build(master, work, &claim, &sink).await;
    let result = match result {
        Ok(files) => {
            sink.say(format!("uploading {} artifacts", files.len()));
            // Everything printed so far has to reach the log before the upload
            // closes the target, or the tail lands after "done".
            drop(sink);
            let _ = pump.await;
            master.upload(&claim, agent, &files).await.map(|v| {
                println!("uploaded: {v}");
            })
        }
        Err(e) => {
            drop(sink);
            let _ = pump.await;
            Err(e)
        }
    };

    if let Err(e) = result {
        eprintln!("{} failed: {e:#}", claim.target);
        if !lost.load(Ordering::Relaxed) {
            if let Err(report) = master.fail(&claim, agent, &format!("{e:#}")).await {
                eprintln!("could not report the failure: {report:#}");
            }
        }
    }
}

async fn build(master: &Master, work: &Path, claim: &Claim, sink: &Sink) -> Result<Vec<PathBuf>> {
    let archive = work.join("source.zip");
    let src = work.join("src");
    sink.say(format!("downloading source {}", claim.source_url));
    master.download(&claim.source_url, &archive).await?;
    let files = {
        let (archive, src) = (archive.clone(), src.clone());
        tokio::task::spawn_blocking(move || source::unpack(&archive, &src)).await??
    };
    sink.say(format!("unpacked {files} files"));

    let out = compile::build(&src, &claim.target, &claim.tag, sink).await?;
    compile::upload_symbols(&src, &out, &claim.target, sink).await?;
    let dist = work.join("dist");
    package::package(&src, &out, &dist, &claim.target, &claim.tag, sink).await
}

/// Ships log lines in batches and keeps the claim alive. Ends when the sink is
/// dropped, after one last flush.
async fn pump(
    master: Arc<Master>,
    claim: Arc<Claim>,
    agent: String,
    mut rx: mpsc::UnboundedReceiver<String>,
    lost: Arc<AtomicBool>,
) {
    let mut buf = String::new();
    let mut last_sent = tokio::time::Instant::now();
    let mut tick = tokio::time::interval(FLUSH);
    loop {
        let closed = tokio::select! {
            line = rx.recv() => match line {
                Some(line) => {
                    buf.push_str(&line);
                    buf.push('\n');
                    false
                }
                None => true,
            },
            _ = tick.tick() => false,
        };
        let due = !buf.is_empty() || last_sent.elapsed() >= HEARTBEAT;
        if (due && last_sent.elapsed() >= FLUSH) || closed {
            match master.log(&claim, &agent, std::mem::take(&mut buf)).await {
                Ok(Beat::Kept) => {}
                Ok(Beat::Lost) => {
                    eprintln!("the master took {} away; stopping", claim.target);
                    lost.store(true, Ordering::Relaxed);
                    return;
                }
                // A dropped request loses a few lines, not the build. The
                // claim survives several missed heartbeats.
                Err(e) => eprintln!("log upload failed: {e:#}"),
            }
            last_sent = tokio::time::Instant::now();
        }
        if closed {
            return;
        }
    }
}
