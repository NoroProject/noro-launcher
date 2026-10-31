//! Download order and per-stage concurrency.

use bridge::SyncStage;
use schema::ArtifactKind;

pub(super) struct StageGroup {
    pub stage: SyncStage,
    /// How the sync report names the stage.
    pub name: &'static str,
    pub kinds: &'static [ArtifactKind],
    pub concurrency: usize,
}

/// Concurrency is picked by file size. Assets are thousands of a few kilobytes
/// each, so the time goes into round-trips and they want many requests in
/// flight. Large archives are the opposite: they share one link and all finish
/// later than they would in sequence.
///
/// Measured on a real modpack (3357 files, 107 MB, warm CDN, 31.10.2026): 48
/// in flight took 19.9 s, 128 took 16.8 s, 256 took 17.6 s. Past ~128 the
/// round trips are hidden and the link is the limit. The requests spread over
/// `http::SYNC_CONNECTIONS` connections, so 128 is 32 streams on each.
pub(super) const STAGE_GROUPS: &[StageGroup] = &[
    StageGroup {
        stage: SyncStage::DownloadingJava,
        name: "java",
        kinds: &[ArtifactKind::Java],
        concurrency: 64,
    },
    StageGroup {
        stage: SyncStage::DownloadingMinecraft,
        name: "minecraft",
        kinds: &[ArtifactKind::ClientJar],
        // One file, nothing to parallelise.
        concurrency: 2,
    },
    StageGroup {
        stage: SyncStage::DownloadingLibraries,
        name: "libraries",
        kinds: &[
            ArtifactKind::Library,
            ArtifactKind::Runtime,
            ArtifactKind::Native,
        ],
        concurrency: 64,
    },
    StageGroup {
        stage: SyncStage::DownloadingAssets,
        name: "assets",
        kinds: &[ArtifactKind::Asset, ArtifactKind::AssetIndex],
        // Multiplexed over the pool's few connections, so this isn't 128 sockets.
        concurrency: 128,
    },
    StageGroup {
        stage: SyncStage::DownloadingMods,
        name: "mods",
        kinds: &[ArtifactKind::Mod, ArtifactKind::Config, ArtifactKind::Other],
        concurrency: 128,
    },
];
