// Over 150 lines: the report's shape and the rules that turn its numbers into
// a verdict sit together so the thresholds can be read next to what they test.
//! Turning a sync's trace into a verdict a person can act on.
//!
//! Times per file overlap — forty downloads run at once — so they are compared
//! as shares of the summed time, not against the wall clock. "Disk 45%" means
//! that of all the time files spent being fetched, 45% went into waiting for
//! writes.

use super::trace::{FileRecord, Snapshot};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Serialize)]
pub struct Report {
    pub summary: String,
    pub findings: Vec<String>,
    pub wall_ms: u64,
    pub bytes: u64,
    pub files: usize,
    pub connections: usize,
    /// Download slots across all stages running at once.
    pub slots: usize,
    pub avg_in_flight: f64,
    pub peak_mb_s: f64,
    pub retries: u64,
    pub phases: Vec<(&'static str, u64)>,
    pub stages: BTreeMap<String, StageTotals>,
    pub per_connection: Vec<ConnTotals>,
    pub cache: BTreeMap<String, usize>,
    pub ttfb_by_cache: BTreeMap<String, Percentiles>,
    pub http: BTreeMap<String, usize>,
    /// (second, MB/s, in flight).
    pub timeline: Vec<(u64, f64, usize)>,
    /// Slowest first. Every file with `NORO_SYNC_DEBUG`, otherwise the top 50.
    pub slowest: Vec<FileRecord>,
}

#[derive(Serialize, Default)]
pub struct StageTotals {
    pub files: usize,
    pub bytes: u64,
    pub queued_ms: u64,
    pub ttfb_ms: u64,
    pub net_ms: u64,
    pub disk_ms: u64,
    pub hash_ms: u64,
}

#[derive(Serialize, Default, Clone)]
pub struct ConnTotals {
    pub files: usize,
    pub bytes: u64,
    pub net_ms: u64,
}

pub(super) fn pct(part: u64, whole: u64) -> u64 {
    (part * 100).checked_div(whole).unwrap_or(0)
}

/// Time to first byte, split by `cf-cache-status`: a MISS goes to the origin
/// and back, so its numbers are the origin's.
#[derive(Serialize)]
pub struct Percentiles {
    pub n: usize,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p99_ms: u64,
}

fn percentiles(mut v: Vec<u64>) -> Percentiles {
    v.sort_unstable();
    let at = |q: f64| {
        v.get(((v.len() as f64 - 1.0) * q) as usize)
            .copied()
            .unwrap_or(0)
    };
    Percentiles {
        n: v.len(),
        p50_ms: at(0.5),
        p90_ms: at(0.9),
        p99_ms: at(0.99),
    }
}

pub(super) fn build(
    snap: Snapshot<'_>,
    wall: Duration,
    connections: usize,
    retries: u64,
    keep_all: bool,
) -> Report {
    let files = snap.files;
    let mut stages: BTreeMap<String, StageTotals> = BTreeMap::new();
    let mut per_connection = vec![ConnTotals::default(); connections.max(1)];
    let mut cache = BTreeMap::new();
    let mut http = BTreeMap::new();
    let mut ttfb: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for f in files {
        let s = stages.entry(f.stage.clone()).or_default();
        s.files += 1;
        s.bytes += f.size;
        s.queued_ms += f.queued_ms;
        s.ttfb_ms += f.ttfb_ms;
        s.net_ms += f.net_ms;
        s.disk_ms += f.disk_ms;
        s.hash_ms += f.hash_ms;
        if let Some(c) = per_connection.get_mut(f.conn) {
            c.files += 1;
            c.bytes += f.size;
            c.net_ms += f.net_ms;
        }
        let status = f.cache.clone().unwrap_or_else(|| "none".into());
        ttfb.entry(status.clone()).or_default().push(f.ttfb_ms);
        *cache.entry(status).or_insert(0) += 1;
        *http.entry(f.http.to_string()).or_insert(0) += 1;
    }
    let ttfb_by_cache: BTreeMap<String, Percentiles> =
        ttfb.into_iter().map(|(k, v)| (k, percentiles(v))).collect();

    let bytes: u64 = files.iter().map(|f| f.size).sum();
    let wall_ms = wall.as_millis() as u64;
    let download_ms = snap
        .phases
        .iter()
        .find(|(name, _)| *name == "download")
        .map(|(_, ms)| *ms)
        .unwrap_or(wall_ms);
    let mb_s = if download_ms > 0 {
        bytes as f64 / 1e6 / (download_ms as f64 / 1000.0)
    } else {
        0.0
    };
    let peak_mb_s = snap.samples.iter().map(|s| s.1).fold(0.0, f64::max);
    let busy_samples: Vec<usize> = snap
        .samples
        .iter()
        .map(|s| s.2)
        .filter(|n| *n > 0)
        .collect();
    let avg_in_flight = if busy_samples.is_empty() {
        0.0
    } else {
        busy_samples.iter().sum::<usize>() as f64 / busy_samples.len() as f64
    };
    let summary = format!(
        "sync: {} files, {:.1} MB in {:.1} s of download ({mb_s:.1} MB/s, peak {peak_mb_s:.1}) over {connections} connections, {avg_in_flight:.0}/{} slots busy on average, {retries} retries; total {:.1} s",
        files.len(),
        bytes as f64 / 1e6,
        download_ms as f64 / 1000.0,
        snap.slots,
        wall_ms as f64 / 1000.0,
    );

    let findings = verdict(&Ctx {
        files,
        phases: snap.phases,
        cache: &cache,
        ttfb: &ttfb_by_cache,
        wall_ms,
        mb_s,
        peak_mb_s,
        connections,
        retries,
        avg_in_flight,
        slots: snap.slots,
    });

    let mut slowest = files.to_vec();
    slowest.sort_by_key(|f| std::cmp::Reverse(f.total_ms));
    if !keep_all {
        slowest.truncate(50);
    }

    Report {
        summary,
        findings,
        wall_ms,
        bytes,
        files: files.len(),
        connections,
        slots: snap.slots,
        avg_in_flight,
        peak_mb_s,
        retries,
        phases: snap.phases.to_vec(),
        stages,
        per_connection,
        cache,
        ttfb_by_cache,
        http,
        timeline: snap.samples,
        slowest,
    }
}

struct Ctx<'a> {
    files: &'a [FileRecord],
    phases: &'a [(&'static str, u64)],
    cache: &'a BTreeMap<String, usize>,
    ttfb: &'a BTreeMap<String, Percentiles>,
    wall_ms: u64,
    mb_s: f64,
    peak_mb_s: f64,
    connections: usize,
    retries: u64,
    avg_in_flight: f64,
    slots: usize,
}

