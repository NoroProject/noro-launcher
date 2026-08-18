//! Транспорт WebSocket лаунчер ↔ мастер: апгрейд, read-loop, канал отправки.
//!
//! Разбор самих сообщений — в [`super::messages`].

use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use schema::{ClientWsMsg, ServerWsMsg};

use tokio::sync::mpsc;
use uuid::Uuid;

/// WebSocket-апгрейд.
pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();

    // Канал для исходящих сообщений (из read-loop и из hub broadcast).
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerWsMsg>();
    let conn_id = state.ws.register(tx.clone());

    // Задача отправки.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink
                .send(Message::Text(msg.to_json().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut authed_user: Option<Uuid> = None;
    // Платформа приходит во входе и нужна манифесту: держим её рядом с сессией.
    let mut platform = String::new();

    while let Some(Ok(msg)) = stream.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => continue,
        };
        let parsed: Result<ClientWsMsg, _> = serde_json::from_str(&text);
        let Ok(client_msg) = parsed else {
            continue;
        };

        if let Err(e) = super::messages::handle(
            &state,
            conn_id,
            &mut authed_user,
            &mut platform,
            client_msg,
            &tx,
        )
        .await
        {
            tracing::warn!(error = %e, "WS message handling failed");
            let _ = tx.send(ServerWsMsg::Notification {
                key: "notif-server-error".into(),
                args: [("reason".to_string(), e.to_string())].into(),
                level: schema::NotifLevel::Error,
            });
        }
    }

    state.ws.unregister(conn_id);
    send_task.abort();
}
