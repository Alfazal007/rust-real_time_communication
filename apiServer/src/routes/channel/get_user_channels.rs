use actix_web::{web, HttpResponse, Responder};
use sqlx::prelude::FromRow;

use crate::{
    responses::general_error::GeneralError, validators::get_my_channels::WebSocketUserChannels,
    AppState,
};

#[derive(serde::Serialize, FromRow, Debug, serde::Deserialize)]
struct Channels {
    id: Vec<i32>,
}

pub async fn current_user_for_socket(
    app_state: web::Data<AppState>,
    ws_channel_user_data: web::Json<WebSocketUserChannels>,
) -> impl Responder {
    if app_state.api_secret != ws_channel_user_data.0.endpoint_secret {
        return HttpResponse::Unauthorized().json(GeneralError {
            message: "You are not authorized to visit this endpoint".to_string(),
        });
    }

    let channel_db_result =
        sqlx::query_as::<_, Channels>("select COALESCE(array_agg(channel_id), ARRAY[]::integer[]) as id from membership where user_id = $1")
            .bind(ws_channel_user_data.0.user_id)
            .fetch_optional(&app_state.database)
            .await;

    if channel_db_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    let channel_data = channel_db_result.as_ref().unwrap();
    if channel_data.is_none() {
        return HttpResponse::Ok().json(Channels { id: Vec::new() });
    }

    HttpResponse::Ok().json(channel_db_result.unwrap().unwrap())
}
