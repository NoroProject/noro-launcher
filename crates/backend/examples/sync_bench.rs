//! Download a real build's files with the sync's own pool and compare
//! connection counts — the measurement behind `http::SYNC_CONNECTIONS`.
//!
//! ```text
//! cargo run --release -p backend --example sync_bench -- \
//!     files.json https://cdn.example.com/files 1,4,8 [sample]
//! ```
//!
//! `files.json` is a list of `{ "path", "sha1", "size" }` — what
//! `/api/admin/builds/<id>/files` returns. `sample` keeps a fixed-seed subset
//! of about that many files so a run doesn't take the whole modpack each time.
//! Every run starts from an empty directory, the way a first install does.

use backend::sync::downloader::{download_all, DownloadTask, Net};
use backend::sync::trace::SyncTrace;
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
struct File {
    path: String,
    sha1: String,
    size: u64,
}

/// The mods stage's concurrency — the stage a modpack's files go through.
/// `BENCH_CONCURRENCY` overrides it to see whether more in flight helps.
fn concurrency() -> usize {
    std::env::var("BENCH_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(48)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("sync_trace=info,warn")
        .with_target(false)
        .without_time()
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [list, base, conns, rest @ ..] = args.as_slice() else {
        anyhow::bail!("usage: sync_bench <files.json> <files-base-url> <1,4,8> [sample]");
    };
    let mut files: Vec<File> = serde_json::from_slice(&std::fs::read(list)?)?;
    if let Some(n) = rest.first().and_then(|n| n.parse::<usize>().ok()) {
        // Every k-th file, which keeps the mix of tiny configs and big jars.
        let step = (files.len() / n.max(1)).max(1);
        files = files.into_iter().step_by(step).collect();
    }
    let bytes: u64 = files.iter().map(|f| f.size).sum();
    println!("{} files, {:.1} MB", files.len(), bytes as f64 / 1e6);

    for conns in conns
        .split(',')
        .filter_map(|c| c.trim().parse::<usize>().ok())
    {
        let dir = std::env::temp_dir().join(format!("noro-sync-bench-{conns}"));
        let _ = std::fs::remove_dir_all(&dir);
        let tasks: Vec<DownloadTask> = files
            .iter()
            .map(|f| DownloadTask {
                url: format!("{}/{}", base.trim_end_matches('/'), f.sha1),
                dest: dir.join(&f.path),
                sha1: f.sha1.clone(),
                size: f.size,
                executable: false,
            })
            .collect();
        let net = Net {
            pool: backend::http::HttpClientPool::new(conns)?,
            trace: SyncTrace::new(conns),
        };
        println!("\n=== {conns} connection(s) ===");
        let sampler = net.trace.spawn_sampler();
        let t = Instant::now();
        let result = download_all(&net, "mods", tasks, concurrency(), |_| {}, || false).await;
        net.trace.phase("download", t.elapsed());
        sampler.abort();
        net.trace.finish(&dir).await;
        result?;
    }
    Ok(())
}
