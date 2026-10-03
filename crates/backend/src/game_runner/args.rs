use super::rules::arg_values;
use super::{classpath, LoginInfo, ServerConnect};
use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest};
use std::path::Path;
use tokio::process::Command;

pub struct Substitution<'a> {
    pub instance_dir: &'a Path,
    pub manifest: &'a BuildManifest,
    pub login: &'a LoginInfo,
    /// `${…}` → value, built once. Rebuilding it per argument cloned the whole
    /// classpath and searched the manifest for the client jar thirty times.
    table: Vec<(&'static str, String)>,
}

impl<'a> Substitution<'a> {
    pub fn new(
        instance_dir: &'a Path,
        natives_dir: &Path,
        classpath: &str,
        manifest: &'a BuildManifest,
        login: &'a LoginInfo,
        primary_game_artifact: &str,
    ) -> Self {
        let table = table(
            instance_dir,
            natives_dir,
            classpath,
            manifest,
            login,
            primary_game_artifact,
        );
        Self {
            instance_dir,
            manifest,
            login,
            table,
        }
    }
}

pub fn push_jvm_args(cmd: &mut Command, ctx: &Substitution<'_>, loader_client_name: Option<&str>) {
    let has_classpath = ctx
        .manifest
        .jvm_args
        .iter()
        .flat_map(arg_values)
        .any(|arg| matches!(arg.as_str(), "-cp" | "-classpath" | "--class-path"));
    // `-Djava.library.path` is already among the base arguments; passing it a
    // second time here only made the command line longer.
    if !has_classpath {
        cmd.arg("-cp").arg(lookup(ctx, "${classpath}"));
    }

    for arg in &ctx.manifest.jvm_args {
        for s in arg_values(arg) {
            let mut substituted = substitute(s, ctx);
            if substituted.starts_with("-DignoreList=") {
                classpath::remove_from_ignore_list(&mut substituted, loader_client_name);
            }
            cmd.arg(substituted);
        }
    }
}

pub fn push_game_args(
    cmd: &mut Command,
    ctx: &Substitution<'_>,
    connect: Option<ServerConnect>,
    fullscreen: bool,
) {
    if ctx.manifest.game_args.is_empty() {
        for (key, value) in default_game_args(ctx) {
            cmd.arg(key).arg(value);
        }
    } else {
        for arg in &ctx.manifest.game_args {
            for s in arg_values(arg) {
                cmd.arg(substitute(s, ctx));
            }
        }
    }

    if let Some(server) = connect {
        if supports_quick_play(&ctx.manifest.mc_version) {
            cmd.arg("--quickPlayMultiplayer")
                .arg(format!("{}:{}", server.host, server.port));
        } else {
            // Before 1.20 the client ignores quick play and silently opens
            // the main menu instead of joining.
            cmd.arg("--server")
                .arg(&server.host)
                .arg("--port")
                .arg(server.port.to_string());
        }
    }

    if fullscreen {
        cmd.arg("--fullscreen");
    }
}

/// `--quickPlayMultiplayer` arrived in 1.20 (23w14a). Snapshot ids and
/// anything unparsable are taken as new.
pub fn supports_quick_play(mc_version: &str) -> bool {
    let mut parts = mc_version.split('.');
    match (
        parts.next(),
        parts.next().and_then(|m| m.parse::<u32>().ok()),
    ) {
        (Some("1"), Some(minor)) => minor >= 20,
        _ => true,
    }
}

fn lookup(ctx: &Substitution<'_>, key: &str) -> String {
    ctx.table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

fn substitute(arg: &str, ctx: &Substitution<'_>) -> String {
    if !arg.contains("${") {
        return arg.to_string();
    }
    let mut out = arg.to_string();
    for (key, value) in &ctx.table {
        if out.contains(key) {
            out = out.replace(key, value);
        }
    }
    out
}

fn table(
    instance_dir: &Path,
    natives_dir: &Path,
    classpath: &str,
    manifest: &BuildManifest,
    login: &LoginInfo,
    primary_game_artifact: &str,
) -> Vec<(&'static str, String)> {
    let assets_root = instance_dir.join("assets");
    let library_dir = instance_dir.join("libraries");
    let game_jar = manifest
        .verified_files
        .iter()
        .find(|f| manifest.kind_of(&f.path) == ArtifactKind::ClientJar)
        .and_then(|f| safe_join(instance_dir, &f.path))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    vec![
        ("${auth_player_name}", login.username.clone()),
        ("${version_name}", manifest.version.clone()),
        (
            "${game_directory}",
            instance_dir.to_string_lossy().into_owned(),
        ),
        ("${assets_root}", assets_root.to_string_lossy().into_owned()),
        ("${game_assets}", assets_root.to_string_lossy().into_owned()),
        ("${assets_index_name}", manifest.assets_index_name.clone()),
        ("${auth_uuid}", login.uuid.clone()),
        ("${auth_access_token}", login.access_token.clone()),
        ("${auth_session}", login.access_token.clone()),
        ("${clientid}", String::new()),
        ("${auth_xuid}", String::new()),
        ("${user_type}", "msa".to_string()),
        ("${user_properties}", "{}".to_string()),
        ("${version_type}", "release".to_string()),
        (
            "${natives_directory}",
            natives_dir.to_string_lossy().into_owned(),
        ),
        ("${launcher_name}", "noro".to_string()),
        ("${launcher_version}", env!("CARGO_PKG_VERSION").to_string()),
        ("${classpath}", classpath.to_string()),
        ("${game_jar}", game_jar),
        (
            "${primary_game_artifact}",
            primary_game_artifact.to_string(),
        ),
        (
            "${library_directory}",
            library_dir.to_string_lossy().into_owned(),
        ),
        (
            "${classpath_separator}",
            classpath::classpath_separator().to_string(),
        ),
    ]
}

/// Splits the player's JVM flags the way a shell would for the simple cases:
/// whitespace separates, and single or double quotes keep a value with spaces
/// in one argument (`-XX:OnOutOfMemoryError="kill -9 %p"`). The quotes
/// themselves are dropped.
pub fn split_flags(flags: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for c in flags.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => current.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                started = true;
            }
            (None, c) if c.is_whitespace() => {
                if started {
                    out.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            (None, c) => {
                current.push(c);
                started = true;
            }
        }
    }
    if started {
        out.push(current);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_flags_stay_one_argument() {
        assert_eq!(
            split_flags(r#"-Xss2m -XX:OnOutOfMemoryError="kill -9 %p"  -Dname='a b'"#),
            vec![
                "-Xss2m".to_string(),
                "-XX:OnOutOfMemoryError=kill -9 %p".to_string(),
                "-Dname=a b".to_string(),
            ]
        );
        assert!(split_flags("   ").is_empty());
        assert_eq!(split_flags("-Da=\"\""), vec!["-Da=".to_string()]);
    }

    #[test]
    fn quick_play_only_where_the_client_knows_it() {
        assert!(supports_quick_play("1.20.1"));
        assert!(supports_quick_play("1.21"));
        assert!(supports_quick_play("23w14a"));
        assert!(!supports_quick_play("1.19.4"));
        assert!(!supports_quick_play("1.12.2"));
        assert!(!supports_quick_play("1.7.10"));
    }
}
