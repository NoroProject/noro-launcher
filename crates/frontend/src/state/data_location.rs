// Over 150 lines: the folder's state as the backend reports it, and the
// player's steps through a move — pick, confirm, watch, resolve.
//! The game files folder in the window: where it is, and moving it.

use super::*;
use bridge::{DataLocationView, DataMoveOutcome, DataMovePlan, DataMoveProgress, DataStatus};
use gpui::PathPromptOptions;
use std::collections::BTreeMap;

/// A translation key with its arguments, as the backend sends them.
pub type DataText = (String, BTreeMap<String, String>);

#[derive(Default)]
pub struct DataUi {
    /// `None` until the backend's first report, which comes at start.
    pub view: Option<DataLocationView>,
    /// The size is being counted; a second click doesn't count it again.
    pub counting: bool,
    pub plan: Option<DataMovePlan>,
    pub progress: Option<DataMoveProgress>,
    pub rate: crate::sync_text::Rate,
    /// The last refusal or outcome, under the controls.
    pub note: Option<(DataText, NotifLevel)>,
    /// The start-up prompt was put off with «Later».
    pub prompt_dismissed: bool,
}

impl DataUi {
    pub fn moving(&self) -> bool {
        self.progress.is_some()
    }

    pub fn status(&self) -> DataStatus {
        self.view
            .as_ref()
            .map(|v| v.status.clone())
            .unwrap_or_default()
    }
}

impl LauncherUI {
    /// The settings page, with the folder's size counted afresh: the game
    /// adds worlds and screenshots, so a count from last time is stale.
    pub fn open_settings(&mut self) {
        self.page = Page::Settings;
        if !self.data.counting && !self.data.moving() {
            self.data.counting = true;
            self.backend.send(MessageToBackend::RequestDataLocation);
        }
    }

    /// What blocks a move on the window's side; the backend checks again.
    fn data_busy(&self) -> bool {
        self.data.moving() || self.sync.values().any(|s| s.running || s.syncing)
    }

