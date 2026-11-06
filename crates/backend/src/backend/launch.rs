// Over 150 lines: syncing and launching are one task with one cancel handle;
// apart, the hand-off between them would need state of its own.
//! Syncing a build and running the game.

use super::*;

const PROGRESS_INTERVAL: Duration = Duration::from_millis(50);

/// Everything needed to sync a build and start the game.
pub struct Launch {
    pub ctx: Ctx,
    pub server_id: Uuid,
    pub manifest: BuildManifest,
    pub user: UserProfile,
    pub login: LoginInfo,
    pub connect: Option<ServerConnect>,
    pub enabled_optional: Vec<String>,
    /// The build's card; its game servers end up in the instance's servers.dat.
    pub server: Option<ServerEntry>,
    /// Aliases of every server the player can reach, for the agent's table.
    pub aliases: crate::host_aliases::HostAliases,
    pub modal: bridge::ModalAction,
    /// Keeps a move of the data root out until the game has exited.
    pub data_use: crate::data_location::UseGuard,
}

pub fn spawn_sync_and_launch(req: Launch) {
    let Launch {
        ctx,
        server_id,
        manifest,
        user,
        login,
        connect,
        enabled_optional,
        server,
        aliases,
        modal,
        data_use,
    } = req;
    tokio::spawn(async move {
        let _data_use = data_use;
        let instance_dir = ctx.dirs.instance(&server_id);

        let to_fe = ctx.frontend.clone();
        let modal_clone = modal.clone();
        // Download stages run in parallel but the modal has one bar. Keeping the
        // last report per stage and showing the sum stops the bar jumping back
        // and forth with whichever stage reported last.
        let totals: Arc<Mutex<BTreeMap<bridge::SyncStage, (u64, u64)>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        // The window hears from each stage every 50ms at most, plus its first
        // and last report. A sync reports every file, and the 13 000 messages
        // of one GTNH install buried the one saying the sync was over.
        let last_sent: Arc<Mutex<BTreeMap<bridge::SyncStage, Instant>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let progress: crate::sync::ProgressFn = Arc::new(move |stage, done, total, file| {
            let due = {
                let mut last = last_sent.lock();
                let now = Instant::now();
                let due = done == 0
                    || done >= total
                    || last
                        .get(&stage)
                        .is_none_or(|at| now.duration_since(*at) >= PROGRESS_INTERVAL);
                if due {
                    last.insert(stage, now);
                }
                due
            };
            if due {
                to_fe.send(MessageToFrontend::SyncProgress {
                    server_id,
                    stage,
                    done,
                    total,
                    file: file.clone(),
                });
            }
            if stage.is_download() {
                let (sum_done, sum_total) = {
                    let mut g = totals.lock();
                    g.insert(stage, (done, total));
                    g.values()
                        .fold((0u64, 0u64), |(d, t), (sd, st)| (d + sd, t + st))
                };
                modal_clone.set_stage("downloading");
                modal_clone.set_progress(sum_done, sum_total);
            } else {
                modal_clone.set_stage(format!("{stage:?}"));
                modal_clone.set_progress(done, total);
            }
            if !file.is_empty() {
                modal_clone.set_detail(file);
            }
        });

        let cancelled_modal = modal.clone();
        let cancelled: Arc<dyn Fn() -> bool + Send + Sync> =
            Arc::new(move || cancelled_modal.is_cancelled());

        let sync_result = crate::sync::sync_server(
            &ctx.http,
            &instance_dir,
            &manifest,
            &enabled_optional,
            &user,
            progress,
            cancelled,
        )
        .await;

        if let Err(e) = sync_result {
            // A cancel surfaces as an error from deep inside the download; it
            // is the player's choice, not a failure to report.
            if modal.is_cancelled() {
                ctx.send(MessageToFrontend::LaunchCancelled { server_id });
                return;
            }
            // `{:#}` keeps the cause chain: "download of X failed: SHA1
            // mismatch" rather than only the outermost context.
            let detail = format!("{e:#}");
            tracing::error!(%server_id, error = %detail, "sync failed");
            modal.fail(detail.clone());
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::sync_failure_key(&e).into(),
                detail,
            });
            return;
        }
        ctx.send(MessageToFrontend::SyncComplete { server_id });
        // The files are in place, so the button has to stop offering to install
        // or update.
        ctx.send(MessageToFrontend::BuildStateChanged {
            server_id,
            state: crate::sync::build_state(&instance_dir, &manifest),
        });
        modal.finish();
        crate::offline_cache::save_manifest(&ctx.dirs, &manifest);

        // Nothing looks at the directory between the sync and the launch, so
        // check it against the manifest here. Extra files go, mismatches go to
        // the master, and the player keeps launching: a finding is something to
        // look into later, not a refusal.
        ctx.send(MessageToFrontend::LaunchStep {
            server_id,
            step: bridge::LaunchStep::Verifying,
        });
        let report =
            crate::sync::verify_before_launch(&instance_dir, &manifest, &enabled_optional, &user)
                .await;
        if !report.findings.is_empty() {
            tracing::warn!(findings = report.findings.len(), "found mismatched files");
        }
        // Only when something was actually put right. A new pack the player
        // added is a finding for the master, not a repair to announce.
        if report.findings.iter().any(|f| f.repaired) {
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-build-files-restored".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Info,
            });
        }
        let blocked = report.block_launch;
        ctx.ws.send(ClientWsMsg::ReportIntegrity { report });
        if blocked {
            // The file is left where it is: the player has to see what is
            // holding them up, and deleting it quietly would look like the
            // launcher breaking.
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-launch-blocked".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Error,
            });
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::LAUNCH_BLOCKED.into(),
                detail: String::new(),
            });
            return;
        }

        // Delivered packs are switched on the first time they arrive.
        crate::sync::live::enable_delivered_packs(
            &instance_dir,
            &manifest,
            &enabled_optional,
            &user,
        )
        .await;
        let legacy_prefix_pack = instance_dir.join("resourcepacks/noro-prefixes.zip");
        if legacy_prefix_pack.exists() {
            let _ = tokio::fs::remove_file(&legacy_prefix_pack).await;
        }

        let launch_config = ctx
            .config
            .get()
            .launch_config_for_server(&server_id, &manifest.recommended_client_settings);

        // Before servers.dat and the join address: whether they may carry
        // aliases depends on the agent this launch gets.
        let (agent, aliases) = match game_runner::prepare_agent(
            &ctx.http,
            &launch_config,
            &ctx.dirs,
            &instance_dir,
            &aliases,
        )
        .await
        {
            Ok(prepared) => prepared,
            Err(e) => {
                report_launch_failure(&ctx, server_id, e);
                return;
            }
        };
        let connect = connect.map(|c| c.via_alias(&aliases));

        // After the sync but before the launch: the game reads servers.dat at
        // start and rewrites it on exit. A broken server list is no reason to
        // keep the player out, so failures are only logged.
        if let Some(server) = &server {
            match crate::servers_dat::sync(&instance_dir, server, &aliases) {
                Ok(true) => tracing::info!("servers.dat updated from the build's game servers"),
                Ok(false) => {}
                Err(e) => tracing::warn!("servers.dat not updated: {e}"),
            }
        }
        let server_name = server
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Minecraft".into());
        let online = server.as_ref().and_then(|s| s.online);
        let max_online = server.as_ref().and_then(|s| s.max_online);

        // Cancelled while the files were being checked: stop before the game.
        if modal.is_cancelled() {
            ctx.send(MessageToFrontend::LaunchCancelled { server_id });
            return;
        }

        ctx.send(MessageToFrontend::LaunchStep {
            server_id,
            step: bridge::LaunchStep::Starting,
        });

        // The channel to the case mod has to be up before the game starts: the
        // mod reads the handshake file once, at startup, and being late here
        // means no panel until the next login.
        ctx.mod_link.start(&ctx, instance_dir.clone()).await;

        // Without a ticket the game still starts; only mods that insist on one
        // refuse, and they say why.
        let ticket = match crate::master_api::MasterApi::for_session(&ctx) {
            Some(api) => api
                .launch_ticket()
                .await
                .inspect_err(|e| tracing::warn!("no launch ticket: {e:#}"))
                .ok(),
            None => None,
        };

        match game_runner::launch(
            &launch_config,
            &ctx.dirs,
            &server_id,
            &manifest,
            &login,
            connect,
            ticket.as_deref(),
            &agent,
        )
        .await
        {
            Ok(child) => {
                run_game_process(ctx, server_id, server_name, online, max_online, child).await;
            }
            Err(e) => {
                ctx.mod_link.stop().await;
                report_launch_failure(&ctx, server_id, e);
            }
        }
    });
}

