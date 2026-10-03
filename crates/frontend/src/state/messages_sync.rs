// Over 150 lines: one handler for every sync message; split, the match would
// only move.
//! Backend messages about servers, builds, downloads and running games.

use super::*;

impl LauncherUI {
    /// Only ever gets the variants `on_message` routes here.
    pub(super) fn on_sync_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::ServerList { servers } => self.replace_servers(servers, cx),
            MessageToFrontend::OptionalMods {
                server_id,
                mods,
                allow_suggestions,
                allow_personal,
                installed_files,
            } => {
                self.optional_mods.insert(server_id, mods);
                self.allow_mod_suggestions
                    .insert(server_id, allow_suggestions);
                self.allow_personal_content
                    .insert(server_id, allow_personal);
                if let Some(files) = installed_files {
                    let file_keys = files
                        .iter()
                        .filter_map(|f| {
                            std::path::Path::new(f)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(crate::pages::normalized_mod_name)
                        })
                        .filter(|k| !k.is_empty())
                        .collect();
                    self.installed_file_keys.insert(server_id, file_keys);
                }
                let mut keys: std::collections::HashSet<String> = self
                    .optional_mods
                    .get(&server_id)
                    .map(|mods| {
                        mods.iter()
                            .map(|m| crate::pages::normalized_mod_name(&m.name))
                            .filter(|k| !k.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(file_keys) = self.installed_file_keys.get(&server_id) {
                    keys.extend(file_keys.iter().cloned());
                }
                self.installed_keys.insert(server_id, keys);
            }
            MessageToFrontend::ServerClientRecommendation {
                server_id,
                settings,
            } => {
                self.server_recommendations.insert(server_id, settings);
            }
            MessageToFrontend::SyncProgress {
                server_id,
                stage,
                done,
                total,
                file,
            } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = stage != SyncStage::Done;
                let cancelling = s.heading == Some(SyncHeading::Cancelling);
                if stage.is_download() {
                    s.stages.insert(stage, (done, total));
                    s.rate.record(s.done());
                    if !cancelling {
                        s.heading = Some(SyncHeading::Downloading);
                    }
                } else {
                    // Checking files opens a new pass; the bars from the last
                    // run don't belong to it.
                    if stage == SyncStage::CheckingFiles && done == 0 {
                        s.stages.clear();
                        s.rate.clear();
                    }
                    if !cancelling {
                        s.heading = Some(SyncHeading::Stage(stage));
                    }
                }
                if !file.is_empty() {
                    s.detail = file;
                }
                s.failed = None;
            }
            MessageToFrontend::SyncComplete { server_id } => {
                // Not playable yet: the files get checked and the JVM started,
                // which for a big build takes seconds. The button stays busy
                // until GameStarted or SyncFailed — a second click here started
                // a second game in the same folder.
                let s = self.sync.entry(server_id).or_default();
                s.heading = Some(SyncHeading::Launching);
            }
            MessageToFrontend::LaunchStep { server_id, step } => {
                let s = self.sync.entry(server_id).or_default();
                // A cancel already under way keeps its heading.
                if s.heading != Some(SyncHeading::Cancelling) {
                    s.heading = Some(match step {
                        bridge::LaunchStep::Verifying => {
                            SyncHeading::Stage(SyncStage::CheckingFiles)
                        }
                        bridge::LaunchStep::Starting => SyncHeading::Launching,
                    });
                }
            }
            MessageToFrontend::LaunchCancelled { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.failed = None;
                s.launch = None;
                s.heading = None;
                s.stages.clear();
                s.rate.clear();
            }
            MessageToFrontend::LiveSynced {
                server_id: _,
                updated,
                locked,
            } => {
                // New packs arrived while the game is running. Nothing shows
                // until the client reloads its resources, and that's the
                // player's call: mid-fight it isn't welcome. Said in a toast:
                // the sync panel this used to go to is hidden while playing.
                let text = if locked.is_empty() {
                    i18n::t_count("sync-live-updated", updated.len() as i64)
                } else {
                    let mut args = i18n::FluentArgs::new();
                    args.set("count", updated.len() as i64);
                    args.set("locked", locked.len() as i64);
                    i18n::t_args("sync-live-partial", &args)
                };
                self.notify_toast(text, NotifLevel::Info, cx);
            }
            MessageToFrontend::SyncFailed {
                server_id,
                reason,
                detail,
            } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.launch = None;
                s.heading = None;
                s.rate.clear();
                self.notify_toast(i18n::t(&reason), NotifLevel::Error, cx);
                // The technical chain goes where people look when a toast isn't
                // enough: the console, and from there a support report.
                if !detail.is_empty() {
                    let logs = self.logs.entry(server_id).or_default();
                    logs.push_back(LogEntry {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        level: bridge::GameLogLevel::Error,
                        text: format!("[launcher] {detail}"),
                    });
                }
                self.sync.entry(server_id).or_default().failed =
                    Some(SyncFailure { reason, detail });
            }
            MessageToFrontend::GameStarted { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.running = true;
                s.syncing = false;
                s.launch = None;
                s.heading = None;
                if self
                    .server_client_settings(server_id)
                    .show_console_on_launch
                {
                    self.open_console(server_id, cx);
                }
            }
            MessageToFrontend::GameStopped { server_id, exit_ok } => {
                let s = self.sync.entry(server_id).or_default();
                s.running = false;
                if !exit_ok {
                    if self
                        .server_client_settings(server_id)
                        .show_console_on_launch
                    {
                        self.open_console(server_id, cx);
                    }
                    self.notify_toast(i18n::t("error-game-exited"), NotifLevel::Warning, cx);
                }
            }
            MessageToFrontend::BuildStateChanged { server_id, state } => {
                self.build_state.insert(server_id, state);
            }
            MessageToFrontend::JavaRuntimes {
                server_id,
                options,
                default_component,
                selected,
            } => {
                self.java_options.insert(server_id, options);
                self.java_default.insert(server_id, default_component);
                self.java_selected.insert(server_id, selected);
                self.java_busy = false;
            }

            _ => {}
        }
    }
}
