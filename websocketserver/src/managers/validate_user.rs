use reqwest::Client;
use serde_json::json;

pub async fn validate_user(token: String, user_id: i32, api_secret: &str) -> bool {
    let client = Client::new();
    let url = "http://localhost:8000/websocket/isValidUser";
    let body = json!({
        "token": token,
        "endpoint_secret": api_secret,
        "user_id": user_id
    });

    let response = client.post(url).json(&body).send().await;
    if response.is_err() {
        return false;
    }

    let response_body = response.unwrap().json().await;
    if response_body.is_err() {
        return false;
    }

    response_body.unwrap()
}
