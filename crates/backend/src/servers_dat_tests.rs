use super::*;
use schema::{GameServerEntry, Modloader};
use uuid::Uuid;

fn node(name: &str, host: &str, port: u16, proxy: bool) -> GameServerEntry {
    GameServerEntry {
        id: Uuid::new_v4(),
        name: name.into(),
        mc_host: host.into(),
        mc_port: port,
        online: 0,
        max_online: 20,
        live: true,
        proxy,
    }
}

fn build(nodes: Vec<GameServerEntry>) -> ServerEntry {
    ServerEntry {
        id: Uuid::new_v4(),
        name: "MauMods".into(),
        description: String::new(),
        icon_url: None,
        background_url: None,
        mc_host: nodes.first().map(|n| n.mc_host.clone()).unwrap_or_default(),
        mc_port: nodes.first().map(|n| n.mc_port).unwrap_or(25565),
        modloader: Modloader::NeoForge,
        mc_version: "1.21.1".into(),
        current_build_id: None,
        current_version: None,
        limited: false,
        sort_order: 0,
        game_servers: nodes,
        online: None,
        max_online: None,
    }
}

fn names(dir: &Path) -> Vec<(String, String)> {
    let bytes = std::fs::read(dir.join("servers.dat")).unwrap();
    let dat: ServersDat = fastnbt::from_bytes(&bytes).unwrap();
    dat.servers
        .iter()
        .map(|s| {
            (
                s.get("name").and_then(as_str).unwrap_or_default(),
                s.get("ip").and_then(as_str).unwrap_or_default(),
            )
        })
        .collect()
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("noro-servers-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn writes_game_servers_and_skips_unchanged() {
    let dir = tempdir();
    let server = build(vec![node("Main", "play.noro.dev", 25565, false)]);

    assert!(sync(&dir, &server).unwrap());
    assert_eq!(names(&dir), vec![("Main".into(), "play.noro.dev".into())]);
    // Второй проход без изменений на мастере файл не трогает.
    assert!(!sync(&dir, &server).unwrap());
}

#[test]
fn proxy_hides_backends_behind_it() {
    let dir = tempdir();
    let server = build(vec![
        node("Proxy", "play.noro.dev", 25565, true),
        node("Survival", "10.0.0.2", 25566, false),
        node("Creative", "10.0.0.3", 25567, false),
    ]);

    sync(&dir, &server).unwrap();

    // Внутренние бэкенды в списке игрока не место: прямой коннект обошёл бы прокси.
    assert_eq!(names(&dir), vec![("Proxy".into(), "play.noro.dev".into())]);
}

#[test]
fn without_proxy_every_node_is_listed() {
    let dir = tempdir();
    let server = build(vec![
        node("Survival", "s1.noro.dev", 25565, false),
        node("Creative", "s2.noro.dev", 25566, false),
    ]);

    sync(&dir, &server).unwrap();

    let got = names(&dir);
    assert!(got.contains(&("Survival".into(), "s1.noro.dev".into())));
    assert!(got.contains(&("Creative".into(), "s2.noro.dev:25566".into())), "{got:?}");
}

#[test]
fn keeps_player_entries_and_drops_removed_ones() {
    let dir = tempdir();
    sync(&dir, &build(vec![node("Old", "old.noro.dev", 25565, false)])).unwrap();

    // Игрок дописал свой сервер.
    let mut dat: ServersDat =
        fastnbt::from_bytes(&std::fs::read(dir.join("servers.dat")).unwrap()).unwrap();
    dat.servers.push(record("Друг", "friend.example"));
    std::fs::write(dir.join("servers.dat"), fastnbt::to_bytes(&dat).unwrap()).unwrap();

    sync(&dir, &build(vec![node("New", "new.noro.dev", 25566, false)])).unwrap();

    let got = names(&dir);
    assert!(got.contains(&("New".into(), "new.noro.dev:25566".into())));
    assert!(got.contains(&("Друг".into(), "friend.example".into())), "{got:?}");
    assert!(
        !got.iter().any(|(_, ip)| ip == "old.noro.dev"),
        "снятый со сборки сервер должен уйти: {got:?}"
    );
}
