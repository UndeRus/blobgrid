use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::{broadcast, Mutex};

use crate::state::AppState;

pub async fn ws_grid(
    ws: WebSocketUpgrade,
    //user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    log::debug!("connecting ws");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_ws(socket, addr, state))
}

async fn handle_ws(socket: WebSocket, addr: SocketAddr, state: AppState) {
    //TODO: check if already connected
    log::debug!("Connected ws from: {}", addr);
    let (sender, receiver) = socket.split();

    let sender = Arc::new(Mutex::new(sender));
    {
        let broadcast_receiver = state.broadcast.lock().await.subscribe();
        tokio::spawn(async move {
            recv_broadcast(sender, broadcast_receiver).await;
        });
    }
    read(receiver, state).await;
}

async fn read(mut receiver: SplitStream<WebSocket>, state: AppState) {
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Ping(_) => {
                log::debug!("Got ping");
            }
            Message::Pong(_) => {
                log::debug!("Got pong");
            }
            Message::Binary(bin) => {
                if bin.len() < 3 {
                    log::warn!("Wrong message, len is {}", bin.len());
                    continue;
                }
                let b0: usize = bin.get(0).cloned().unwrap_or(0) as usize;
                let b1: usize = bin.get(1).cloned().unwrap_or(0) as usize;
                let b2: usize = bin.get(2).cloned().unwrap_or(0) as usize;
                let index = b0 + (b1 << 8) + (b2 << 16);
                if index >= 1_000_000 {
                    continue;
                }
                state.toggle(index).await;
            }
            Message::Close(_) => {
                log::debug!("Disconnecting");
                return;
            }
            _ => {}
        }

        log::debug!("Broadcasting");
    }
}

async fn recv_broadcast(
    client_tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    mut broadcast_receiver: broadcast::Receiver<Message>,
) {
    while let Ok(msg) = broadcast_receiver.recv().await {
        if client_tx.lock().await.send(msg).await.is_err() {
            return; // disconnected.
        }
    }
}
