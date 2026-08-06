use super::{classpath, LoginInfo, ServerConnect};
use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest};
use std::path::Path;
use tokio::process::Command;

pub struct Substitution<'a> {
    pub instance_dir: &'a Path,
    pub natives_dir: &'a Path,
    pub classpath: &'a str,
    pub manifest: &'a BuildManifest,
    pub login: &'a LoginInfo,
    pub primary_game_artifact: &'a str,
}

pub fn push_jvm_args(cmd: &mut Command, ctx: &Substitution<'_>, loader_client_name: Option<&str>) {
    if ctx.manifest.jvm_args.is_empty() {
        cmd.arg("-cp").arg(ctx.classpath);
        cmd.arg(format!(
            "-Djava.library.path={}",
            ctx.natives_dir.to_string_lossy()
        ));
        return;
    }

    for arg in &ctx.manifest.jvm_args {
        for s in get_arg_values(arg) {
            let mut substituted = substitute(&s, ctx);
            if substituted.starts_with("-DignoreList=") {
                classpath::remove_from_ignore_list(&mut substituted, loader_client_name);
            }
            cmd.arg(substituted);
        }
    }
}

pub fn push_game_args(cmd: &mut Command, ctx: &Substitution<'_>, connect: Option<ServerConnect>) {
    if ctx.manifest.game_args.is_empty() {
        for (key, value) in default_game_args(ctx) {
            cmd.arg(key).arg(value);
        }
    } else {
        for arg in &ctx.manifest.game_args {
            for s in get_arg_values(arg) {
                cmd.arg(substitute(&s, ctx));
            }
        }
    }
    if let Some(server) = connect {
        cmd.arg("--quickPlayMultiplayer")
            .arg(format!("{}:{}", server.host, server.port));
    }
}

fn rules_allow(rules: &[schema::ManifestRule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        let action_allow = rule.action == "allow";
        if rule.features.is_some() {
            continue;
        }
        let os_matches = match &rule.os {
            None => true,
            Some(os) => {
                let name_ok = os.name.as_deref().map(|n| n == mojang_os_name()).unwrap_or(true);
                let arch_ok = os.arch.as_deref().map(|a| a == mojang_arch()).unwrap_or(true);
                name_ok && arch_ok
            }
        };
        if os_matches {
            allowed = action_allow;
        }
    }
    allowed
}

fn mojang_os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        "linux" => "linux",
        "windows" => "windows",
        _ => "linux",
    }
}

fn mojang_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x86_64",
    }
}

fn get_arg_values(arg: &schema::ManifestArg) -> Vec<String> {
    match arg {
        schema::ManifestArg::String(s) => vec![s.clone()],
        schema::ManifestArg::Conditional { rules, value } => {
            if rules_allow(rules) {
                value.clone()
            } else {
                Vec::new()
            }
        }
    }
}

fn substitute(arg: &str, ctx: &Substitution<'_>) -> String {
    let assets_root = ctx.instance_dir.join("assets");
    let library_dir = ctx.instance_dir.join("libraries");
    let game_jar = ctx
        .manifest
        .verified_files
        .iter()
        .find(|f| ctx.manifest.kind_of(&f.path) == ArtifactKind::ClientJar)
        .and_then(|f| safe_join(ctx.instance_dir, &f.path))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let replacements: &[(&str, String)] = &[
        ("${auth_player_name}", ctx.login.username.clone()),
        ("${version_name}", ctx.manifest.version.clone()),
        (
            "${game_directory}",
            ctx.instance_dir.to_string_lossy().into_owned(),
        ),
        ("${assets_root}", assets_root.to_string_lossy().into_owned()),
        ("${game_assets}", assets_root.to_string_lossy().into_owned()),
        (
            "${assets_index_name}",
            ctx.manifest.assets_index_name.clone(),
        ),
        ("${auth_uuid}", ctx.login.uuid.clone()),
        ("${auth_access_token}", ctx.login.access_token.clone()),
        ("${auth_session}", ctx.login.access_token.clone()),
        ("${clientid}", String::new()),
        ("${auth_xuid}", String::new()),
        ("${user_type}", "msa".to_string()),
        ("${user_properties}", "{}".to_string()),
        ("${version_type}", "release".to_string()),
        (
            "${natives_directory}",
            ctx.natives_dir.to_string_lossy().into_owned(),
        ),
        ("${launcher_name}", "noro".to_string()),
        ("${launcher_version}", env!("CARGO_PKG_VERSION").to_string()),
        ("${classpath}", ctx.classpath.to_string()),
        ("${game_jar}", game_jar),
        (
            "${primary_game_artifact}",
            ctx.primary_game_artifact.to_string(),
        ),
        (
            "${library_directory}",
            library_dir.to_string_lossy().into_owned(),
        ),
        (
            "${classpath_separator}",
            classpath::classpath_separator().to_string(),
        ),
    ];

    let mut out = arg.to_string();
    for (key, value) in replacements {
        out = out.replace(key, value);
    }
    out
}

fn default_game_args(ctx: &Substitution<'_>) -> Vec<(&'static str, String)> {
    vec![
        ("--username", ctx.login.username.clone()),
        ("--uuid", ctx.login.uuid.clone()),
        ("--accessToken", ctx.login.access_token.clone()),
        ("--version", ctx.manifest.version.clone()),
        ("--gameDir", ctx.instance_dir.to_string_lossy().into_owned()),
        (
            "--assetsDir",
            ctx.instance_dir
                .join("assets")
                .to_string_lossy()
                .into_owned(),
        ),
        ("--assetIndex", ctx.manifest.assets_index_name.clone()),
        ("--userType", "msa".to_string()),
        ("--versionType", "release".to_string()),
    ]
}
