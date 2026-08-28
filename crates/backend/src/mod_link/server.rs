//! The loopback listener: handshake, then the frame loop.
//!
//! One connection per game. The first frame must be `Hello` carrying the key
//! from the handshake file; nothing else is accepted before it, not even a
//! queue request.

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
        // The bind is already 127.0.0.1, but checking is cheaper than reasoning
        // about what it actually did.
        if !addr.ip().is_loopback() {
            continue;
        }
        let (ctx, link) = (ctx.clone(), link.clone());
        tokio::spawn(async move {
            if let Err(e) = session(ctx, link, stream).await {
                tracing::debug!("mod_link: connection closed: {e}");
            }
        });
    }
}

async fn session(ctx: Ctx, link: ModLink, stream: TcpStream) -> anyhow::Result<()> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut sink, mut incoming) = ws.split();

    // Only something that read the file in the game directory knows the key. A
    // web page can open the socket; it can't read the file.
    let hello = incoming.next().await.transpose()?;
    let Some(Message::Text(text)) = hello else {
        anyhow::bail!("first frame was not text");
    };
    match serde_json::from_str::<ToLauncher>(&text) {
        Ok(ToLauncher::Hello { key, protocol }) if key == link.key() && !key.is_empty() => {
            if protocol != PROTOCOL {
                tracing::info!(
                    mod_protocol = protocol,
                    ours = PROTOCOL,
                    "mod_link: protocol versions differ"
                );
            }
        }
        _ => anyhow::bail!("handshake rejected"),
    }

    let (tx, mut outgoing) = mpsc::unbounded_channel::<ToMod>();
    link.attach(tx);
    link.send(push::ready(&ctx));
    // The queue goes out immediately: the panel gets opened to pick up the next
    // case.
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
                        // An unknown frame doesn't drop the connection. The mod
                        // ships with the build and outlives this launcher
                        // version on people's machines.
                        match serde_json::from_str::<ToLauncher>(&text) {
                            Ok(frame) => intents::handle(ctx, link, frame).await,
                            Err(e) => tracing::debug!("mod_link: could not parse frame: {e}"),
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
