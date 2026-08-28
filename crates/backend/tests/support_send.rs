//! Тот же путь, что дёргает кнопка «Сообщить о проблеме»: собрать → упаковать →
//! отправить мастеру. Запускается вручную против поднятого стенда.

#[tokio::test]
#[ignore = "нужен поднятый мастер и живая сессия"]
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
        "проверка кнопки",
    )
    .await
    .expect("бандл должен уехать");

    println!("бандл принят: {id}");
    let _ = std::fs::remove_dir_all(&dir);
}
