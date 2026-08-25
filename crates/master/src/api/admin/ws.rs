//! WebSocket админки: пуш вместо опроса.
//!
//! До него карточка дела перечитывалась раз в пять секунд — и всё равно
//! отставала ровно настолько, чтобы модератор успел нажать кнопку дважды.
//!
//! Токен приходит первым кадром, а не заголовком: браузерный `WebSocket` не
//! умеет слать `Authorization`, а тащить его в query значит положить сессию
//! в логи прокси.

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

    // Не `abort()`: последним кадром часто уходит `AuthFail`, и оборванная
    // задача отправки съедала бы его. Вкладка тогда видела бы просто разрыв и
    // переподключалась с тем же негодным токеном по кругу.
    state.admin_ws.unregister(conn_id);
    drop(tx);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), send_task).await;
}

/// Токен сессии — тот же, что у REST. Права здесь не проверяются: кадр не несёт
/// данных, а за карточкой страница пойдёт обычным запросом.
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
