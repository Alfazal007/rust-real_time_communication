use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Validate, Debug)]
pub struct MessageSendType {
    #[validate(length(min = 1, message = "Message not provided"))]
    pub message: String,
    pub channel_id: i32,
}
