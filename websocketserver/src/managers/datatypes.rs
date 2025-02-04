use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::sink::SinkExt;
use futures_util::stream::SplitSink;
use tokio::sync::{Mutex, RwLock};

#[derive(serde::Deserialize)]
struct MessageToBeBroadcasted {
    message: String,
    sender: i32,
}

use super::{subscribe_connection::RedisPubSub, unsubscribe_connection::unsubscribe_from_redis};

#[derive(Debug)]
pub struct Connection {
    pub sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct UserId(i32);

pub struct ChannelManager {
    pub channels: HashMap<i32, HashSet<UserId>>,
    pub connections: HashMap<UserId, Connection>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            connections: HashMap::new(),
        }
    }
}

impl ChannelManager {
    pub async fn user_connected(&self, user_id: i32) -> bool {
        if self.connections.contains_key(&UserId(user_id)) {
            return true;
        }
        false
    }

    pub async fn add_user(
        &mut self,
        user_id: i32,
        channel_ids: Vec<i32>,
        websocket_sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
        redis_subscription_struct: Arc<Mutex<RedisPubSub>>,
    ) {
        if !self.user_connected(user_id).await {
            let user_id = UserId(user_id);
            for channel_id in channel_ids.iter() {
                self.channels
                    .entry(*channel_id)
                    .or_insert_with(HashSet::new)
                    .insert(user_id.clone());
            }
            self.connections.insert(
                user_id,
                Connection {
                    sender: websocket_sender,
                },
            );
            {
                redis_subscription_struct
                    .lock()
                    .await
                    .subscribe(channel_ids)
                    .await;
            }
        }
    }
}

impl ChannelManager {
    pub async fn remove_user(&mut self, connection: &Arc<RwLock<SplitSink<WebSocket, Message>>>) {
        let user_id_to_be_removed = {
            self.connections
                .iter()
                .find(|(_, conn)| Arc::ptr_eq(&conn.sender, &connection))
                .map(|(user_id, _)| UserId(user_id.0))
                .unwrap_or(UserId(-1))
        };
        {
            self.connections.remove(&user_id_to_be_removed);
        }
        let mut channels_to_remove = Vec::new();
        {
            for (channel_id, user_set) in self.channels.iter_mut() {
                user_set.remove(&user_id_to_be_removed);
                if user_set.is_empty() {
                    channels_to_remove.push(*channel_id);
                }
            }
            for channel in channels_to_remove.iter() {
                self.channels.remove(channel);
            }
            unsubscribe_from_redis(channels_to_remove).await;
        }
    }
}

impl ChannelManager {
    pub async fn send_message(&self, channel_id: i32, message: &str) {
        let parsed_message: MessageToBeBroadcasted =
            serde_json::from_str(message).expect("Failed to parse JSON");
        let users = self.channels.get(&channel_id);
        if users.is_some() {
            for user_to_send_message_to in users.unwrap().iter() {
                if user_to_send_message_to.0 != parsed_message.sender {
                    let connection = self.connections.get(user_to_send_message_to).unwrap();
                    let sender = &connection.sender;
                    sender
                        .write()
                        .await
                        .send(axum::extract::ws::Message::Text(Utf8Bytes::from(
                            &parsed_message.message,
                        )))
                        .await
                        .unwrap();
                }
            }
        }
    }
}
