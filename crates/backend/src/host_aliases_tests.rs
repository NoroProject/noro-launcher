use super::*;
use schema::{GameServerEntry, Modloader};
use uuid::Uuid;

fn node(host: &str, port: u16, alias: Option<&str>) -> GameServerEntry {
    GameServerEntry {
        id: Uuid::new_v4(),
        name: "Node".into(),
        mc_host: host.into(),
        mc_port: port,
        online: 0,
        max_online: 0,
        live: true,
        proxy: false,
        proxy_id: None,
        proxy_role: None,
        is_default: false,
        alias: alias.map(str::to_string),
    }
}

fn build(nodes: Vec<GameServerEntry>) -> ServerEntry {
    ServerEntry {
        id: Uuid::new_v4(),
        name: "Build".into(),
        description: String::new(),
        icon_url: None,
        background_url: None,
        mc_host: None,
        mc_port: None,
        mc_alias: None,
        modloader: Modloader::Vanilla,
        mc_version: "1.21.1".into(),
        current_build_id: None,
        current_version: None,
        limited: false,
        sort_order: 0,
        game_servers: nodes,
        available_builds: Vec::new(),
        online: None,
        max_online: None,
        my_playtime_minutes: None,
    }
}

#[test]
fn every_build_contributes_and_disabled_or_broken_aliases_are_skipped() {
    let servers = vec![
        build(vec![
            node("203.0.113.7", 25565, Some("aoa.example")),
            node("10.0.0.2", 25566, None),
        ]),
        build(vec![
            node("play.example.net", 25577, Some("sky.example")),
            node("10.0.0.3", 25565, Some("bad alias")),
            node("10.0.0.4", 25565, Some("aoa.example")),
        ]),
    ];
    let aliases = HostAliases::from_servers(&servers);
    assert_eq!(
        aliases.table(),
        "aoa.example 203.0.113.7 25565\nsky.example play.example.net 25577\n"
    );
    assert_eq!(
        aliases.alias_of("PLAY.example.net", 25577),
        Some("sky.example")
    );
    assert_eq!(aliases.alias_of("play.example.net", 25565), None);
    assert_eq!(aliases.alias_of("10.0.0.2", 25566), None);
}

#[test]
fn only_a_jar_that_says_so_resolves_aliases() {
    let dir = std::env::temp_dir().join(format!("noro-aliases-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let jar = |name: &str, manifest: &str| {
        let path = dir.join(name);
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        zip.start_file(
            "META-INF/MANIFEST.MF",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        std::io::Write::write_all(&mut zip, manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
        path
    };
    let fork = jar(
        "fork.jar",
        "Manifest-Version: 1.0\r\nNoro-Host-Aliases: 1\r\n",
    );
    let upstream = jar("upstream.jar", "Manifest-Version: 1.0\r\n");
    assert!(agent_resolves_aliases(&fork));
    assert!(!agent_resolves_aliases(&upstream));
    assert!(!agent_resolves_aliases(&dir.join("missing.jar")));
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn the_table_lands_where_the_agent_is_told_to_look() {
    let dir = std::env::temp_dir().join(format!("noro-aliases-{}", Uuid::new_v4()));
    let aliases =
        HostAliases::from_servers(&[build(vec![node("203.0.113.7", 25565, Some("aoa.example"))])]);
    let path = aliases.write(&dir).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "aoa.example 203.0.113.7 25565\n"
    );
    std::fs::remove_dir_all(&dir).ok();
}
