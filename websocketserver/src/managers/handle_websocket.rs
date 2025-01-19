use std::sync::Arc;

use axum::extract::ws::{Utf8Bytes, WebSocket};

use crate::{managers::message_type_check::JoinMessage, AppState};

use super::validate_user::validate_user;

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(msg)) = socket.recv().await {
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
								// TODO:: Make api calls to add user to respective channels
								let is_valid_user = validate_user(token, user_id, &state.api_secret).await;
								println!("{:?}", is_valid_user);
							},
							crate::managers::message_type_check::IncomingMessageFromUser::LeaveMessage => {
								println!("Closed message sent");
								// TODO:: remove channel data
								if let Err(e) = socket
									.send(
										axum::extract::ws::Message::Close(None)).await {
											eprintln!("Error sending close acknowledgement: {:?}", e);
										}
								break;
							}
						}
                    }

                    Err(_) => {
                        let error_response = "Invalid message format".to_string();
                        socket
                            .send(axum::extract::ws::Message::Text(Utf8Bytes::from(
                                error_response,
                            )))
                            .await
                            .unwrap();
                    }
                }
            }

            axum::extract::ws::Message::Close(_) => {
                // TODO:: remove channel data
                println!("Client gone");
                break;
            }
            _ => {}
        }
    }
}
