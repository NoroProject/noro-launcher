// Over 150 lines: the counters a sync fills in, the once-a-second sampler and
// the report, which only make sense read together.
//! Where a sync's time goes: waiting for the server, waiting for bytes,
//! waiting for the disk.
//!
//! Always collected — a few `Instant::now()` per chunk, nothing next to the
//! download itself — and summarised into the log and
//! `.noro/sync-report.json` after every sync. `NORO_SYNC_DEBUG=1` adds a live
//! line under the progress bar and keeps every file in the report instead of
//! the slowest ones.

use parking_lot::Mutex;
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// One file as the fetcher saw it.
#[derive(Default, Clone, Debug)]
pub struct FetchStats {
    /// Request sent until response headers: server think time plus one
    /// round trip. On a CDN miss it includes the origin fetch.
    pub ttfb: Duration,
    /// Awaiting body chunks — the network.
    pub net: Duration,
    /// Awaiting writes, flush and rename — the disk, and on Windows the
    /// antivirus scanning each new file.
    pub disk: Duration,
    pub hash: Duration,
    pub http: &'static str,
    /// `cf-cache-status` from Cloudflare: HIT, MISS, DYNAMIC…
    pub cache: Option<String>,
    /// Ranged parts fetched in parallel; 1 for an ordinary download.
    pub segments: u32,
}

impl FetchStats {
    pub fn from_response(resp: &reqwest::Response, ttfb: Duration) -> Self {
        let http = match resp.version() {
            reqwest::Version::HTTP_2 => "h2",
            reqwest::Version::HTTP_3 => "h3",
            reqwest::Version::HTTP_10 => "h1.0",
            _ => "h1.1",
        };
        let cache = resp
            .headers()
            .get("cf-cache-status")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        Self {
            ttfb,
            http,
            cache,
            segments: 1,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Clone)]
pub struct FileRecord {
    pub url: String,
    pub stage: String,
    pub size: u64,
    /// Which connection carried it; equal to the connection count when the
    /// file was split across all of them.
    pub conn: usize,
    /// Waited for a free slot in its stage's pool.
    pub queued_ms: u64,
    pub ttfb_ms: u64,
    pub net_ms: u64,
    pub disk_ms: u64,
    pub hash_ms: u64,
    pub total_ms: u64,
    pub attempts: u32,
    pub http: &'static str,
    pub cache: Option<String>,
    pub segments: u32,
}

#[derive(Serialize, Clone, Copy)]
struct Sample {
    at_s: u64,
    mb_per_s: f64,
    in_flight: usize,
}

#[derive(Default)]
struct State {
    files: Vec<FileRecord>,
    phases: Vec<(&'static str, u64)>,
    samples: Vec<Sample>,
}

pub struct SyncTrace {
    debug: bool,
    started: Instant,
    connections: usize,
    bytes: AtomicU64,
    in_flight: AtomicUsize,
    retries: AtomicU64,
    files_total: AtomicU64,
    slots: AtomicUsize,
    state: Mutex<State>,
    ui: Mutex<String>,
}

pub fn debug_enabled() -> bool {
    std::env::var("NORO_SYNC_DEBUG").is_ok_and(|v| !v.is_empty() && v != "0")
}

fn ms(d: Duration) -> u64 {
    d.as_millis() as u64
}

impl SyncTrace {
    pub fn new(connections: usize) -> Arc<Self> {
        Arc::new(Self {
            debug: debug_enabled(),
            started: Instant::now(),
            connections,
            bytes: AtomicU64::new(0),
            in_flight: AtomicUsize::new(0),
            retries: AtomicU64::new(0),
            files_total: AtomicU64::new(0),
            slots: AtomicUsize::new(0),
            state: Mutex::new(State::default()),
            ui: Mutex::new(String::new()),
        })
    }

    pub fn phase(&self, name: &'static str, took: Duration) {
        self.state.lock().phases.push((name, ms(took)));
    }

    /// A stage starting with `n` files and `slots` downloads at once.
    pub fn expect_files(&self, n: usize, slots: usize) {
        self.files_total.fetch_add(n as u64, Ordering::Relaxed);
        self.slots.fetch_add(slots.min(n), Ordering::Relaxed);
    }

