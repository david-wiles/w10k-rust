use warp::{Error, Filter};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use uuid::{Bytes, Uuid};

pub async fn setup_connection(ws: WebSocket, conns: w10k::Connections) {
    // Generate a random UUID to identify the new connection
    let my_id = Uuid::new_v4();

    println!("new connection: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Create channel for sending messages to this websocket
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    w10k::spawn_sender(user_ws_tx, rx);

    // Save the sender in our list of connected users.
    conns.write().await.insert(my_id, tx);

    // Read messages from the user until they disconnect
    while let Some(result) = user_ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(text) = msg.to_str() {
                if text.len() > 36 {
                    let dst = Uuid::parse_str(&text[..36]).unwrap();
                    if let Some(other_rx) = conns.read().await.get(&dst) {
                        other_rx.send(Message::binary(&text[36..])).unwrap();
                    }
                } else {
                    eprintln!("Invalid message");
                }
            }
        }
    }

    // Remove this websocket's sender from the map. Once this function exits, the
    // channels will be cleaned up automatically
    w10k::user_disconnected(my_id, &conns).await;
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connections using UUID (String?)
    let connections = w10k::Connections::default();
    // Turn our "state" into a new Filter...
    let conns_filter = warp::any().map(move || connections.clone());

    // GET /ws -> websocket upgrade
    let routes = warp::path("ws")
        .and(warp::ws())
        .and(conns_filter)
        .map(|ws: warp::ws::Ws, conns| {
            ws.on_upgrade(move |socket| setup_connection(socket, conns))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
