//! Moving JVM arguments into an `@argfile` when the command line gets too long.
//!
//! Windows caps a command line at 32 767 characters. A big modpack installed
//! under a long profile path gets there on the classpath alone, and the JVM
//! then never starts — `CreateProcess` refuses with a bare "file name too
//! long". Java 9 and later read arguments from a file named with `@`; Java 8
//! doesn't, so older builds keep the plain command line.

use std::ffi::OsString;
use std::path::Path;
use tokio::process::Command;

/// Below `CreateProcess`'s limit with room for the executable path and the
/// quotes Windows adds around arguments with spaces.
const LIMIT: usize = 30_000;

pub fn command_line_len(cmd: &Command) -> usize {
    let std = cmd.as_std();
    std.get_program().len()
        + std
            .get_args()
            .map(|a| a.len() + 3) // a space and possible quotes
            .sum::<usize>()
}

pub fn too_long(len: usize) -> bool {
    cfg!(windows) && len > LIMIT
}

/// Major version of the runtime `java` belongs to, from the `release` file
/// every JDK and JRE ships next to `bin/`. `None` when it can't be told.
pub fn java_major(java: &Path) -> Option<u32> {
    let home = java.parent()?.parent()?;
    let release = std::fs::read_to_string(home.join("release")).ok()?;
    let line = release
        .lines()
        .find_map(|l| l.strip_prefix("JAVA_VERSION="))?;
    parse_major(line.trim().trim_matches('"'))
}

fn parse_major(version: &str) -> Option<u32> {
    let mut parts = version.split(['.', '_', '-', '+']);
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        // "1.8.0_402" is Java 8.
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

/// One argument as the JVM's argfile parser reads it: quoted, with
/// backslashes and quotes escaped, since inside quotes a backslash escapes.
fn quote(arg: &str) -> String {
    let mut out = String::with_capacity(arg.len() + 2);
    out.push('"');
    for c in arg.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn render(args: &[OsString]) -> String {
    args.iter()
        .map(|a| quote(&a.to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A fresh command for the same program, with the first `jvm_count`
/// arguments in `file` and `@file` in their place. Working directory,
/// environment and the remaining arguments (main class, game arguments)
/// carry over unchanged.
pub async fn split_into_file(
    cmd: &Command,
    jvm_count: usize,
    file: &Path,
) -> std::io::Result<Command> {
    let std = cmd.as_std();
    let args: Vec<OsString> = std.get_args().map(|a| a.to_owned()).collect();
    let (jvm, rest) = args.split_at(jvm_count.min(args.len()));
    crate::fsutil::write_atomic(file, render(jvm)).await?;

    let mut out = Command::new(std.get_program());
    if let Some(dir) = std.get_current_dir() {
        out.current_dir(dir);
    }
    for (key, value) in std.get_envs() {
        match value {
            Some(v) => out.env(key, v),
            None => out.env_remove(key),
        };
    }
    let mut at = OsString::from("@");
    at.push(file.as_os_str());
    out.arg(at);
    out.args(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_both_version_schemes() {
        assert_eq!(parse_major("1.8.0_402"), Some(8));
        assert_eq!(parse_major("17.0.10"), Some(17));
        assert_eq!(parse_major("21"), Some(21));
        assert_eq!(parse_major("garbage"), None);
    }

    #[test]
    fn quotes_paths_the_way_the_jvm_reads_them() {
        let args = [
            OsString::from("-cp"),
            OsString::from(r"C:\Users\Jo Doe\a.jar;C:\b.jar"),
            OsString::from(r#"-Dx="y""#),
        ];
        assert_eq!(
            render(&args),
            "\"-cp\"\n\"C:\\\\Users\\\\Jo Doe\\\\a.jar;C:\\\\b.jar\"\n\"-Dx=\\\"y\\\"\""
        );
    }

    #[tokio::test]
    async fn keeps_directory_environment_and_game_arguments() {
        let dir = crate::test_http::TempDir::new("argfile");
        let mut cmd = Command::new("java");
        cmd.current_dir(dir.path()).env("A", "1").args([
            "-Xmx2g",
            "-cp",
            "a.jar",
            "Main",
            "--username",
            "x",
        ]);
        let file = dir.path().join("args.txt");
        let moved = split_into_file(&cmd, 3, &file).await.unwrap();
        let std = moved.as_std();
        assert_eq!(std.get_current_dir(), Some(dir.path()));
        assert_eq!(std.get_envs().count(), 1);
        let args: Vec<_> = std.get_args().map(|a| a.to_string_lossy()).collect();
        assert!(args[0].starts_with('@'));
        assert_eq!(&args[1..], ["Main", "--username", "x"]);
        let written = tokio::fs::read_to_string(&file).await.unwrap();
        assert_eq!(written, "\"-Xmx2g\"\n\"-cp\"\n\"a.jar\"");
    }

    #[test]
    fn version_comes_from_the_release_file() {
        let dir = crate::test_http::TempDir::new("argfile-java");
        std::fs::create_dir_all(dir.path().join("bin")).unwrap();
        std::fs::write(
            dir.path().join("release"),
            "IMPLEMENTOR=\"x\"\nJAVA_VERSION=\"17.0.2\"\n",
        )
        .unwrap();
        assert_eq!(java_major(&dir.path().join("bin").join("java")), Some(17));
        assert_eq!(
            java_major(&dir.path().join("a").join("b").join("java")),
            None
        );
    }
}
