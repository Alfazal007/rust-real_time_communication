use axum::extract::State;
use axum::routing::get;
use axum::{extract::WebSocketUpgrade, response::IntoResponse, Router};
use futures_util::StreamExt;
use managers::datatypes::ChannelManager;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub api_secret: String,
    pub channel_user_map: ChannelManager,
    pub redis_client: redis::Client,
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

    let redis_connection = redis_client
        .lock()
        .await
        .get_async_pubsub()
        .await
        .expect("Issue connecting to redis");

    let pubsubparent = Arc::new(Mutex::new(redis_connection));
    let pubsub = pubsubparent.clone();

    tokio::spawn(async move {
        loop {
            let msgg = pubsub.lock().await.on_message().next().await;
            match msgg {
                Some(msg) => {
                    let channel: String = msg.get_channel_name().to_string();
                    let payload: String = msg.get_payload().expect("Invalid redis message");
                    println!("Received on '{}': {}", channel, payload);
                }
                None => {
                    break;
                }
            }
        }
    });

    let app_state = Arc::new(AppState {
        api_secret,
        channel_user_map: ChannelManager::new(),
        redis_client: redis_client.lock().await.clone(),
    });

    let mut redis_subscription_struct = managers::subscribe_connection::RedisPubSub {
        subscribed_channels: HashSet::new(),
    };
    {
        let pubsubclone = pubsubparent.clone();
        let mut c = pubsubclone.lock().await;

        redis_subscription_struct
            .subscribe(&mut c, vec![1, 2, 3])
            .await;
    }
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