fn verdict(c: &Ctx<'_>) -> Vec<String> {
    let mut out = Vec::new();
    for (name, ms) in c.phases {
        if *name != "download" && c.wall_ms > 0 && pct(*ms, c.wall_ms) >= 20 {
            out.push(format!(
                "{name} took {}% of the sync ({:.1} s) — that is local work, not the network",
                pct(*ms, c.wall_ms),
                *ms as f64 / 1000.0
            ));
        }
    }
    let files = c.files;
    if files.is_empty() {
        out.push("nothing was downloaded".into());
        return out;
    }

    let sum = |f: fn(&FileRecord) -> u64| files.iter().map(f).sum::<u64>();
    let (ttfb, net, disk, hash) = (
        sum(|f| f.ttfb_ms),
        sum(|f| f.net_ms),
        sum(|f| f.disk_ms),
        sum(|f| f.hash_ms),
    );
    let busy = ttfb + net + disk + hash;
    out.push(format!(
        "time per file: server wait {}%, transfer {}%, disk {}%, hashing {}%",
        pct(ttfb, busy),
        pct(net, busy),
        pct(disk, busy),
        pct(hash, busy)
    ));
    for (status, p) in c.ttfb {
        out.push(format!(
            "first byte, {status}: p50 {} ms, p90 {} ms, p99 {} ms over {} files",
            p.p50_ms, p.p90_ms, p.p99_ms, p.n
        ));
    }

    if pct(disk, busy) >= 30 {
        out.push("DISK-BOUND: writes are a large share. On Windows that is usually the antivirus scanning each new file — exclude the launcher folder; otherwise a slow or full disk".into());
    }
    let small = files.iter().filter(|f| f.size < 256 * 1024).count();
    let latency_bound = pct(ttfb, busy) >= 40 && small * 2 > files.len();
    if latency_bound {
        out.push("LATENCY-BOUND: most files are small and most of their time is waiting for the first byte. More requests in flight helps; bandwidth does not".into());
    }
    // Slots busy most of the time and still waiting on round trips: the pool
    // is the cap. Mostly idle: the tail — a few large files left on their own.
    if c.slots > 0 {
        let busy_share = c.avg_in_flight / c.slots as f64;
        if latency_bound && busy_share >= 0.85 {
            out.push(format!(
                "SLOT-BOUND: {:.0} of {} slots busy on average while files wait on round trips — more in flight would go faster",
                c.avg_in_flight, c.slots
            ));
        } else if busy_share < 0.5 {
            out.push(format!(
                "UNDERUSED: only {:.0} of {} slots busy on average — the sync spent its time on a few large files rather than on many at once",
                c.avg_in_flight, c.slots
            ));
        }
    }
    let misses =
        c.cache.get("MISS").copied().unwrap_or(0) + c.cache.get("EXPIRED").copied().unwrap_or(0);
    if pct(misses as u64, files.len() as u64) >= 30 {
        out.push(format!(
            "CDN COLD: {}% of files were a Cloudflare MISS and came from the origin, which is far slower per connection. A second sync of the same build should be faster",
            pct(misses as u64, files.len() as u64)
        ));
    }
    if pct(net, busy) >= 60 {
        out.push(format!(
            "NETWORK-BOUND: transfer dominates at {:.1} MB/s (peak {:.1}) on {} connections. Try NORO_SYNC_CONNECTIONS=8 — if the rate doesn't grow, that is your link",
            c.mb_s, c.peak_mb_s, c.connections
        ));
    }
    if c.retries > 0 {
        out.push(format!(
            "{} retries — the connection or the server dropped requests",
            c.retries
        ));
    }
    out
}
