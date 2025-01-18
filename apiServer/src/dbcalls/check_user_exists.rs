use serde::Deserialize;
use sqlx::prelude::FromRow;

use crate::AppState;

#[derive(Deserialize, FromRow)]
struct ExistsResult {
    exists: bool,
}
pub async fn check_user_exists(
    user_id: i32,
    username: &str,
    app_state: &AppState,
) -> Result<bool, String> {
    let query_result = sqlx::query_as::<_, ExistsResult>(
        "SELECT EXISTS(
        SELECT * FROM users WHERE username=$1 and id=$2
    ) AS exists", // Alias the result as "exists"
    )
    .bind(username)
    .bind(user_id)
    .fetch_optional(&app_state.database)
    .await;

    match query_result {
        Err(_) => Err("Issue talking to the database".to_string()),
        Ok(row_data) => match row_data {
            None => Err("Issue finding the user".to_string()),
            Some(res_struct) => {
                if !res_struct.exists {
                    Err("User not found".to_string())
                } else {
                    Ok(true)
                }
            }
        },
    }
}
