use reqwest::Client;
use serde_json::json;

#[derive(serde::Deserialize, Debug)]
pub struct Channels {
    pub id: Vec<i32>,
}

pub async fn get_channels(user_id: i32, api_secret: &str) -> Option<Vec<i32>> {
    let client = Client::new();
    let url = "http://localhost:8000/websocket/channels";
    let body = json!({
        "endpoint_secret": api_secret,
        "user_id": user_id
    });

    let response = client.post(url).json(&body).send().await;
    if response.is_err() {
        return None;
    }

    let response_text_result = response.unwrap().text().await;
    if response_text_result.is_err() {
        return None;
    }

    let channels_result: Result<Channels, _> = serde_json::from_str(&response_text_result.unwrap());
    if channels_result.is_err() {
        return None;
    }

    Some(channels_result.unwrap().id)
}
