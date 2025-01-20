use std::sync::Arc;

use axum::extract::ws::{Utf8Bytes, WebSocket};
use tokio::sync::RwLock;

use crate::{managers::message_type_check::JoinMessage, AppState};

use super::{get_channels::get_channels, validate_user::validate_user};

use futures_util::{sink::SinkExt, stream::StreamExt};

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(RwLock::new(sender));
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            axum::extract::ws::Message::Text(text_message) => {
                match serde_json::from_str::<
                    crate::managers::message_type_check::IncomingMessageFromUser,
                >(&text_message)
                {
                    Ok(parsed_message) => {
                        match parsed_message {
							crate::managers::message_type_check::IncomingMessageFromUser::JoinMessage(JoinMessage {
								token,
								user_id
							}) => {
								let is_valid_user = validate_user(token, user_id, &state.api_secret).await;
								if !is_valid_user {
								if let Err(e) = sender.write().await
									.send(
										axum::extract::ws::Message::Close(None)).await{};
								break;
								}

								let channels = get_channels(user_id, &state.api_secret).await;
								state.channel_user_map.add_user(user_id, channels.unwrap(), sender.clone()).await;
							},
							crate::managers::message_type_check::IncomingMessageFromUser::LeaveMessage => {
								state.channel_user_map.remove_user(sender.clone()).await;
								let _ = sender.write().await.flush().await;
								drop(sender);
								break;
							}
						}
                    }

                    Err(_) => {
                        let error_response = "Invalid message format".to_string();
                        sender
                            .write()
                            .await
                            .send(axum::extract::ws::Message::Text(Utf8Bytes::from(
                                error_response,
                            )))
                            .await
                            .unwrap();
                    }
                }
            }

            axum::extract::ws::Message::Close(_) => {
                state.channel_user_map.remove_user(sender.clone()).await;
                let _ = sender.write().await.flush().await;
                drop(sender);
                break;
            }
            _ => {}
        }
    }
}
