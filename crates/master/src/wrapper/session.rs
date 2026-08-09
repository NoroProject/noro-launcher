//! WebSocket-сессия враппера.
//!
//! Соединение всегда открывает враппер: игровая машина может стоять за NAT, и
//! стучаться к ней с мастера — надежда, а не архитектура. Авторизация та же, что
//! у агента, — секрет игрового сервера, из него же берётся, о ком речь.

use super::proto::{FromWrapper, WrapperStatus};
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
    let game_server_id = agent.game_server.id;
    let name = agent.game_server.name.clone();
    ws.on_upgrade(move |socket| session(socket, state, game_server_id, name))
}

async fn session(socket: WebSocket, state: AppState, game_server_id: Uuid, name: String) {
    let (mut sink, mut stream) = socket.split();

    // Первым кадром обязан быть hello: без платформы и версии соединение
    // бесполезно, а «подключён неизвестно кто» админке нечего показывать.
    let Some(hello) = next_message(&mut stream).await else {
        return;
    };
    let (info, status) = match serde_json::from_str::<FromWrapper>(&hello) {
        Ok(FromWrapper::Hello { info, status }) => (info, status),
        _ => {
            tracing::warn!(server = %name, "враппер начал не с hello — рву соединение");
            return;
        }
    };
    tracing::info!(server = %name, platform = %info.platform, "враппер подключился");

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let conn = state.wrappers.register(game_server_id, info, status, tx);

    let send_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(text) = next_message(&mut stream).await {
        match serde_json::from_str::<FromWrapper>(&text) {
            Ok(FromWrapper::Console { line }) => conn.push_console(line),
            Ok(FromWrapper::Status { status }) => on_status(&state, game_server_id, &conn, status),
            Ok(FromWrapper::Reply {
                id,
                ok,
                data,
                error,
            }) => {
                let result = if ok {
                    Ok(data)
                } else {
                    Err(error.unwrap_or_else(|| "враппер не объяснил отказ".into()))
                };
                conn.resolve(id, result);
            }
            // Второй hello — это переподключение поверх живого сокета; новая
            // сессия зарегистрируется сама, здесь его игнорируем.
            Ok(FromWrapper::Hello { .. }) => {}
            Err(e) => tracing::warn!(server = %name, error = %e, "непонятный кадр от враппера"),
        }
    }

    tracing::info!(server = %name, "враппер отключился");
    state.wrappers.unregister(game_server_id, &conn);
    send_task.abort();
}

/// Статус приходит раз в полминуты и на каждое изменение.
///
/// Пока сервер грузится, отмечаем его живым отсюда: агент внутри ещё не
/// поднялся, а мастер считает сервер мёртвым через 90 секунд, и карточка в
/// лаунчере всё это время была бы серой. Как только сервер готов — замолкаем:
/// дальше heartbeat шлёт агент, и он знает настоящий онлайн, а наши нули его
/// затирали бы.
fn on_status(state: &AppState, game_server_id: Uuid, conn: &super::hub::Conn, status: WrapperStatus) {
    let was_ready = conn.state().status.ready;
    conn.set_status(status.clone());

    let booting = status.running && !status.ready;
    let just_ready = status.ready && !was_ready;
    if !booting && !just_ready {
        return;
    }

    let state = state.clone();
    tokio::spawn(async move {
        if booting {
            if let Err(e) = crate::db::touch_game_server(&state.db, game_server_id, 0, 0, None).await {
                tracing::warn!(error = %e, "не удалось отметить загрузку сервера");
            }
        }
        if just_ready {
            state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
        }
    });
}

async fn next_message(
    stream: &mut futures_util::stream::SplitStream<WebSocket>,
) -> Option<String> {
    while let Some(Ok(message)) = stream.next().await {
        match message {
            Message::Text(text) => return Some(text.to_string()),
            Message::Close(_) => return None,
            // Пинги axum отбивает сам, бинарных кадров в протоколе нет.
            _ => continue,
        }
    }
    None
}
