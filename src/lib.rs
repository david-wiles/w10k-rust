use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use futures_util::stream::SplitSink;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use uuid::Uuid;


pub type Connections = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

pub fn spawn_sender(mut tx: SplitSink<WebSocket, Message>, mut rx: UnboundedReceiverStream<Message>) {
    // While this channel is open, read messages and send to the client
    tokio::spawn(async move {
        while let Some(message) = rx.next().await {
            tx.send(message).unwrap_or_else(|e| {
                eprintln!("websocket send error: {}", e);
            }).await;
        }
    });
}

pub async fn user_disconnected(my_id: Uuid, conns: &Connections) {
    conns.write().await.remove(&my_id);
}
