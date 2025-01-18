use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Validate)]
pub struct Channel {
    #[validate(length(
        min = 1,
        max = 20,
        message = "Channel name should be between 1 and 20 length"
    ))]
    #[serde(rename = "channelName")]
    pub channel_name: String,
}
