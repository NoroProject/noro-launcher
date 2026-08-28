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
        available_builds: Vec::new(),
        id: Uuid::new_v4(),
        name: "MauMods".into(),
        description: String::new(),
        icon_url: None,
        background_url: None,
        // Matches the master: no address at all when there are no game servers.
        mc_host: nodes.first().map(|n| n.mc_host.clone()),
        mc_port: nodes.first().map(|n| n.mc_port),
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
    // Nothing changed on the master, so the second pass leaves the file alone.
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
    assert!(
        got.contains(&("Creative".into(), "s2.noro.dev:25566".into())),
        "{got:?}"
    );
}

#[test]
fn keeps_player_entries_and_drops_removed_ones() {
    let dir = tempdir();
    sync(
        &dir,
        &build(vec![node("Old", "old.noro.dev", 25565, false)]),
    )
    .unwrap();

    // The player adds a server of their own.
    let mut dat: ServersDat =
        fastnbt::from_bytes(&std::fs::read(dir.join("servers.dat")).unwrap()).unwrap();
    dat.servers.push(record("Friend", "friend.example"));
    std::fs::write(dir.join("servers.dat"), fastnbt::to_bytes(&dat).unwrap()).unwrap();

    sync(
        &dir,
        &build(vec![node("New", "new.noro.dev", 25566, false)]),
    )
    .unwrap();

    let got = names(&dir);
    assert!(got.contains(&("New".into(), "new.noro.dev:25566".into())));
    assert!(
        got.contains(&("Friend".into(), "friend.example".into())),
        "{got:?}"
    );
    assert!(
        !got.iter().any(|(_, ip)| ip == "old.noro.dev"),
        "a server dropped from the build should go: {got:?}"
    );
}

/// The stamp answers "did the master's list change", not "is the file on disk".
#[test]
fn a_missing_file_is_rebuilt_even_when_the_stamp_matches() {
    let dir = tempdir();
    let server = build(vec![node("Main", "create.example.dev", 25565, false)]);

    assert!(sync(&dir, &server).unwrap(), "first run writes the file");
    assert!(dir.join("servers.dat").exists());

    assert!(
        !sync(&dir, &server).unwrap(),
        "same list, nothing to rewrite"
    );

    std::fs::remove_file(dir.join("servers.dat")).unwrap();

    assert!(sync(&dir, &server).unwrap(), "missing file has to come back");
    assert!(
        names(&dir).contains(&("Main".into(), "create.example.dev".into())),
        "the server should be back in the list: {:?}",
        names(&dir)
    );
}

// --- Xaero waypoints ----------------------------------------------------------
//
// Xaero keys its waypoints on `XaeroWaypoints/Multiplayer_<entry>/`, where the
// entry comes from the server list. So "the map lost all my waypoints" reduces
// to one question these tests answer: does our entry change between launches?

/// What Xaero uses to tell "the same server" from a new one.
fn identity(dir: &Path) -> Vec<(String, String)> {
    names(dir)
}

#[test]
fn the_server_entry_is_identical_across_restarts() {
    let dir = tempdir();
    let server = build(vec![node("Main", "play.noro.dev", 25565, false)]);

    sync(&dir, &server).unwrap();
    let first = identity(&dir);

    for _ in 0..5 {
        assert!(!sync(&dir, &server).unwrap());
    }
    assert_eq!(identity(&dir), first);
}

#[test]
fn reordered_nodes_from_the_database_do_not_rewrite_the_file() {
    // The fingerprint is computed over a sorted list, so the order the master
    // happens to return nodes in doesn't matter.
    let dir = tempdir();
    let a = node("A", "a.noro.dev", 25565, false);
    let b = node("B", "b.noro.dev", 25565, false);

    sync(&dir, &build(vec![a.clone(), b.clone()])).unwrap();
    assert!(!sync(&dir, &build(vec![b, a])).unwrap());
}

#[test]
fn renaming_the_server_does_change_the_entry() {
    // The other half of the answer: renaming a server in the admin panel really
    // does change the entry, and Xaero will start a fresh waypoint directory.
    // That's the one case where waypoints go missing because of us.
    let dir = tempdir();
    sync(
        &dir,
        &build(vec![node("Main", "play.noro.dev", 25565, false)]),
    )
    .unwrap();

    assert!(sync(
        &dir,
        &build(vec![node("Main Server", "play.noro.dev", 25565, false)])
    )
    .unwrap());
    assert_eq!(
        identity(&dir),
        vec![("Main Server".into(), "play.noro.dev".into())]
    );
}

#[test]
fn xaero_directories_are_protected_from_the_sync() {
    // Rewriting servers.dat must not take the map's own files with it. The
    // patterns come from the build; this checks the paths Xaero actually uses
    // match them.
    let protected: Vec<String> = ["xaero*", "config/xaero*", "xaerominimap*", "xaeroworldmap*"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    for path in [
        "XaeroWaypoints/Multiplayer_play.noro.dev/waypoints.txt",
        "xaero/minimap.json",
        "config/xaerominimap.txt",
        "XaeroWorldMap/Multiplayer_play.noro.dev/region.zip",
    ] {
        assert!(
            crate::sync::file_sync::is_protected(path, &protected),
            "{path} should be protected from deletion"
        );
    }
}