fn report_launch_failure(ctx: &Ctx, server_id: Uuid, e: anyhow::Error) {
    tracing::error!(%server_id, error = %format!("{e:#}"), "launch failed");
    let key = match crate::failure::sync_failure_key(&e) {
        "sync-error-unknown" => crate::failure::LAUNCH_FAILED,
        key => key,
    };
    ctx.send(MessageToFrontend::SyncFailed {
        server_id,
        reason: key.into(),
        detail: format!("{e:#}"),
    });
}

async fn run_game_process(
    ctx: Ctx,
    server_id: Uuid,
    server_name: String,
    online: Option<u32>,
    max_online: Option<u32>,
    mut child: tokio::process::Child,
) {
    let started = Instant::now();
    let start_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel::<()>();
    ctx.running.lock().insert(
        server_id,
        RunningGame {
            started,
            kill: kill_tx,
        },
    );

    ctx.rpc
        .update(crate::discord_rpc::DiscordRpcState::GameMenu {
            server_name: server_name.clone(),
            start_timestamp,
        });

    ctx.send(MessageToFrontend::GameStarted { server_id });
    ctx.ws.send(ClientWsMsg::ReportGameStart { server_id });

    // Both pipes feed one scan: the JVM's own complaints come on stderr, the
    // game's crash report on stdout.
    let crash_scan = std::sync::Arc::new(parking_lot::Mutex::new(
        crate::crash_hint::CrashScan::default(),
    ));
    let mut readers = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        readers.push(tokio::spawn(crate::log_reader::spawn_log_reader(
            stdout,
            server_id,
            ctx.frontend.clone(),
            false,
            Some(crate::log_reader::RpcLogContext {
                rpc: ctx.rpc.clone(),
                server_name: server_name.clone(),
                start_timestamp,
                online_current: online,
                online_max: max_online,
            }),
            Some(crash_scan.clone()),
        )));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(tokio::spawn(crate::log_reader::spawn_log_reader(
            stderr,
            server_id,
            ctx.frontend.clone(),
            true,
            None,
            Some(crash_scan.clone()),
        )));
    }

    // A stop the player asked for is not a failure: reporting it as one put an
    // error toast and the console in front of someone who just pressed Stop.
    let exit_ok = tokio::select! {
        status = child.wait() => status.map(|s| s.success()).unwrap_or(false),
        _ = kill_rx.recv() => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            true
        }
    };

    // The last lines — the crash report itself — may still be in the pipes when
    // the process is gone. The pipes close with it, so the wait is short; the cap
    // is for a child that left a grandchild holding them open.
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        futures::future::join_all(readers),
    )
    .await;
    let hint = (!exit_ok)
        .then(|| crash_scan.lock().hint())
        .flatten()
        .map(|h| h.key().to_string());

    // The game is gone, so the channel comes down with it: a handshake file
    // left on disk promises access that no longer exists.
    ctx.mod_link.stop().await;

    let playtime = started.elapsed().as_secs();
    ctx.running.lock().remove(&server_id);
    ctx.ws.send(ClientWsMsg::ReportGameStop {
        server_id,
        playtime_secs: playtime,
    });
    ctx.send(MessageToFrontend::GameStopped {
        server_id,
        exit_ok,
        hint,
    });

    ctx.rpc
        .update(crate::discord_rpc::DiscordRpcState::Launcher {
            server_name: Some(server_name),
        });
}
