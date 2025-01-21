use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::sink::SinkExt;
use futures_util::stream::SplitSink;
use tokio::sync::RwLock;

use super::{subscribe_connection, unsubscribe_connection::unsubscribe_from_redis};

#[derive(Debug)]
pub struct Connection {
    pub sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
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
        websocket_sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
        //        redis_pool: &Pool<Client>,
    ) {
        if !self.user_connected(user_id).await {
            let mut locked_channels = self.channels.write().await;
            let user_id = UserId(user_id);
            for channel_id in channel_ids.iter() {
                locked_channels
                    .entry(*channel_id)
                    .or_insert_with(HashSet::new)
                    .insert(user_id.clone());
            }
            let mut connections = self.connections.write().await;
            connections.insert(
                user_id,
                Connection {
                    sender: websocket_sender,
                },
            );

            //             subscribe_connection::subscribe(channel_ids).await;
        }
        self.print_room().await;
    }
}

impl ChannelManager {
    pub async fn remove_user(&self, connection: &Arc<RwLock<SplitSink<WebSocket, Message>>>) {
        let user_id_to_be_removed = {
            let user_id_connections = self.connections.read().await;
            user_id_connections
                .iter()
                .find(|(_, conn)| Arc::ptr_eq(&conn.sender, &connection))
                .map(|(user_id, _)| UserId(user_id.0))
                .unwrap_or(UserId(-1))
        };

        {
            let mut user_id_connections = self.connections.write().await;
            user_id_connections.remove(&user_id_to_be_removed);
        }

        let mut channels_to_remove = Vec::new();
        {
            let mut channels_user_id = self.channels.write().await;
            for (channel_id, user_set) in channels_user_id.iter_mut() {
                user_set.remove(&user_id_to_be_removed);
                if user_set.is_empty() {
                    channels_to_remove.push(*channel_id);
                }
            }
            for channel in channels_to_remove.iter() {
                channels_user_id.remove(channel);
            }
            unsubscribe_from_redis(channels_to_remove).await;
        }
        self.print_room().await;
    }
}

impl ChannelManager {
    pub async fn print_room(&self) {
        {
            let locked_channels = self.channels.write().await;
            println!("Channels");
            println!("{:?}", locked_channels);
            let connections = self.connections.read().await;
            println!("Connections");
            println!("{:?}", connections);
        }
    }
}

impl ChannelManager {
    pub async fn send_message(&self, channel_id: i32, message: String) {
        let channel_user_ids = self.channels.read().await;
        let connections = self.connections.write().await;
        let users = channel_user_ids.get(&channel_id);
        if users.is_some() {
            for user_to_send_message_to in users.unwrap().iter() {
                let connection = connections.get(user_to_send_message_to).unwrap();
                let sender = &connection.sender;
                sender
                    .write()
                    .await
                    .send(axum::extract::ws::Message::Text(Utf8Bytes::from(&message)))
                    .await
                    .unwrap();
            }
        }
    }
}
