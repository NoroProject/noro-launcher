//! Download order and per-stage concurrency.

use bridge::SyncStage;
use schema::ArtifactKind;

pub(super) struct StageGroup {
    pub stage: SyncStage,
    pub kinds: &'static [ArtifactKind],
    pub concurrency: usize,
}

/// Concurrency is picked by file size. Assets are thousands of a few kilobytes
/// each, so the time goes into round-trips and they want many requests in
/// flight. Large archives are the opposite: they share one link and all finish
/// later than they would in sequence.
pub(super) const STAGE_GROUPS: &[StageGroup] = &[
    StageGroup {
        stage: SyncStage::DownloadingJava,
        kinds: &[ArtifactKind::Java],
        concurrency: 8,
    },
    StageGroup {
        stage: SyncStage::DownloadingMinecraft,
        kinds: &[ArtifactKind::ClientJar],
        // One file, nothing to parallelise.
        concurrency: 2,
    },
    StageGroup {
        stage: SyncStage::DownloadingLibraries,
        kinds: &[
            ArtifactKind::Library,
            ArtifactKind::Runtime,
            ArtifactKind::Native,
        ],
        concurrency: 16,
    },
    StageGroup {
        stage: SyncStage::DownloadingAssets,
        kinds: &[ArtifactKind::Asset, ArtifactKind::AssetIndex],
        // Multiplexed onto one HTTP/2 connection, so this isn't 48 sockets.
        concurrency: 48,
    },
    StageGroup {
        stage: SyncStage::DownloadingMods,
        kinds: &[ArtifactKind::Mod, ArtifactKind::Config, ArtifactKind::Other],
        concurrency: 12,
    },
];
