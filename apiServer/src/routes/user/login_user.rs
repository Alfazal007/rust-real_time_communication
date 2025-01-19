use actix_web::{
    cookie::{Cookie, SameSite},
    web, HttpResponse, Responder,
};
use redis::Commands;
use validator::Validate;

use crate::{validators::create_user_type::User, AppState};

#[derive(serde::Serialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "userId")]
    user_id: i32,
}

pub async fn login_user(
    app_state: web::Data<AppState>,
    login_user_data: web::Json<User>,
) -> impl Responder {
    if let Err(e) = login_user_data.validate() {
        let mut validation_errors: Vec<String> = Vec::new();
        for (_, err) in e.field_errors().iter() {
            if let Some(message) = &err[0].message {
                validation_errors.push(message.clone().into_owned());
            }
        }
        return HttpResponse::BadRequest().json(
            crate::responses::validation_errors::ValidationErrorsToBeReturned {
                errors: validation_errors,
            },
        );
    }

    let existing_user = sqlx::query_as::<_, crate::models::user::UserFromDBWithPassword>(
        "select * from users where username = $1",
    )
    .bind(&login_user_data.0.username)
    .fetch_optional(&app_state.database)
    .await;

    if existing_user.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if existing_user.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(crate::responses::general_error::GeneralError {
            message: "User not found".to_string(),
        });
    }

    let user_data = existing_user.unwrap().unwrap();

    let validate_password = bcrypt::verify(login_user_data.0.password, &user_data.password);

    if validate_password.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue validating password".to_string(),
            },
        );
    }

    if !validate_password.unwrap() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Incorrect password".to_string(),
        });
    }

    let access_token = crate::tokens::generate_token::generate_token(
        &user_data.username,
        user_data.id,
        &app_state.access_token_secret,
    );

    if access_token.is_err() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Issue generating access token".to_string(),
        });
    }

    let redis_connection_result = app_state.redis_pool.get();

    if redis_connection_result.is_ok() {
        let key = format!("auth:{}", user_data.id);
        let expiry_in_seconds = 86400; // 24 hours expiry
        let _: () = redis_connection_result
            .unwrap()
            .set_ex(key, access_token.as_ref().unwrap(), expiry_in_seconds)
            .unwrap();
    }

    let cookie1 = Cookie::build("accessToken", access_token.as_ref().unwrap())
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::None)
        .finish();

    let cookie2 = Cookie::build("userId", format!("{}", user_data.id))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::None)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie1)
        .cookie(cookie2)
        .json(LoginResponse {
            access_token: access_token.unwrap(),
            user_id: user_data.id,
        })
}
