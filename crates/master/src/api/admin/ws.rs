//! Admin WebSocket: push instead of polling.
//!
//! The token arrives in the first frame rather than a header — the browser
//! `WebSocket` API can't set `Authorization`, and putting it in the query
//! string writes the session into every proxy log on the way.

use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use schema::{AdminWsClientMsg, AdminWsMsg};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<AdminWsMsg>();
    let conn_id = state.admin_ws.register(tx.clone());

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

    while let Some(Ok(msg)) = stream.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<AdminWsClientMsg>(&text) else {
            continue;
        };
        match parsed {
            AdminWsClientMsg::Authenticate { access_token } => {
                match authenticate(&state, &access_token).await {
                    Some(user_id) => {
                        state.admin_ws.authenticate(conn_id, user_id);
                        let _ = tx.send(AdminWsMsg::AuthOk);
                    }
                    None => {
                        let _ = tx.send(AdminWsMsg::AuthFail);
                        break;
                    }
                }
            }
            AdminWsClientMsg::Ping => {
                let _ = tx.send(AdminWsMsg::Pong);
            }
        }
    }

    // Drain rather than `abort()`. The last frame out is often `AuthFail`, and
    // killing the send task would swallow it — the tab would see a plain
    // disconnect and reconnect with the same bad token forever.
    state.admin_ws.unregister(conn_id);
    drop(tx);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), send_task).await;
}

/// Same session token as REST. No permission check here on purpose: the frames
/// carry no data, and the page fetches anything it needs over the normal API.
async fn authenticate(state: &AppState, token: &str) -> Option<Uuid> {
    let token = Uuid::parse_str(token).ok()?;
    let row = crate::db::user_by_access_token(&state.db, token)
        .await
        .ok()??;
    if row.banned {
        return None;
    }
    Some(row.id)
}
