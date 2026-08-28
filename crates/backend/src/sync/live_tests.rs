use super::live::{live, replace, Applied};

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

/// Replacement goes through a temp file, so an interruption doesn't leave half
/// a pack behind.
#[tokio::test]
async fn replaces_through_a_temporary_file() {
    let dir = std::env::temp_dir().join(format!("noro-live-{}", uuid::Uuid::new_v4()));
    let path = dir.join("resourcepacks/pack.zip");

    assert!(replace(&path, b"first").await.unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"first".to_vec());

    assert!(replace(&path, b"second").await.unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"second".to_vec());

    // No temp files left over.
    let mut entries = tokio::fs::read_dir(path.parent().unwrap()).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        assert_eq!(entry.file_name(), "pack.zip");
    }
    let _ = tokio::fs::remove_dir_all(&dir).await;
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
