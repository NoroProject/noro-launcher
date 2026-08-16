//! Порядок и параллельность загрузки по видам артефактов.

use bridge::SyncStage;
use schema::ArtifactKind;

pub(super) struct StageGroup {
    pub stage: SyncStage,
    pub kinds: &'static [ArtifactKind],
    pub concurrency: usize,
}

/// Сопоставление стадий и категорий артефактов.
///
/// Параллелизм подобран под размер файлов: у ассетов их тысячи по несколько
/// килобайт, и время уходит на round-trip, а не на передачу — им нужно много
/// запросов сразу. Крупным архивам это наоборот вредит: они делят один канал
/// и все финишируют позже, чем если бы качались по очереди.
pub(super) const STAGE_GROUPS: &[StageGroup] = &[
    StageGroup {
        stage: SyncStage::DownloadingJava,
        kinds: &[ArtifactKind::Java],
        concurrency: 8,
    },
    StageGroup {
        stage: SyncStage::DownloadingMinecraft,
        kinds: &[ArtifactKind::ClientJar],
        // Один файл — параллелить нечего.
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
        // Мультиплексируются в одно HTTP/2-соединение, так что это не 48 сокетов.
        concurrency: 48,
    },
    StageGroup {
        stage: SyncStage::DownloadingMods,
        kinds: &[ArtifactKind::Mod, ArtifactKind::Config, ArtifactKind::Other],
        concurrency: 12,
    },
];
