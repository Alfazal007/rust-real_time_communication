use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Validate)]
pub struct AddUserToChannel {
    #[validate(length(
        min = 1,
        max = 20,
        message = "Channel name should be between 1 and 20 length"
    ))]
    #[serde(rename = "channelName")]
    pub channel_name: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Username should be between 6 and 20 length"
    ))]
    pub username: String,
}
