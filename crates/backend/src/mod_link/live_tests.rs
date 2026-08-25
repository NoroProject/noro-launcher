//! Живая проверка канала: поднять слушатель, прикинуться модом, получить дело.
//!
//! Помечен `#[ignore]`: нужен работающий мастер и токен сессии, а CI ни того ни
//! другого не имеет. Запускается руками, когда правится канал:
//!
//! ```text
//! NORO_TEST_MASTER=http://127.0.0.1:8080 NORO_TEST_TOKEN=<uuid> \
//!   cargo test -p backend mod_link -- --ignored --nocapture
//! ```

use super::ModLink;
use crate::backend::Ctx;
use futures_util::{SinkExt, StreamExt};
use mod_link::{Handshake, ToLauncher, ToMod, HANDSHAKE_FILE, PROTOCOL};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
#[ignore = "нужен живой мастер: NORO_TEST_MASTER + NORO_TEST_TOKEN"]
async fn mod_gets_ready_and_queue() {
    let (Ok(master), Ok(token)) = (
        std::env::var("NORO_TEST_MASTER"),
        std::env::var("NORO_TEST_TOKEN"),
    ) else {
        panic!("задайте NORO_TEST_MASTER и NORO_TEST_TOKEN");
    };

    let dir = std::env::temp_dir().join(format!("noro-mod-link-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("каталог инстанса");

    let ctx = ctx_for(&master, &token);
    ctx.mod_link.start(&ctx, dir.clone()).await;

    // Мод читает файл рукопожатия — ровно так же, как это сделает Java.
    let raw = std::fs::read(dir.join(HANDSHAKE_FILE)).expect("файл рукопожатия");
    let hs: Handshake = serde_json::from_slice(&raw).expect("разбор рукопожатия");
    assert_eq!(hs.protocol, PROTOCOL);

    let url = format!("ws://127.0.0.1:{}", hs.port);
    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("соединение с лаунчером");
    let hello = ToLauncher::Hello {
        key: hs.key.clone(),
        protocol: PROTOCOL,
    };
    ws.send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await
        .expect("рукопожатие ушло");

    let mut saw_ready = false;
    let mut first_case = None;
    for _ in 0..4 {
        match next_frame(&mut ws).await {
            Some(ToMod::Ready { permissions, .. }) => {
                println!("Ready, прав: {}", permissions.len());
                saw_ready = true;
            }
            Some(ToMod::Queue {
                cases,
                total,
                offset,
                ..
            }) => {
                println!("Queue, дел: {} из {total} (сдвиг {offset})", cases.len());
                first_case = cases.first().map(|c| c.id);
            }
            Some(other) => println!("кадр: {other:?}"),
            None => break,
        }
        if saw_ready && first_case.is_some() {
            break;
        }
    }
    assert!(saw_ready, "не пришёл Ready");
    let case_id = first_case.expect("в очереди нет ни одного дела — нечего открывать");

    // Намерение → запрос к мастеру → карточка обратно. Это и есть весь канал.
    let open = ToLauncher::OpenCase { case_id };
    ws.send(Message::Text(serde_json::to_string(&open).unwrap()))
        .await
        .expect("намерение ушло");
    let mut saw_card = false;
    for _ in 0..4 {
        match next_frame(&mut ws).await {
            Some(ToMod::Case { view }) => {
                println!(
                    "Case N-{}, событий: {}, чат разрешён: {}",
                    view.brief.number,
                    view.events.len(),
                    view.chat_allowed
                );
                assert_eq!(view.brief.id, case_id);
                saw_card = true;
                break;
            }
            Some(other) => println!("кадр: {other:?}"),
            None => break,
        }
    }

    ctx.mod_link.stop().await;
    assert!(saw_card, "карточка не приехала");
    assert!(
        !dir.join(HANDSHAKE_FILE).exists(),
        "ключ остался лежать после остановки"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Ключ обязателен: чужая страница может открыть сокет, но не прочитать файл.
#[tokio::test]
async fn wrong_key_is_refused() {
    let dir = std::env::temp_dir().join(format!("noro-mod-link-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("каталог инстанса");

    let ctx = ctx_for("http://127.0.0.1:1", "no-token");
    ctx.mod_link.start(&ctx, dir.clone()).await;
    let raw = std::fs::read(dir.join(HANDSHAKE_FILE)).expect("файл рукопожатия");
    let hs: Handshake = serde_json::from_slice(&raw).expect("разбор рукопожатия");

    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{}", hs.port))
        .await
        .expect("соединение");
    let hello = ToLauncher::Hello {
        key: "подобранный".into(),
        protocol: PROTOCOL,
    };
    ws.send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await
        .ok();

    // Лаунчер обязан закрыть соединение, не прислав ни одного кадра состояния.
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next()).await;
    let refused = matches!(
        next,
        Ok(None) | Ok(Some(Err(_))) | Ok(Some(Ok(Message::Close(_))))
    );
    ctx.mod_link.stop().await;
    let _ = std::fs::remove_dir_all(&dir);
    assert!(refused, "соединение с чужим ключом не закрылось: {next:?}");
}

type ModSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn next_frame(ws: &mut ModSocket) -> Option<ToMod> {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(std::time::Duration::from_secs(10), ws.next()).await
    else {
        return None;
    };
    Some(serde_json::from_str::<ToMod>(&text).expect("кадр разобран"))
}

fn ctx_for(master: &str, token: &str) -> Ctx {
    let (_rx_backend, _backend, _rx_frontend, frontend) = bridge::create_pair();
    let (inbound, _inbound_rx) = tokio::sync::mpsc::unbounded_channel();
    let (conn, _conn_rx) = tokio::sync::mpsc::unbounded_channel();
    let ws = crate::ws_client::spawn(
        format!("{}/ws/launcher", master.replace("http", "ws")),
        Some(token.to_string()),
        inbound,
        conn,
    );
    // Каталог свой, временный: настоящий конфиг лаунчера тест портить не должен.
    let dirs = crate::directories::LauncherDirectories {
        root: std::env::temp_dir().join(format!("noro-mod-link-home-{}", uuid::Uuid::new_v4())),
    };
    std::fs::create_dir_all(&dirs.root).ok();
    let config = crate::persistent::Persistent::load(dirs.config_file());
    config.update(|c: &mut crate::config::LauncherConfig| c.master_url = master.to_string());
    let (internal, _internal_rx) = tokio::sync::mpsc::unbounded_channel();
    Ctx {
        frontend,
        ws,
        http: reqwest::Client::new(),
        optional: crate::persistent::Persistent::load(dirs.optional_mods_file()),
        dirs,
        config,
        running: Default::default(),
        internal,
        rpc: crate::discord_rpc::spawn_discord_rpc(),
        mod_link: ModLink::default(),
        profile: Default::default(),
    }
}
