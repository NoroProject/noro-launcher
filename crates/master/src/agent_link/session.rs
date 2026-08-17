//! WebSocket-сессия агента: `GET /api/agent/link`.
//!
//! Соединение открывает агент — до игровой машины снаружи дороги может и не
//! быть. Авторизация та же, что у остального агентского API: секрет игрового
//! сервера, из него же мастер берёт, о какой сборке речь.

use crate::api::auth::AgentAuth;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn ws_handler(
    State(state): State<AppState>,
    agent: AgentAuth,
    ws: WebSocketUpgrade,
) -> Response {
    let server_id = agent.game_server.server_id;
    let name = agent.game_server.name.clone();
    ws.on_upgrade(move |socket| session(socket, state, server_id, name))
}

async fn session(socket: WebSocket, state: AppState, server_id: Uuid, name: String) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let conn_id = state.agents.register(server_id, tx);
    tracing::info!(server = %name, "агент подключился к каналу наказаний");

    let send_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Читаем, только чтобы заметить разрыв: агент сюда ничего не присылает.
    // Наказания он выдаёт обычным POST — на них нужен ответ с телом, а через
    // этот канал отвечать нечем.
    while let Some(Ok(message)) = stream.next().await {
        if matches!(message, Message::Close(_)) {
            break;
        }
    }

    tracing::info!(server = %name, "агент отключился от канала наказаний");
    state.agents.unregister(conn_id);
    send_task.abort();
}
