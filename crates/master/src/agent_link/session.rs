//! Agent WebSocket session: `GET /api/agent/link`.
//!
//! The agent dials out, since there may be no route into the game machine from
//! outside. Auth is the same as the rest of the agent API: the game server
//! secret, which also tells the master which build this is.

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
    tracing::info!(server = %name, "agent connected to punishment channel");

    let send_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Punishments still go out over POST, which needs a response body. What
    // arrives here is game events, and those are never answered.
    while let Some(Ok(message)) = stream.next().await {
        match message {
            Message::Text(frame) => super::inbox::handle(&state, &server, &frame).await,
            Message::Close(_) => break,
            _ => {}
        }
    }

    tracing::info!(server = %name, "agent disconnected from punishment channel");
    state.agents.unregister(conn_id);
    // Nothing confirms the roster any more. Only once the last agent for this
    // server is gone — a reconnect must not empty the list.
    if !state.agents.has_game_server(server.id) {
        state.roster.clear(server.id);
    }
    send_task.abort();
}