    pub fn pick_data_folder(&mut self, cx: &mut Context<Self>) {
        if self.data_busy() {
            self.data_refused("data-move-refused-busy");
            return;
        }
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(i18n::t("settings-data-pick-title").into()),
        });
        cx.spawn(async move |this, cx| {
            // Cancelled, dropped or failed: the dialog said all there was.
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(target) = paths.into_iter().next() else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                this.plan_data_move(target);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn plan_data_move(&mut self, target: std::path::PathBuf) {
        self.data.note = None;
        self.data.plan = None;
        self.backend.send(MessageToBackend::PlanDataMove { target });
    }

    pub fn move_data_back(&mut self) {
        if self.data_busy() {
            self.data_refused("data-move-refused-busy");
            return;
        }
        if let Some(view) = &self.data.view {
            let default = view.default_path.clone();
            self.plan_data_move(default);
        }
    }

    pub fn start_data_move(&mut self) {
        if self.data_busy() {
            self.data_refused("data-move-refused-busy");
            return;
        }
        if let Some(plan) = self.data.plan.take() {
            self.data.rate.clear();
            self.backend.send(MessageToBackend::StartDataMove {
                target: plan.target,
            });
        }
    }

    pub fn cancel_data_plan(&mut self) {
        self.data.plan = None;
    }

    pub fn cancel_data_move(&mut self) {
        self.backend.send(MessageToBackend::CancelDataMove);
    }

    /// Finish or take back a move that was interrupted. Its progress shows on
    /// the settings page.
    pub fn resolve_data_move(&mut self, resume: bool) {
        self.data.prompt_dismissed = true;
        if resume {
            self.data.rate.clear();
            self.page = Page::Settings;
        }
        self.backend
            .send(MessageToBackend::ResolveDataMove { resume });
    }

    pub fn retry_data_location(&mut self) {
        self.backend.send(MessageToBackend::RequestDataLocation);
    }

    pub fn reset_data_location(&mut self) {
        self.data.prompt_dismissed = true;
        self.backend.send(MessageToBackend::ResetDataLocation);
    }

    pub fn open_data_folder(&self) {
        self.backend.send(MessageToBackend::OpenDataFolder);
    }

    fn data_refused(&mut self, key: &str) {
        self.data.note = Some(((key.to_string(), BTreeMap::new()), NotifLevel::Warning));
    }

    pub(super) fn on_data_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::DataLocation { view } => {
                // Every answer ends a count, even one without a size: a
                // folder that could not be walked must not say «counting» forever.
                self.data.counting = false;
                // Back from unavailable: the prompt may show again next time.
                if view.status == DataStatus::Ready {
                    self.data.prompt_dismissed = false;
                }
                self.data.view = Some(view);
            }
            MessageToFrontend::DataMovePlanned { plan } => self.data.plan = Some(plan),
            MessageToFrontend::DataMoveRefused { key, args } => {
                self.data.note = Some(((key, args), NotifLevel::Warning));
            }
            MessageToFrontend::DataMoveProgress { progress } => {
                // Each phase counts its bytes from zero; the speed starts over.
                if self
                    .data
                    .progress
                    .as_ref()
                    .is_some_and(|p| p.phase != progress.phase)
                {
                    self.data.rate.clear();
                }
                self.data.rate.record(progress.done_bytes);
                self.data.progress = Some(progress);
            }
            MessageToFrontend::DataMoveFinished { outcome } => {
                self.data.progress = None;
                self.data.rate.clear();
                self.on_data_outcome(outcome, cx);
            }
            _ => {}
        }
    }

    fn on_data_outcome(&mut self, outcome: DataMoveOutcome, cx: &mut Context<Self>) {
        let path_arg =
            |p: &std::path::Path| BTreeMap::from([("path".to_string(), p.display().to_string())]);
        let (text, level) = match outcome {
            DataMoveOutcome::Done {
                leftover: Some(left),
                ..
            } => (
                ("data-move-done-leftover".to_string(), path_arg(&left)),
                NotifLevel::Warning,
            ),
            DataMoveOutcome::Done {
                path,
                leftover: None,
            } => (
                ("data-move-done".to_string(), path_arg(&path)),
                NotifLevel::Info,
            ),
            DataMoveOutcome::Cancelled => (
                ("data-move-cancelled".to_string(), BTreeMap::new()),
                NotifLevel::Info,
            ),
            DataMoveOutcome::Failed { path, key, args } => {
                self.notify_toast(
                    i18n::t_args("data-move-failed", &fluent(&path_arg(&path))),
                    NotifLevel::Error,
                    cx,
                );
                ((key, args), NotifLevel::Error)
            }
        };
        if level != NotifLevel::Error {
            self.notify_toast(data_text(&text), level, cx);
        }
        self.data.note = Some((text, level));
    }
}

/// Bytes as the player reads them: «12.3 GB», or «640 MB» below a gigabyte.
pub fn size_text(bytes: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    let mut args = i18n::FluentArgs::new();
    if bytes as f64 >= GB {
        args.set("value", format!("{:.1}", bytes as f64 / GB));
        i18n::t_args("settings-data-gb", &args)
    } else {
        args.set("value", format!("{:.0}", bytes as f64 / 1_048_576.0));
        i18n::t_args("settings-data-mb", &args)
    }
}

/// A backend key put into words. Byte counts arrive as plain numbers under
/// `need`, `free` and `size` and are formatted here.
pub fn data_text((key, args): &DataText) -> String {
    let mut readable = args.clone();
    for name in ["need", "free", "size"] {
        if let Some(bytes) = args.get(name).and_then(|v| v.parse::<u64>().ok()) {
            readable.insert(name.to_string(), size_text(bytes));
        }
    }
    i18n::t_args(key, &fluent(&readable))
}

fn fluent(args: &BTreeMap<String, String>) -> i18n::FluentArgs<'static> {
    let mut out = i18n::FluentArgs::new();
    for (name, value) in args {
        out.set(name.clone(), value.clone());
    }
    out
}
