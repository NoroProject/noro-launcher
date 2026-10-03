use super::*;

const GTNH_FLAGS: &str =
    "-Djava.system.class.loader=com.gtnewhorizons.retrofuturabootstrap.RfbSystemClassLoader \
                          --add-opens java.base/jdk.internal.loader=ALL-UNNAMED";

fn gtnh() -> RecommendedClientSettings {
    RecommendedClientSettings {
        memory_min_mb: 4096,
        memory_max_mb: 6144,
        jvm_flags: GTNH_FLAGS.into(),
        show_console_on_launch: false,
        fullscreen: false,
    }
}

/// The shape a config is in once the player has touched any setting of a
/// server: an override, its flags empty. GTNH crashed on it with "System
/// classloader not overwritten" — the override replaced the build's flags.
fn with_override(server: Uuid, flags: &str) -> LauncherConfig {
    let mut config = LauncherConfig::default();
    config.set_server_memory(server, 2048, 4096);
    config.set_server_jvm_flags(server, flags.into());
    config
}

#[test]
fn an_override_keeps_the_builds_flags() {
    let server = Uuid::new_v4();
    let launch = with_override(server, "").launch_config_for_server(&server, &gtnh());
    assert!(launch.jvm_flags.contains("RfbSystemClassLoader"));
    assert_eq!(launch.memory_max_mb, 4096);
}

#[test]
fn the_players_flags_go_before_the_builds() {
    let server = Uuid::new_v4();
    let config = with_override(server, "-XX:+UseZGC\n-Djava.system.class.loader=mine");
    let flags = config.launch_config_for_server(&server, &gtnh()).jvm_flags;
    assert!(flags.starts_with("-XX:+UseZGC -Djava.system.class.loader=mine "));
    assert!(flags.ends_with("ALL-UNNAMED"));
}

#[test]
fn without_an_override_the_build_recommends_and_the_global_flags_apply() {
    let server = Uuid::new_v4();
    let config = LauncherConfig {
        jvm_flags: "-XX:+UseG1GC".into(),
        ..LauncherConfig::default()
    };
    let launch = config.launch_config_for_server(&server, &gtnh());
    assert_eq!(launch.memory_max_mb, 6144);
    assert!(launch.jvm_flags.starts_with("-XX:+UseG1GC "));
    assert!(launch.jvm_flags.contains("RfbSystemClassLoader"));
}

/// Written by 2.0.8: the override's `jvm_flags` was never the build's there
/// (nothing in that version could put them in), so it reads as the player's.
#[test]
fn an_old_config_reads_as_the_players_flags() {
    let server = Uuid::new_v4();
    let raw = format!(
        r#"{{"master_url":"http://127.0.0.1:8080","memory_min_mb":2048,"memory_max_mb":4096,
            "jvm_flags":"","show_console_on_launch":true,
            "server_settings":{{"{server}":{{"memory_min_mb":2048,"memory_max_mb":4096,
            "jvm_flags":"-XX:+UseZGC","show_console_on_launch":true,"fullscreen":false}}}}}}"#
    );
    let config: LauncherConfig = serde_json::from_str(&raw).unwrap();
    let flags = config.launch_config_for_server(&server, &gtnh()).jvm_flags;
    assert_eq!(flags, format!("-XX:+UseZGC {GTNH_FLAGS}"));
}

#[test]
fn a_config_missing_fields_keeps_what_it_has() {
    // An older or newer launcher may not know every field. Before, one missing
    // field failed the whole parse and the player's settings were reset.
    let json = r#"{"master_url":"https://example.com","memory_min_mb":1024,"memory_max_mb":3072,
        "server_settings":{"00000000-0000-0000-0000-000000000001":{"memory_min_mb":512}}}"#;
    let config: LauncherConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.memory_max_mb, 3072);
    assert!(config.jvm_flags.is_empty());
    let server = config
        .server_settings
        .values()
        .next()
        .expect("server override survives");
    assert_eq!(server.memory_min_mb, 512);
    assert_eq!(server.memory_max_mb, 4096);
}
