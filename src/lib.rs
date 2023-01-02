use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::SystemTime;

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use uuid::Uuid;
use chrono::offset::Utc;
use chrono::DateTime;

pub type Connections = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

pub async fn user_connected(ws: WebSocket, conns: Connections) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = Uuid::new_v4();

    println!("new conection: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    // Send messages to the websocket received from this write stream
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

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
    user_disconnected(my_id, &conns).await;
}

// Sends the message to all connected websockets
pub async fn broadcast(conns: &Connections) {
    let now: DateTime<Utc> = SystemTime::now().into();
    let text = format!("The current time is {}", now.format("%+"));

    // Send message to all connected websockets
    for (&uid, tx) in conns.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(text.clone())) {
            // The tx is disconnected, the websocket is waiting to be removed from the map
        }
    }
}

async fn user_disconnected(my_id: Uuid, conns: &Connections) {
    conns.write().await.remove(&my_id);
}
