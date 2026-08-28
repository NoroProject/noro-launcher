use super::live::{live, replace, Applied};

/// Живыми считаются только паки и шейдеры.
///
/// Список закрытый намеренно: мир под работающей игрой рвётся, `options.txt`
/// перезаписывается при выходе, а jar'ы держит JVM.
#[test]
fn only_packs_and_shaders_are_live() {
    assert!(live("resourcepacks/noro-prefixes.zip"));
    assert!(live("shaderpacks/complementary.zip"));

    assert!(!live("saves/world/level.dat"));
    assert!(!live("options.txt"));
    assert!(!live("mods/jei.jar"));
    assert!(!live("config/jei.toml"));
}

/// Замена идёт через временный файл: обрыв не оставляет половину пака.
#[tokio::test]
async fn replaces_through_a_temporary_file() {
    let dir = std::env::temp_dir().join(format!("noro-live-{}", uuid::Uuid::new_v4()));
    let path = dir.join("resourcepacks/pack.zip");

    assert!(replace(&path, b"first").await.unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"first".to_vec());

    assert!(replace(&path, b"second").await.unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"second".to_vec());

    // Временных файлов после себя не оставляем.
    let mut entries = tokio::fs::read_dir(path.parent().unwrap()).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        assert_eq!(entry.file_name(), "pack.zip");
    }
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

/// Пустой итог отличается от непустого: по нему решают, говорить ли игроку.
#[test]
fn tells_an_empty_result_from_a_real_one() {
    assert!(Applied::default().nothing());
    assert!(!Applied {
        updated: vec!["resourcepacks/pack.zip".into()],
        locked: Vec::new(),
    }
    .nothing());
}

/// Пак дописывается в конец списка, остальные не трогаются.
#[test]
fn adds_the_pack_to_the_enabled_list() {
    let was = "fov:70\nresourcePacks:[\"vanilla\",\"mod_resources\"]\nlang:ru_ru\n";
    let now = super::live::add_pack(was, "\"file/noro-prefixes.zip\"").unwrap();

    assert!(
        now.contains("resourcePacks:[\"vanilla\",\"mod_resources\",\"file/noro-prefixes.zip\"]")
    );
    assert!(now.contains("fov:70"), "чужие настройки на месте");
    assert!(now.contains("lang:ru_ru"));
}

/// Уже включён — не дописываем второй раз.
#[test]
fn leaves_an_already_enabled_pack_alone() {
    let was = "resourcePacks:[\"file/noro-prefixes.zip\"]\n";
    assert!(super::live::add_pack(was, "\"file/noro-prefixes.zip\"").is_none());
}

/// Пустой список — пак становится единственным.
#[test]
fn fills_an_empty_list() {
    let now = super::live::add_pack("resourcePacks:[]\n", "\"file/pack.zip\"").unwrap();
    assert!(now.contains("resourcePacks:[\"file/pack.zip\"]"), "{now}");
}
