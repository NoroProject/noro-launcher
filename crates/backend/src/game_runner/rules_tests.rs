//! Правила берутся из настоящих манифестов Mojang, а не выдуманные: смысл
//! проверки в том, что реальная сборка запустится, а не в том, что код
//! согласен сам с собой.

use super::arg_values;
use schema::ManifestArg;

fn parse(json: &str) -> ManifestArg {
    serde_json::from_str(json).expect("аргумент манифеста")
}

/// Условные jvm-аргументы 1.21.1 — дословно.
const OSX_ONLY: &str =
    r#"{"rules":[{"action":"allow","os":{"name":"osx"}}],"value":["-XstartOnFirstThread"]}"#;
const WINDOWS_ONLY: &str = r#"{"rules":[{"action":"allow","os":{"name":"windows"}}],"value":"-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"}"#;
const X86_ONLY: &str = r#"{"rules":[{"action":"allow","os":{"arch":"x86"}}],"value":"-Xss1M"}"#;

#[test]
fn plain_string_passes_through() {
    let arg = parse(r#""-cp""#);
    assert_eq!(arg_values(&arg), ["-cp"]);
}

/// Тот самый аргумент, из-за которого всё и делалось: на macOS он обязателен,
/// а на Linux и Windows JVM от него не стартует.
#[test]
fn start_on_first_thread_only_on_macos() {
    let arg = parse(OSX_ONLY);
    let got = arg_values(&arg);
    if cfg!(target_os = "macos") {
        assert_eq!(got, ["-XstartOnFirstThread"]);
    } else {
        assert!(got.is_empty(), "не macOS, а аргумент попал: {got:?}");
    }
}

#[test]
fn windows_heap_dump_only_on_windows() {
    let arg = parse(WINDOWS_ONLY);
    assert_eq!(arg_values(&arg).is_empty(), !cfg!(target_os = "windows"));
}

/// `x86` у Mojang значит 32 бита. Ни одна платформа лаунчера такой не бывает,
/// поэтому `-Xss1M` не должен появляться нигде.
#[test]
fn xss_never_matches_64bit() {
    let arg = parse(X86_ONLY);
    assert!(arg_values(&arg).is_empty());
}

/// Feature-правила (demo, свой размер окна, quick play) не выполняются никогда:
/// иначе игра получила бы `--width` без значения.
#[test]
fn feature_args_are_dropped() {
    let demo =
        r#"{"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"}"#;
    let resolution = r#"{"rules":[{"action":"allow","features":{"has_custom_resolution":true}}],"value":["--width","${resolution_width}","--height","${resolution_height}"]}"#;
    let (demo, resolution) = (parse(demo), parse(resolution));
    assert!(arg_values(&demo).is_empty());
    assert!(arg_values(&resolution).is_empty());
}

/// Форма из манифестов Forge и старых версий: `allow` всем, `disallow` одной
/// ОС. Решает последнее подошедшее правило, иначе вышло бы «запрещено всем».
#[test]
fn last_matching_rule_wins() {
    let all_but_osx = r#"{"rules":[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}],"value":"-Dfoo"}"#;
    let arg = parse(all_but_osx);
    assert_eq!(arg_values(&arg).is_empty(), cfg!(target_os = "macos"));
}

/// `version` есть в манифестах 1.16–1.19. На не-Windows правило отсекается уже
/// по `os.name`, так что аргумент не проходит ни там, ни там при чужой версии.
#[test]
fn os_version_is_honoured() {
    let win10 = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\."}}],"value":["-Dos.name=Windows 10","-Dos.version=10.0"]}"#;
    let arg = parse(win10);
    if !cfg!(target_os = "windows") {
        assert!(arg_values(&arg).is_empty());
    }
    // Битый regex не должен ронять запуск.
    let broken = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\.("}}],"value":"-Dbroken"}"#;
    let broken = parse(broken);
    assert!(arg_values(&broken).is_empty());
}

/// Поля, которых мы не понимаем, обязаны доживать до БД: мастер перекладывает
/// правила через эти же структуры, и потеря `version` была бы необратимой.
#[test]
fn unknown_shape_survives_round_trip() {
    let src = r#"{"rules":[{"action":"allow","os":{"name":"windows","version":"^10\\."}}],"value":["-Da","-Db"]}"#;
    let arg: ManifestArg = serde_json::from_str(src).expect("аргумент");
    let back = serde_json::to_string(&arg).expect("сериализация");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&back).unwrap(),
        serde_json::from_str::<serde_json::Value>(src).unwrap()
    );
}
