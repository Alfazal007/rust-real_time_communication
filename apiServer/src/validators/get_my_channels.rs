use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Validate)]
pub struct WebSocketUserChannels {
    #[validate(length(min = 10, max = 10, message = "Secret not provided"))]
    pub endpoint_secret: String,
    pub user_id: i32,
}
