use super::*;
use crate::test_http::{serve, TempDir};

fn signed(bytes: &[u8]) -> String {
    use base64::Engine;
    use ed25519_dalek::Signer;
    // Tests build without a configured key, so the check uses the dev seed.
    let key = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
    base64::engine::general_purpose::STANDARD.encode(key.sign(bytes).to_bytes())
}

fn version(url: String, body: &[u8], signature: String) -> LauncherVersion {
    LauncherVersion {
        id: uuid::Uuid::nil(),
        version: "9.9.9".into(),
        platform: schema::current_platform().into(),
        url,
        sha256: sha256_hex(body),
        signature,
        is_current: true,
    }
}

/// One test rather than several: the install guard is process-wide, and
/// tests running in parallel would trip it for each other.
#[tokio::test]
async fn installs_only_what_verifies_and_leaves_the_bootstrapper_a_matching_signature() {
    let body = b"new launcher binary".to_vec();
    let server = serve(vec![("/core", 200, body.clone())]).await;
    let tmp = TempDir::new("updater");
    let dirs = LauncherDirectories {
        root: tmp.path().to_path_buf(),
    };
    let client = reqwest::Client::new();
    let core = core_binary_path(&dirs);

    // A hash that doesn't match: nothing is written.
    let mut wrong = version(server.url("/core"), &body, signed(&body));
    wrong.sha256 = "00".repeat(32);
    assert!(install_update(&client, &dirs, &wrong, |_, _| {})
        .await
        .is_err());
    assert!(!core.exists());

    // A signature over different bytes: refused as well.
    let forged = version(server.url("/core"), &body, signed(b"something else"));
    assert!(install_update(&client, &dirs, &forged, |_, _| {})
        .await
        .is_err());
    assert!(!core.exists());

    // The real thing: binary, signature beside it and the version file.
    let good = version(server.url("/core"), &body, signed(&body));
    let installed = install_update(&client, &dirs, &good, |_, _| {})
        .await
        .unwrap();
    assert_eq!(installed, core);
    assert_eq!(std::fs::read(&core).unwrap(), body);
    assert_eq!(
        std::fs::read_to_string(sig_path(&core)).unwrap(),
        good.signature
    );
    assert_eq!(
        std::fs::read_to_string(dirs.root().join("version")).unwrap(),
        "9.9.9"
    );
    assert!(!PathBuf::from(format!("{}.new", core.display())).exists());
}
