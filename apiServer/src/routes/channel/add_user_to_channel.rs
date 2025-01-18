use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use validator::Validate;

use crate::{
    middlewares::auth_middleware::UserData,
    models::{channel::ChannelDB, membership::MembershipDb, user::UserFromDB},
    validators::add_user_to_channel_type::AddUserToChannel,
    AppState,
};

pub async fn add_user_to_channel(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    add_user_to_channel_data: web::Json<AddUserToChannel>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if let Err(e) = add_user_to_channel_data.validate() {
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

    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    if add_user_to_channel_data.0.username == user_data.username {
        return HttpResponse::Unauthorized().json(crate::responses::general_error::GeneralError {
            message: "You are not the channel admin".to_string(),
        });
    }

    let channel_result = sqlx::query_as::<_, ChannelDB>("select * from channel where name=$1")
        .bind(&add_user_to_channel_data.0.channel_name)
        .fetch_optional(&app_state.database)
        .await;

    if channel_result.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if channel_result.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(crate::responses::general_error::GeneralError {
            message: "Channel not found".to_string(),
        });
    }

    if channel_result.as_ref().unwrap().as_ref().unwrap().admin_id != user_data.user_id {
        return HttpResponse::Unauthorized().json(crate::responses::general_error::GeneralError {
            message: "You are not the channel admin".to_string(),
        });
    }

    let user_result = sqlx::query_as::<_, UserFromDB>("select * from users where username=$1")
        .bind(add_user_to_channel_data.0.username)
        .fetch_optional(&app_state.database)
        .await;

    if user_result.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if user_result.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(crate::responses::general_error::GeneralError {
            message: "User not found".to_string(),
        });
    }

    let membership_result = sqlx::query_as::<_, MembershipDb>(
        "select * from membership where user_id=$1 and channel_id=$2",
    )
    .bind(user_result.as_ref().unwrap().as_ref().unwrap().id)
    .bind(channel_result.as_ref().unwrap().as_ref().unwrap().id)
    .fetch_optional(&app_state.database)
    .await;

    if membership_result.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if membership_result.as_ref().unwrap().is_some() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Already part of this channel".to_string(),
        });
    }

    let new_member = sqlx::query_as::<_, MembershipDb>(
        "insert into membership(user_id, channel_id) values ($1,$2) returning *",
    )
    .bind(user_result.unwrap().unwrap().id)
    .bind(channel_result.unwrap().unwrap().id)
    .fetch_optional(&app_state.database)
    .await;

    if new_member.is_err() || new_member.as_ref().unwrap().is_none() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue inserting to the database".to_string(),
            },
        );
    }

    HttpResponse::Ok().json(new_member.unwrap())
}
