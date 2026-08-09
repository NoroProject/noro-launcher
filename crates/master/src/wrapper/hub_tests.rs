//! Тесты канала управления. Проверяем то, что связывает два языка: форму кадра
//! и сопоставление ответа с запросом.

use super::hub::WrapperHub;
use super::proto::{Op, WrapperInfo, WrapperStatus};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use uuid::Uuid;

fn info() -> WrapperInfo {
    WrapperInfo {
        platform: "paper".into(),
        mc_version: "1.21.1".into(),
        wrapper_version: "dev".into(),
        server_dir: "/srv/mc".into(),
    }
}

#[tokio::test]
async fn call_sends_a_frame_java_can_read() {
    let hub = WrapperHub::default();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = Uuid::new_v4();
    let conn = hub.register(id, info(), WrapperStatus::default(), tx);

    let call = tokio::spawn(async move {
        conn.call(Op::Power {
            action: "restart".into(),
        })
        .await
    });

    let frame: Value = serde_json::from_str(&rx.recv().await.expect("кадр ушёл")).unwrap();
    assert_eq!(frame["type"], "request");
    assert_eq!(frame["op"], "power");
    assert_eq!(frame["args"], json!({ "action": "restart" }));
    // Идентификатор обязателен: по нему враппер адресует ответ.
    assert!(frame["id"].is_u64());

    hub.get(id)
        .unwrap()
        .resolve(frame["id"].as_u64().unwrap(), Ok(json!({ "ok": true })));
    assert_eq!(call.await.unwrap().unwrap(), json!({ "ok": true }));
}

#[tokio::test]
async fn unit_op_still_carries_its_name() {
    let hub = WrapperHub::default();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let conn = hub.register(Uuid::new_v4(), info(), WrapperStatus::default(), tx);
    tokio::spawn(async move { conn.call(Op::BackupList).await });

    let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
    assert_eq!(frame["op"], "backup_list");
}

#[tokio::test]
async fn failure_from_wrapper_becomes_an_error() {
    let hub = WrapperHub::default();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = Uuid::new_v4();
    let conn = hub.register(id, info(), WrapperStatus::default(), tx);

    let call = tokio::spawn(async move {
        conn.call(Op::Command {
            line: "stop".into(),
        })
        .await
    });
    let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
    hub.get(id).unwrap().resolve(
        frame["id"].as_u64().unwrap(),
        Err("server is not running".into()),
    );

    let error = call.await.unwrap().unwrap_err().to_string();
    assert!(error.contains("server is not running"), "{error}");
}

#[tokio::test]
async fn call_without_a_reader_fails_immediately() {
    // Приёмник уронили — это ровно то, что происходит при обрыве сокета.
    let hub = WrapperHub::default();
    let (tx, rx) = mpsc::unbounded_channel();
    let conn = hub.register(Uuid::new_v4(), info(), WrapperStatus::default(), tx);
    drop(rx);

    let error = conn
        .call(Op::Power {
            action: "stop".into(),
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("отключился"), "{error}");
}

#[test]
fn reconnect_does_not_let_the_old_socket_unregister_the_new_one() {
    let hub = WrapperHub::default();
    let id = Uuid::new_v4();
    let (tx_old, _rx_old) = mpsc::unbounded_channel();
    let (tx_new, _rx_new) = mpsc::unbounded_channel();

    let old = hub.register(id, info(), WrapperStatus::default(), tx_old);
    let _new = hub.register(id, info(), WrapperStatus::default(), tx_new);
    // Старая сессия дочитала свой сокет и уходит — новая должна остаться.
    hub.unregister(id, &old);

    assert!(hub.get(id).is_some());
}

#[test]
fn console_keeps_a_tail_for_whoever_opens_it_later() {
    let hub = WrapperHub::default();
    let (tx, _rx) = mpsc::unbounded_channel();
    let conn = hub.register(Uuid::new_v4(), info(), WrapperStatus::default(), tx);

    for i in 0..600 {
        conn.push_console(format!("line {i}"));
    }
    let backlog = conn.backlog();
    assert_eq!(backlog.len(), 500);
    assert_eq!(backlog.first().unwrap(), "line 100");
    assert_eq!(backlog.last().unwrap(), "line 599");
}
