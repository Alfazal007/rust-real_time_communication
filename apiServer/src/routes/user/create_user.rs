use actix_web::{web, HttpResponse, Responder};
use validator::Validate;

use crate::{validators::create_user_type::User, AppState};

pub async fn create_user(
    app_state: web::Data<AppState>,
    create_user_data: web::Json<User>,
) -> impl Responder {
    if let Err(e) = create_user_data.validate() {
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

    let existing_user = sqlx::query_as::<_, crate::models::user::UserFromDB>(
        "select * from users where username = $1",
    )
    .bind(&create_user_data.0.username)
    .fetch_optional(&app_state.database)
    .await;

    if existing_user.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if existing_user.unwrap().is_some() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Username already taken, try a different one".to_string(),
        });
    }

    let hashed_password = bcrypt::hash(&create_user_data.0.password, 12);
    if hashed_password.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue processing the password".to_string(),
            },
        );
    }

    let new_user = sqlx::query_as::<_, crate::models::user::UserFromDB>(
        "insert into users(username, password) values($1, $2) returning *",
    )
    .bind(create_user_data.0.username)
    .bind(hashed_password.unwrap())
    .fetch_optional(&app_state.database)
    .await;

    if new_user.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if new_user.as_ref().unwrap().is_none() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Issue writing to the database".to_string(),
        });
    }

    HttpResponse::Ok().json(new_user.unwrap().unwrap())
}
