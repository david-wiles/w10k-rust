use std::env;
use warp::Filter;
use tokio::time::{Duration, interval};

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
                w10k::broadcast(&connections).await;
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
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| w10k::user_connected(socket, conns))
            });

        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    }
}
