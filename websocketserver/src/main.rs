use axum::extract::ws::WebSocket;
use axum::extract::State;
use axum::routing::get;
use axum::{extract::WebSocketUpgrade, response::IntoResponse, Router};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ChannelToUserMap {
    pub channel_id: i32,
    pub connections: HashSet<WebSocket>,
}

pub struct AppState {
    pub api_secret: String,
    pub channel_user_map: Arc<Mutex<HashSet<ChannelToUserMap>>>,
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
    let app_state = Arc::new(AppState {
        api_secret,
        channel_user_map: Arc::new(Mutex::new(HashSet::new())),
    });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
