//! What the sync screens say, in the player's language.
//!
//! The state keeps what happened (a stage, a count); the words are picked at
//! draw time, so switching the language mid-download relabels everything.

use bridge::SyncStage;
use i18n::{t, FluentArgs};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub fn stage_label(stage: SyncStage) -> String {
    t(match stage {
        SyncStage::CheckingFiles => "sync-checking",
        SyncStage::DownloadingJava => "sync-java",
        SyncStage::DownloadingMinecraft => "sync-minecraft",
        SyncStage::DownloadingLibraries => "sync-libraries",
        SyncStage::DownloadingAssets => "sync-assets",
        SyncStage::DownloadingMods => "sync-mods",
        SyncStage::ApplyingForgePatches => "sync-forge",
        SyncStage::Cleaning => "sync-cleaning",
        SyncStage::Done => "sync-done",
    })
}

/// There is very little room next to the bar. Java, Minecraft and Forge are
/// names and read the same in every language.
pub fn stage_short(stage: SyncStage) -> String {
    match stage {
        SyncStage::DownloadingJava => "Java".into(),
        SyncStage::DownloadingMinecraft => "Minecraft".into(),
        SyncStage::ApplyingForgePatches => "Forge".into(),
        SyncStage::CheckingFiles => t("sync-short-checking"),
        SyncStage::DownloadingLibraries => t("sync-short-libraries"),
        SyncStage::DownloadingAssets => t("sync-short-assets"),
        SyncStage::DownloadingMods => t("sync-short-mods"),
        SyncStage::Cleaning => t("sync-short-cleaning"),
        SyncStage::Done => t("sync-done"),
    }
}

pub fn megabytes(done: u64, total: u64) -> String {
    let mut args = FluentArgs::new();
    args.set("done", format!("{:.0}", mb(done)));
    args.set("total", format!("{:.0}", mb(total)));
    i18n::t_args("sync-megabytes", &args)
}

fn mb(bytes: u64) -> f64 {
    bytes as f64 / 1_048_576.0
}

/// Recent byte counts, for a speed that follows the link rather than the
/// average since the start.
#[derive(Default, Clone)]
pub struct Rate {
    samples: VecDeque<(Instant, u64)>,
}

const WINDOW: Duration = Duration::from_secs(5);

impl Rate {
    pub fn record(&mut self, done: u64) {
        let now = Instant::now();
        if self.samples.back().is_some_and(|(_, d)| *d > done) {
            // A rolled-back partial or a new pass: start over.
            self.samples.clear();
        }
        self.samples.push_back((now, done));
        while self
            .samples
            .front()
            .is_some_and(|(at, _)| now.duration_since(*at) > WINDOW)
            && self.samples.len() > 2
        {
            self.samples.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Bytes per second, once there is a second's worth to go on.
    pub fn bytes_per_sec(&self) -> Option<f64> {
        let (t0, d0) = self.samples.front()?;
        let (t1, d1) = self.samples.back()?;
        let secs = t1.duration_since(*t0).as_secs_f64();
        (secs >= 1.0 && d1 > d0).then(|| (d1 - d0) as f64 / secs)
    }
}

/// "4.2 MB/s · 1:23 left", or nothing while the speed isn't known yet.
pub fn rate_label(rate: &Rate, done: u64, total: u64) -> Option<String> {
    let speed = rate.bytes_per_sec()?;
    let left = total.saturating_sub(done) as f64 / speed;
    let left = left.min(99.0 * 3600.0) as u64;
    let eta = if left >= 3600 {
        format!("{}:{:02}:{:02}", left / 3600, left / 60 % 60, left % 60)
    } else {
        format!("{}:{:02}", left / 60, left % 60)
    };
    let mut args = FluentArgs::new();
    args.set("speed", format!("{:.1}", mb(speed as u64)));
    args.set("eta", eta);
    Some(i18n::t_args("sync-rate", &args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_speed_before_a_second_of_samples() {
        let mut rate = Rate::default();
        rate.record(0);
        rate.record(1_000_000);
        assert!(rate.bytes_per_sec().is_none());
    }

    #[test]
    fn a_rollback_starts_over() {
        let mut rate = Rate::default();
        rate.record(10);
        rate.record(5);
        assert_eq!(rate.samples.len(), 1);
    }
}
