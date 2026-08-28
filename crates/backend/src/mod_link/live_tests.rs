//! End-to-end check of the channel: bring the listener up, pretend to be the
//! mod, get a case back.
//!
//! `#[ignore]`d because it needs a running master and a session token, neither
//! of which CI has. Run by hand when the channel changes:
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
#[ignore = "needs a live master: NORO_TEST_MASTER + NORO_TEST_TOKEN"]
async fn mod_gets_ready_and_queue() {
    let (Ok(master), Ok(token)) = (
        std::env::var("NORO_TEST_MASTER"),
        std::env::var("NORO_TEST_TOKEN"),
    ) else {
        panic!("set NORO_TEST_MASTER and NORO_TEST_TOKEN");
    };

    let dir = std::env::temp_dir().join(format!("noro-mod-link-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("instance directory");

    let ctx = ctx_for(&master, &token);
    ctx.mod_link.start(&ctx, dir.clone()).await;

    // Read the handshake file exactly the way the Java side will.
    let raw = std::fs::read(dir.join(HANDSHAKE_FILE)).expect("handshake file");
    let hs: Handshake = serde_json::from_slice(&raw).expect("parse handshake");
    assert_eq!(hs.protocol, PROTOCOL);

    let url = format!("ws://127.0.0.1:{}", hs.port);
    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("connect to the launcher");
    let hello = ToLauncher::Hello {
        key: hs.key.clone(),
        protocol: PROTOCOL,
    };
    ws.send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await
        .expect("handshake sent");

    let mut saw_ready = false;
    let mut first_case = None;
    for _ in 0..4 {
        match next_frame(&mut ws).await {
            Some(ToMod::Ready { permissions, .. }) => {
                println!("Ready, {} permissions", permissions.len());
                saw_ready = true;
            }
            Some(ToMod::Queue {
                cases,
                total,
                offset,
                ..
            }) => {
                println!("Queue, {} of {total} cases (offset {offset})", cases.len());
                first_case = cases.first().map(|c| c.id);
            }
            Some(other) => println!("frame: {other:?}"),
            None => break,
        }
        if saw_ready && first_case.is_some() {
            break;
        }
    }
    assert!(saw_ready, "no Ready frame arrived");
    let case_id = first_case.expect("the queue is empty, nothing to open");

    // Intent, request to the master, card back. That's the whole channel.
    let open = ToLauncher::OpenCase { case_id };
    ws.send(Message::Text(serde_json::to_string(&open).unwrap()))
        .await
        .expect("intent sent");
    let mut saw_card = false;
    for _ in 0..4 {
        match next_frame(&mut ws).await {
            Some(ToMod::Case { view }) => {
                println!(
                    "Case N-{}, {} events, chat allowed: {}",
                    view.brief.number,
                    view.events.len(),
                    view.chat_allowed
                );
                assert_eq!(view.brief.id, case_id);
                saw_card = true;
                break;
            }
            Some(other) => println!("frame: {other:?}"),
            None => break,
        }
    }

    ctx.mod_link.stop().await;
    assert!(saw_card, "no card arrived");
    assert!(
        !dir.join(HANDSHAKE_FILE).exists(),
        "the key was left lying around after stop"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The key is what makes this safe: any page can open the socket, but it can't
/// read the file.
#[tokio::test]
async fn wrong_key_is_refused() {
    let dir = std::env::temp_dir().join(format!("noro-mod-link-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("instance directory");

    let ctx = ctx_for("http://127.0.0.1:1", "no-token");
    ctx.mod_link.start(&ctx, dir.clone()).await;
    let raw = std::fs::read(dir.join(HANDSHAKE_FILE)).expect("handshake file");
    let hs: Handshake = serde_json::from_slice(&raw).expect("parse handshake");

    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{}", hs.port))
        .await
        .expect("connect");
    let hello = ToLauncher::Hello {
        key: "guessed".into(),
        protocol: PROTOCOL,
    };
    ws.send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await
        .ok();

    // The launcher has to close the connection without sending a single state
    // frame.
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next()).await;
    let refused = matches!(
        next,
        Ok(None) | Ok(Some(Err(_))) | Ok(Some(Ok(Message::Close(_))))
    );
    ctx.mod_link.stop().await;
    let _ = std::fs::remove_dir_all(&dir);
    assert!(refused, "a connection with a bad key stayed open: {next:?}");
}

type ModSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn next_frame(ws: &mut ModSocket) -> Option<ToMod> {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(std::time::Duration::from_secs(10), ws.next()).await
    else {
        return None;
    };
    Some(serde_json::from_str::<ToMod>(&text).expect("frame parsed"))
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
    // A throwaway directory: the test must not touch the real launcher config.
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
