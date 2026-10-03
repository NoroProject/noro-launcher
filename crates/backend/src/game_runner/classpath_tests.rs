use super::*;
use crate::sync::verify::fixtures::{entry, manifest, OK_SHA1};
use schema::ManifestArg;

fn forge(game_args: &[&str]) -> BuildManifest {
    let mut m = manifest(vec![
        entry(
            "versions/1.7.10/client.jar",
            OK_SHA1,
            ArtifactKind::ClientJar,
        ),
        entry(
            "libraries/net/minecraftforge/forge/1.7.10/forge-1.7.10-universal.jar",
            OK_SHA1,
            ArtifactKind::Library,
        ),
    ]);
    m.modloader = Modloader::Forge;
    m.game_args = game_args
        .iter()
        .map(|a| ManifestArg::String(a.to_string()))
        .collect();
    m
}

/// GTNH and every other Forge up to 1.12: without the client jar launchwrapper
/// has no Minecraft to hand to FML.
#[test]
fn launchwrapper_forge_gets_the_client_jar_last() {
    let m = forge(&["--tweakClass", "cpw.mods.fml.common.launcher.FMLTweaker"]);
    let cp = build_classpath(Path::new("/i"), &m);
    let entries: Vec<&str> = cp.split(classpath_separator()).collect();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].ends_with("universal.jar"));
    assert!(entries[1].ends_with("versions/1.7.10/client.jar"));
}

#[test]
fn modlauncher_forge_keeps_the_client_jar_off() {
    let m = forge(&["--launchTarget", "forgeclient"]);
    let cp = build_classpath(Path::new("/i"), &m);
    assert!(!cp.contains("client.jar"));
}
