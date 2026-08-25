//! Слушатель на loopback: рукопожатие и цикл кадров.
//!
//! Соединение одно на игру. Первым кадром обязан прийти `Hello` с ключом из
//! файла рукопожатия — до него не принимается ничего, включая запрос очереди.

use super::{intents, push, ModLink};
use crate::backend::Ctx;
use futures_util::{SinkExt, StreamExt};
use mod_link::{ToLauncher, ToMod, PROTOCOL};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub async fn serve(ctx: Ctx, link: ModLink, listener: TcpListener) {
    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            continue;
        };
        // Слушаем только 127.0.0.1, но проверка дешевле рассуждений о том,
        // что там на самом деле сделал bind.
        if !addr.ip().is_loopback() {
            continue;
        }
        let (ctx, link) = (ctx.clone(), link.clone());
        tokio::spawn(async move {
            if let Err(e) = session(ctx, link, stream).await {
                tracing::debug!("mod_link: соединение закрыто: {e}");
            }
        });
    }
}

async fn session(ctx: Ctx, link: ModLink, stream: TcpStream) -> anyhow::Result<()> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut sink, mut incoming) = ws.split();

    // Ключ знает только тот, кто прочитал файл в каталоге игры. Чужая
    // веб-страница открыть сокет может, прочитать файл — нет.
    let hello = incoming.next().await.transpose()?;
    let Some(Message::Text(text)) = hello else {
        anyhow::bail!("первый кадр не текстовый");
    };
    match serde_json::from_str::<ToLauncher>(&text) {
        Ok(ToLauncher::Hello { key, protocol }) if key == link.key() && !key.is_empty() => {
            if protocol != PROTOCOL {
                tracing::info!(
                    mod_protocol = protocol,
                    ours = PROTOCOL,
                    "mod_link: версии договора разошлись"
                );
            }
        }
        _ => anyhow::bail!("рукопожатие не принято"),
    }

    let (tx, mut outgoing) = mpsc::unbounded_channel::<ToMod>();
    link.attach(tx);
    link.send(push::ready(&ctx));
    // Очередь нужна сразу: панель открывают, чтобы взять следующее дело.
    push::refresh_queue(&ctx, &link, None, 0).await;

    let result = pump(&ctx, &link, &mut sink, &mut outgoing, &mut incoming).await;
    link.detach();
    result
}

type Sink = futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>;
type Incoming = futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<TcpStream>>;

async fn pump(
    ctx: &Ctx,
    link: &ModLink,
    sink: &mut Sink,
    outgoing: &mut mpsc::UnboundedReceiver<ToMod>,
    incoming: &mut Incoming,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            frame = outgoing.recv() => {
                let Some(frame) = frame else { return Ok(()) };
                sink.send(Message::Text(serde_json::to_string(&frame)?)).await?;
            }
            msg = incoming.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Незнакомый кадр не рвёт соединение: мод уезжает со
                        // сборкой и живёт у людей дольше, чем эта версия.
                        match serde_json::from_str::<ToLauncher>(&text) {
                            Ok(frame) => intents::handle(ctx, link, frame).await,
                            Err(e) => tracing::debug!("mod_link: кадр не разобран: {e}"),
                        }
                    }
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
        }
    }
}
