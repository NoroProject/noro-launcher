//! The rules here are copied out of real Mojang manifests rather than invented,
//! so what's being tested is that an actual build launches.

use super::arg_values;
use schema::ManifestArg;

fn parse(json: &str) -> ManifestArg {
    serde_json::from_str(json).expect("manifest argument")
}

/// Conditional JVM arguments from 1.21.1, verbatim.
const OSX_ONLY: &str =
    r#"{"rules":[{"action":"allow","os":{"name":"osx"}}],"value":["-XstartOnFirstThread"]}"#;
const WINDOWS_ONLY: &str = r#"{"rules":[{"action":"allow","os":{"name":"windows"}}],"value":"-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"}"#;
const X86_ONLY: &str = r#"{"rules":[{"action":"allow","os":{"arch":"x86"}}],"value":"-Xss1M"}"#;

#[test]
fn plain_string_passes_through() {
    let arg = parse(r#""-cp""#);
    assert_eq!(arg_values(&arg), ["-cp"]);
}

/// The argument this whole module exists for: required on macOS, and the JVM
/// refuses to start with it anywhere else.
#[test]
fn start_on_first_thread_only_on_macos() {
    let arg = parse(OSX_ONLY);
    let got = arg_values(&arg);
    if cfg!(target_os = "macos") {
        assert_eq!(got, ["-XstartOnFirstThread"]);
    } else {
        assert!(got.is_empty(), "not macOS, but the argument got through: {got:?}");
    }
}

#[test]
fn windows_heap_dump_only_on_windows() {
    let arg = parse(WINDOWS_ONLY);
    assert_eq!(arg_values(&arg).is_empty(), !cfg!(target_os = "windows"));
}

/// `x86` means 32-bit to Mojang, and no platform the launcher ships on is, so
/// `-Xss1M` should never appear.
#[test]
fn xss_never_matches_64bit() {
    let arg = parse(X86_ONLY);
    assert!(arg_values(&arg).is_empty());
}

/// Feature rules never match, or the game would be handed a `--width` with an
/// unsubstituted placeholder after it.
#[test]
fn feature_args_are_dropped() {
    let demo =
        r#"{"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"}"#;
    let resolution = r#"{"rules":[{"action":"allow","features":{"has_custom_resolution":true}}],"value":["--width","${resolution_width}","--height","${resolution_height}"]}"#;
    let (demo, resolution) = (parse(demo), parse(resolution));
    assert!(arg_values(&demo).is_empty());
    assert!(arg_values(&resolution).is_empty());
}

/// The shape Forge and older versions use: `allow` for everyone, `disallow` for
/// one OS. Take the first match instead of the last and it reads as "nobody".
#[test]
fn last_matching_rule_wins() {
    let all_but_osx = r#"{"rules":[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}],"value":"-Dfoo"}"#;
    let arg = parse(all_but_osx);
    assert_eq!(arg_values(&arg).is_empty(), cfg!(target_os = "macos"));
}

/// `version` shows up in the 1.16–1.19 manifests. Off Windows the rule is
/// already ruled out by `os.name`, so the argument never survives either way.
#[test]
fn os_version_is_honoured() {
    let win10 = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\."}}],"value":["-Dos.name=Windows 10","-Dos.version=10.0"]}"#;
    let arg = parse(win10);
    if !cfg!(target_os = "windows") {
        assert!(arg_values(&arg).is_empty());
    }
    // A regex that won't compile must not take the launch down with it.
    let broken = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\.("}}],"value":"-Dbroken"}"#;
    let broken = parse(broken);
    assert!(arg_values(&broken).is_empty());
}

/// The master moves rules through these same structs on their way to the
/// database, so a field we drop on deserialize is a field lost for good.
#[test]
fn unknown_shape_survives_round_trip() {
    let src = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\."}}],"value":["-Da","-Db"]}"#;
    let arg: ManifestArg = serde_json::from_str(src).expect("argument");
    let back = serde_json::to_string(&arg).expect("serialize");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&back).unwrap(),
        serde_json::from_str::<serde_json::Value>(src).unwrap()
    );
}
