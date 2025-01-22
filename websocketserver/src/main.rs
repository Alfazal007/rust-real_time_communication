use axum::extract::State;
use axum::routing::get;
use axum::{extract::WebSocketUpgrade, response::IntoResponse, Router};
use futures_util::StreamExt;
use managers::datatypes::ChannelManager;
use managers::subscribe_connection::RedisPubSub;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub api_secret: String,
    pub channel_user_map: ChannelManager,
    pub redis_client: redis::Client,
    pub redis_pub_sub_handler_struct: Arc<Mutex<RedisPubSub>>,
}

pub mod managers;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| managers::handle_websocket::handle_socket(socket, state))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Env file is not found");
    let api_secret = env::var("API_SECRET").expect("Issue finding the api secret url");
    let redis_client = Arc::new(Mutex::new(
        redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client"),
    ));

    let redis_pubsub_connection = redis_client
        .lock()
        .await
        .get_async_pubsub()
        .await
        .expect("Issue connecting to redis");

    let (pubsub_sink, mut pubsub_stream) = redis_pubsub_connection.split();

    let redis_subscription_struct =
        Arc::new(Mutex::new(managers::subscribe_connection::RedisPubSub {
            subscribed_channels: HashSet::new(),
            pubsub_sink,
        }));

    tokio::spawn(async move {
        loop {
            let mut message = pubsub_stream.next().await;
            println!("{:?}", message);
        }
    });

    let app_state = Arc::new(AppState {
        api_secret,
        channel_user_map: ChannelManager::new(),
        redis_client: redis_client.lock().await.clone(),
        redis_pub_sub_handler_struct: redis_subscription_struct,
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
