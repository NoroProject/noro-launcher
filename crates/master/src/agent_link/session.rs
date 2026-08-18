//! WebSocket-сессия агента: `GET /api/agent/link`.
//!
//! Соединение открывает агент — до игровой машины снаружи дороги может и не
//! быть. Авторизация та же, что у остального агентского API: секрет игрового
//! сервера, из него же мастер берёт, о какой сборке речь.

use crate::api::auth::AgentAuth;
use crate::db::GameServerRow;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

pub async fn ws_handler(
    State(state): State<AppState>,
    agent: AgentAuth,
    ws: WebSocketUpgrade,
) -> Response {
    let server = agent.game_server;
    ws.on_upgrade(move |socket| session(socket, state, server))
}

async fn session(socket: WebSocket, state: AppState, server: GameServerRow) {
    let name = server.name.clone();
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let conn_id = state.agents.register(server.server_id, server.id, tx);
    tracing::info!(server = %name, "агент подключился к каналу наказаний");

    let send_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Наказания агент по-прежнему выдаёт обычным POST — на них нужен ответ с
    // телом. Сюда приходят события игры, ответа на которые не бывает.
    while let Some(Ok(message)) = stream.next().await {
        match message {
            Message::Text(frame) => super::inbox::handle(&state, &server, &frame).await,
            Message::Close(_) => break,
            _ => {}
        }
    }

    tracing::info!(server = %name, "агент отключился от канала наказаний");
    state.agents.unregister(conn_id);
    // Состав онлайна больше ничем не подтверждён. Только когда ушёл последний
    // агент этого сервера: переподключение не должно опустошать список.
    if !state.agents.has_game_server(server.id) {
        state.roster.clear(server.id);
    }
    send_task.abort();
}
