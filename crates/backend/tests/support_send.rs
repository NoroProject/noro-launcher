//! The path the report button takes: collect, pack, upload. Run by hand against
//! a running master.

#[tokio::test]
#[ignore = "needs a running master and a live session"]
async fn the_report_button_path_reaches_the_master() {
    let master = std::env::var("NORO_TEST_MASTER").unwrap();
    let token = std::env::var("NORO_TEST_TOKEN").unwrap();

    let dir = std::env::temp_dir().join("noro-send-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("logs")).unwrap();
    std::fs::write(
        dir.join("logs/latest.log"),
        "[main/INFO]: started\n--accessToken eyJhbGciOiJIUzI1NiJ9.SECRET.sig\n",
    )
    .unwrap();

    let id = backend::support::send(
        &reqwest::Client::new(),
        &master,
        &token,
        &dir,
        None,
        "button check",
    )
    .await
    .expect("the bundle should upload");

    println!("bundle accepted: {id}");
    let _ = std::fs::remove_dir_all(&dir);
}
