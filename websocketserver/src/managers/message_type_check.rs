#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum IncomingMessageFromUser {
    JoinMessage(JoinMessage),
    LeaveMessage,
}

#[derive(Debug, serde::Deserialize)]
pub struct JoinMessage {
    pub token: String,
    pub user_id: i32,
}
