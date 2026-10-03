use super::live::{live, swap_in, Applied};

/// The list is deliberately closed: a world breaks under a running game,
/// `options.txt` is rewritten on exit, and the JVM holds the jars.
#[test]
fn only_packs_and_shaders_are_live() {
    assert!(live("resourcepacks/noro-prefixes.zip"));
    assert!(live("shaderpacks/complementary.zip"));

    assert!(!live("saves/world/level.dat"));
    assert!(!live("options.txt"));
    assert!(!live("mods/jei.jar"));
    assert!(!live("config/jei.toml"));
}

/// A downloaded file replaces the old one in a single rename, so an
/// interruption doesn't leave half a pack behind, and no staging file stays.
#[tokio::test]
async fn swaps_a_staged_file_in() {
    let dir = std::env::temp_dir().join(format!("noro-live-{}", uuid::Uuid::new_v4()));
    let path = dir.join("resourcepacks/pack.zip");
    let staged = dir.join("resourcepacks/pack.zip.noro-live");
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&path, b"first").await.unwrap();
    tokio::fs::write(&staged, b"second").await.unwrap();

    assert!(swap_in(&staged, &path).await.unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"second".to_vec());

    let mut entries = tokio::fs::read_dir(path.parent().unwrap()).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        assert_eq!(entry.file_name(), "pack.zip");
    }
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[test]
fn a_folder_pack_is_one_pack() {
    use super::live::pack_name;
    assert_eq!(pack_name("resourcepacks/noro.zip"), Some("noro.zip"));
    assert_eq!(
        pack_name("resourcepacks/Faithful/pack.mcmeta"),
        Some("Faithful")
    );
    assert_eq!(pack_name("shaderpacks/bsl.zip"), None);
    assert_eq!(pack_name("resourcepacks/"), None);
}

#[tokio::test]
async fn a_pack_is_switched_on_only_once() {
    use super::live::enable_once;
    let dir = crate::test_http::TempDir::new("packs");
    tokio::fs::write(
        dir.path().join("options.txt"),
        "resourcePacks:[\"vanilla\"]\n",
    )
    .await
    .unwrap();

    assert!(enable_once(dir.path(), "a.zip").await.unwrap());
    // The player turns it off in game.
    tokio::fs::write(
        dir.path().join("options.txt"),
        "resourcePacks:[\"vanilla\"]\n",
    )
    .await
    .unwrap();
    assert!(!enable_once(dir.path(), "a.zip").await.unwrap());

    let options = tokio::fs::read_to_string(dir.path().join("options.txt"))
        .await
        .unwrap();
    assert!(!options.contains("a.zip"), "their choice stands: {options}");
}

/// An empty result is what decides whether the player gets told anything.
#[test]
fn tells_an_empty_result_from_a_real_one() {
    assert!(Applied::default().nothing());
    assert!(!Applied {
        updated: vec!["resourcepacks/pack.zip".into()],
        locked: Vec::new(),
    }
    .nothing());
}

#[test]
fn adds_the_pack_to_the_enabled_list() {
    let was = "fov:70\nresourcePacks:[\"vanilla\",\"mod_resources\"]\nlang:ru_ru\n";
    let now = super::live::add_pack(was, "\"file/noro-prefixes.zip\"").unwrap();

    assert!(
        now.contains("resourcePacks:[\"vanilla\",\"mod_resources\",\"file/noro-prefixes.zip\"]")
    );
    assert!(now.contains("fov:70"), "other settings left alone");
    assert!(now.contains("lang:ru_ru"));
}

#[test]
fn leaves_an_already_enabled_pack_alone() {
    let was = "resourcePacks:[\"file/noro-prefixes.zip\"]\n";
    assert!(super::live::add_pack(was, "\"file/noro-prefixes.zip\"").is_none());
}

#[test]
fn fills_an_empty_list() {
    let now = super::live::add_pack("resourcePacks:[]\n", "\"file/pack.zip\"").unwrap();
    assert!(now.contains("resourcePacks:[\"file/pack.zip\"]"), "{now}");
}

mod apply {
    use super::super::live::apply;
    use crate::sync::verify::fixtures::{entry, manifest, optional, player};
    use crate::test_http::{serve, sha1_hex, TempDir};
    use schema::ArtifactKind;

    fn pack_manifest(url: &str, sha1: &str) -> schema::BuildManifest {
        let (mut on, k1) = entry("resourcepacks/on.zip", sha1, ArtifactKind::Other);
        on.url = url.to_string();
        let (mut off, k2) = entry("resourcepacks/off.zip", sha1, ArtifactKind::Other);
        off.url = url.to_string();
        let mut m = manifest(vec![(on, k1), (off, k2)]);
        m.optional_mods = vec![optional("Off pack", false, &["resourcepacks/off.zip"])];
        m
    }

    #[tokio::test]
    async fn an_instance_that_was_never_installed_is_left_alone() {
        let body = b"pack".to_vec();
        let sha = sha1_hex(&body);
        let server = serve(vec![("/p", 200, body)]).await;
        let dir = TempDir::new("live-fresh");

        let done = apply(
            &reqwest::Client::new(),
            dir.path(),
            &pack_manifest(&server.url("/p"), &sha),
            &[],
            &player(),
        )
        .await
        .unwrap();

        assert!(done.nothing());
        assert_eq!(
            server.hits(),
            0,
            "looking at a server must not download its packs"
        );
        assert!(!dir.path().join("resourcepacks").exists());
    }

    #[tokio::test]
    async fn only_packs_the_full_sync_would_install_arrive() {
        let body = b"pack".to_vec();
        let sha = sha1_hex(&body);
        let server = serve(vec![("/p", 200, body.clone())]).await;
        let dir = TempDir::new("live-installed");
        tokio::fs::write(dir.path().join(".noro-build"), "1\nx")
            .await
            .unwrap();

        let done = apply(
            &reqwest::Client::new(),
            dir.path(),
            &pack_manifest(&server.url("/p"), &sha),
            &[],
            &player(),
        )
        .await
        .unwrap();

        assert_eq!(done.updated, vec!["resourcepacks/on.zip".to_string()]);
        assert_eq!(
            tokio::fs::read(dir.path().join("resourcepacks/on.zip"))
                .await
                .unwrap(),
            body
        );
        assert!(
            !dir.path().join("resourcepacks/off.zip").exists(),
            "a pack of a disabled optional mod stays out"
        );
    }
}
