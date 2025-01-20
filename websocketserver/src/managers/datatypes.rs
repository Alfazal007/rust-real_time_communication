use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug)]
pub struct Connection {
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct UserId(i32);

#[derive(Debug)]
pub struct ChannelManager {
    pub channels: Arc<RwLock<HashMap<i32, HashSet<UserId>>>>,
    pub connections: Arc<RwLock<HashMap<UserId, Connection>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ChannelManager {
    pub async fn user_connected(&self, user_id: i32) -> bool {
        let connections = self.connections.read().await;
        if connections.contains_key(&UserId(user_id)) {
            return true;
        }
        false
    }

    pub async fn add_user(
        &self,
        user_id: i32,
        channel_ids: Vec<i32>,
        websocket_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) {
        if !self.user_connected(user_id).await {
            let mut locked_channels = self.channels.write().await;
            let user_id = UserId(user_id);
            for channel_id in channel_ids {
                locked_channels
                    .entry(channel_id)
                    .or_insert_with(HashSet::new)
                    .insert(user_id.clone());
                // TODO:: subscribe to the channel id in pub sub
            }
            let mut connections = self.connections.write().await;
            connections.insert(
                user_id,
                Connection {
                    sender: websocket_sender,
                },
            );
        }
        self.print_room().await;
    }
}

impl ChannelManager {
    pub async fn remove_user(&self, user_id: i32) {
        let mut locked_channels = self.channels.write().await;
        let user_id = UserId(user_id);
        for (key, val) in locked_channels {}
        let mut connections = self.connections.write().await;
        connections.remove(&user_id);
        self.print_room().await;
    }
}

impl ChannelManager {
    pub async fn print_room(&self) {
        let locked_channels = self.channels.write().await;
        println!("Channels");
        println!("{:?}", locked_channels);
        let connections = self.connections.read().await;
        println!("Connections");
        println!("{:?}", connections);
    }
}
