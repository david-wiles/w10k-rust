use std::env;
use std::time::SystemTime;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tokio::time::{Duration, interval};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;

use chrono::offset::Utc;
use chrono::DateTime;

use futures_util::{SinkExt, StreamExt, TryFutureExt};

use warp::ws::{Message, WebSocket};
use warp::Filter;
use uuid::Uuid;


// Sends the message to all connected websockets
pub async fn broadcast(conns: &w10k::Connections) {
    let now: DateTime<Utc> = SystemTime::now().into();
    let text = format!("The current time is {}", now.format("%+"));

    // Acquire read lock on conns and write to every client
    for (&uid, tx) in conns.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(text.clone())) {
            // The tx is disconnected, the websocket is waiting to be removed from the map
        }
    }
}


pub async fn setup_connection(ws: WebSocket, conns: w10k::Connections) {
    // Generate a random UUID to identify the new connection
    let my_id = Uuid::new_v4();

    println!("new connection: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Create channel for sending messages to this websocket
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    // Spawn a thread which will send messages to the websocket using the unbounded stream
    w10k::spawn_sender(user_ws_tx, rx);

    // Save the sender in our list of connected users.
    conns.write().await.insert(my_id, tx);

    // Read messages from the user until they disconnect
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => println!("received message from conn {}: {}", my_id, msg.to_str().unwrap_or("[bytes]")),
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
    }

    // Remove this websocket's sender from the map. Once this function exits, the
    // channels will be cleaned up automatically
    w10k::user_disconnected(my_id, &conns).await;
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected client using UUID
    let connections = w10k::Connections::default();
    {
        let connections = connections.clone();

        // Start broadcasting to all connections
        tokio::spawn(async move {
            let ping_interval = env::var("PING_INTERVAL")
                .unwrap_or("10000".to_string())
                .parse::<u64>()
                .unwrap();
            let mut ticker = interval(Duration::from_millis(ping_interval));
            loop {
                ticker.tick().await;
                broadcast(&connections).await;
            }
        });
    }

    {
        let connections = connections.clone();
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
}
