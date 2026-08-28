//! Launching the game: classpath, argument substitution, authlib-injector, JVM.

mod args;
mod authlib;
mod classpath;
mod java;
mod rules;

use crate::config::LauncherConfig;
use crate::directories::LauncherDirectories;
use anyhow::{anyhow, Context, Result};
use schema::BuildManifest;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub use authlib::ensure_authlib_injector;
pub use classpath::classpath_separator;

/// From winbase.h; spawns the process without a console window.
#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct LoginInfo {
    pub username: String,
    /// Minecraft UUID, no dashes.
    pub uuid: String,
    pub access_token: String,
}

/// Where to connect on startup, when the player launched into a server.
pub struct ServerConnect {
    pub host: String,
    pub port: u16,
}

/// Returns the child process with stdout and stderr piped.
#[allow(clippy::too_many_arguments)]
pub async fn launch(
    client: &reqwest::Client,
    config: &LauncherConfig,
    dirs: &LauncherDirectories,
    server_id: &uuid::Uuid,
    manifest: &BuildManifest,
    login: &LoginInfo,
    connect: Option<ServerConnect>,
) -> Result<Child> {
    let instance_dir = dirs.instance(server_id);
    let natives_dir = dirs.natives(server_id);
    tokio::fs::create_dir_all(&natives_dir).await.ok();

    let java = crate::sync::find_java(&instance_dir, manifest)
        .ok_or_else(|| anyhow!("no java in the manifest"))?;
    #[cfg(unix)]
    java::ensure_executable(&java).await?;

    let classpath = classpath::build_classpath(&instance_dir, manifest);
    let primary_game_artifact =
        classpath::primary_game_artifact(&instance_dir, manifest).unwrap_or_default();

    let mut cmd = Command::new(&java);
    cmd.current_dir(&instance_dir);
    add_base_jvm_args(&mut cmd, config, &natives_dir);
    add_modlauncher_args(&mut cmd, &primary_game_artifact);

    let legacy_cp_path = write_legacy_classpath(&instance_dir, &classpath).await?;
    cmd.arg(format!(
        "-DlegacyClassPath.file={}",
        legacy_cp_path.to_string_lossy()
    ));
    cmd.arg(classpath::standard_ignore_list());

    add_authlib(client, config, dirs, &mut cmd).await;
    for flag in config.jvm_flags.split_whitespace() {
        cmd.arg(flag);
    }

    let ctx = args::Substitution {
        instance_dir: &instance_dir,
        natives_dir: &natives_dir,
        classpath: &classpath,
        manifest,
        login,
        primary_game_artifact: &primary_game_artifact,
    };
    args::push_jvm_args(
        &mut cmd,
        &ctx,
        classpath::loader_client_name(manifest).as_deref(),
    );

    cmd.arg(&manifest.main_class);
    args::push_game_args(&mut cmd, &ctx, connect, config.fullscreen);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    // java.exe is a console application and the launcher is a GUI process with
    // no console, so Windows opens a window for it. Closing that window sends
    // CTRL_CLOSE_EVENT to the whole process group and takes the game with it.
    // The flag doesn't affect the pipes; logs keep flowing.
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.spawn().context("could not start the JVM")
}

fn add_base_jvm_args(cmd: &mut Command, config: &LauncherConfig, natives_dir: &std::path::Path) {
    cmd.arg(format!("-Xms{}m", config.memory_min_mb));
    cmd.arg(format!("-Xmx{}m", config.memory_max_mb));
    cmd.arg(format!(
        "-Djava.library.path={}",
        natives_dir.to_string_lossy()
    ));
}

fn add_modlauncher_args(cmd: &mut Command, primary_game_artifact: &str) {
    if !primary_game_artifact.is_empty() {
        tracing::info!("primary game artifact: {}", primary_game_artifact);
        cmd.arg(format!(
            "-Dcpw.mods.modlauncher.primarygameartifact={primary_game_artifact}"
        ));
        cmd.arg("-Dcpw.mods.modlauncher.primarygameartifactid=minecraft");
    }
    cmd.arg("-Dcpw.mods.modlauncher.add-exports=java.base/sun.net.www.protocol.http=ALL-UNNAMED,java.base/sun.net.www.protocol.https=ALL-UNNAMED");
}

/// The classpath file a Forge launch reads. A failed write is propagated rather
/// than swallowed: without the file the game starts and then dies inside Java,
/// where the reason is no longer legible.
async fn write_legacy_classpath(
    instance_dir: &std::path::Path,
    classpath: &str,
) -> anyhow::Result<PathBuf> {
    let legacy_cp_path = instance_dir.join(".forge_classpath");
    let content = classpath
        .split(classpath_separator())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(&legacy_cp_path, content)
        .await
        .with_context(|| format!("writing {}", legacy_cp_path.display()))?;
    Ok(legacy_cp_path)
}

async fn add_authlib(
    client: &reqwest::Client,
    config: &LauncherConfig,
    dirs: &LauncherDirectories,
    cmd: &mut Command,
) {
    if let Ok(authlib) = ensure_authlib_injector(client, dirs).await {
        // authlib-injector wants the Yggdrasil API root, not the master's root:
        // it fetches the ALI metadata from there and derives the authserver and
        // sessionserver paths from it.
        let master_url = config.master_url.replace("localhost", "127.0.0.1");
        cmd.arg(format!(
            "-javaagent:{}={}/api/yggdrasil",
            authlib.to_string_lossy(),
            master_url.trim_end_matches('/')
        ));
    }
}
