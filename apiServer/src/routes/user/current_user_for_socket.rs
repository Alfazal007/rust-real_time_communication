use actix_web::{web, HttpResponse, Responder};
use redis::Commands;

use crate::{
    responses::general_error::GeneralError, validators::get_socket_user_type::WebSocketUser,
    AppState,
};

pub async fn current_user_for_socket(
    app_state: web::Data<AppState>,
    ws_login_user_data: web::Json<WebSocketUser>,
) -> impl Responder {
    if app_state.api_secret != ws_login_user_data.0.endpoint_secret {
        return HttpResponse::Unauthorized().json(GeneralError {
            message: "You are not authorized to visit this endpoint".to_string(),
        });
    }

    let token = ws_login_user_data.0.token;
    let token_eval_result =
        crate::tokens::validate_token::validate_token(&token, &app_state.access_token_secret);

    if token_eval_result.is_err() {
        return HttpResponse::Ok().json(false);
    }

    let claims = token_eval_result.unwrap();
    let redis_connection_result = app_state.redis_pool.get();

    if claims.user_id != ws_login_user_data.0.user_id {
        return HttpResponse::Ok().json(false);
    }

    if redis_connection_result.is_ok() {
        let key = format!("auth:{}", claims.user_id);
        let token_redis_result: Result<String, _> = redis_connection_result.unwrap().get(key);
        if let Ok(token_redis) = token_redis_result {
            if token_redis == token {
                return HttpResponse::Ok().json(true);
            } else {
                return HttpResponse::Ok().json(false);
            }
        }
    }

    // redis is not connected so use this alternate way
    let user_exists = crate::dbcalls::check_user_exists::check_user_exists(
        claims.user_id,
        &claims.username,
        &app_state,
    )
    .await;

    match user_exists {
        Err(_) => HttpResponse::Ok().json(false),
        Ok(_) => HttpResponse::Ok().json(true),
    }
}