    pub fn bytes(&self, delta: i64) {
        if delta >= 0 {
            self.bytes.fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            self.bytes
                .fetch_sub(delta.unsigned_abs(), Ordering::Relaxed);
        }
    }

    pub fn started_file(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
    }

    pub fn retry(&self) {
        self.retries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn finished_file(&self, record: FileRecord) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
        self.state.lock().files.push(record);
    }

    /// The live line for the progress screen; empty unless debugging.
    pub fn ui_line(&self) -> String {
        if self.debug {
            self.ui.lock().clone()
        } else {
            String::new()
        }
    }

    /// Samples throughput once a second until the returned handle is aborted.
    pub fn spawn_sampler(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let trace = self.clone();
        tokio::spawn(async move {
            let mut last = 0u64;
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            tick.tick().await;
            loop {
                tick.tick().await;
                let now = trace.bytes.load(Ordering::Relaxed);
                let sample = Sample {
                    at_s: trace.started.elapsed().as_secs(),
                    mb_per_s: now.saturating_sub(last) as f64 / 1e6,
                    in_flight: trace.in_flight.load(Ordering::Relaxed),
                };
                last = now;
                let line = trace.live_line(sample);
                if trace.debug {
                    tracing::info!(target: "sync_trace", "{line}");
                }
                *trace.ui.lock() = line;
                trace.state.lock().samples.push(sample);
            }
        })
    }

    fn live_line(&self, s: Sample) -> String {
        let st = self.state.lock();
        let done = st.files.len();
        let recent = &st.files[done.saturating_sub(200)..];
        let mut ttfb: Vec<u64> = recent.iter().map(|f| f.ttfb_ms).collect();
        ttfb.sort_unstable();
        let p50 = ttfb.get(ttfb.len() / 2).copied().unwrap_or(0);
        let (net, disk) = recent
            .iter()
            .fold((0u64, 0u64), |(n, d), f| (n + f.net_ms, d + f.disk_ms));
        let disk_share = super::trace_report::pct(disk, net + disk);
        let hits = recent
            .iter()
            .filter(|f| f.cache.as_deref() == Some("HIT"))
            .count();
        let hit = super::trace_report::pct(hits as u64, recent.len() as u64);
        format!(
            "{:.1} MB/s · {}/{} in flight on {} conn · {done}/{} files · ttfb p50 {p50} ms · disk {disk_share}% · CDN hit {hit}% · retries {}",
            s.mb_per_s,
            s.in_flight,
            self.slots.load(Ordering::Relaxed),
            self.connections,
            self.files_total.load(Ordering::Relaxed),
            self.retries.load(Ordering::Relaxed),
        )
    }

    /// Log the verdict and write `.noro/sync-report.json`.
    pub async fn finish(&self, instance_dir: &Path) {
        let report = super::trace_report::build(
            self.state
                .lock()
                .snapshot(self.slots.load(Ordering::Relaxed)),
            self.started.elapsed(),
            self.connections,
            self.retries.load(Ordering::Relaxed),
            self.debug,
        );
        for line in &report.findings {
            tracing::info!(target: "sync_trace", "{line}");
        }
        tracing::info!(target: "sync_trace", "{}", report.summary);
        let path = instance_dir.join(".noro/sync-report.json");
        match serde_json::to_vec_pretty(&report) {
            Ok(json) => {
                if let Err(e) = crate::fsutil::write_atomic(&path, json).await {
                    tracing::warn!(error = %format!("{e:#}"), "sync report not written");
                } else {
                    tracing::info!(target: "sync_trace", "full report: {}", path.display());
                }
            }
            Err(e) => tracing::warn!(error = %e, "sync report not serialised"),
        }
    }
}

/// Read access for the report builder, which lives in its own file.
pub(super) struct Snapshot<'a> {
    pub files: &'a [FileRecord],
    pub phases: &'a [(&'static str, u64)],
    pub samples: Vec<(u64, f64, usize)>,
    pub slots: usize,
}

impl State {
    pub(super) fn snapshot(&self, slots: usize) -> Snapshot<'_> {
        Snapshot {
            slots,
            files: &self.files,
            phases: &self.phases,
            samples: self
                .samples
                .iter()
                .map(|s| (s.at_s, s.mb_per_s, s.in_flight))
                .collect(),
        }
    }
}
