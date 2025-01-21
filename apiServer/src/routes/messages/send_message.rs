use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use validator::Validate;

use crate::{
    middlewares::auth_middleware::UserData,
    models::{membership::MembershipDb, message::MessagesDb},
    validators::message_type::MessageSendType,
    AppState,
};

pub async fn send_message(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    message_data: web::Json<MessageSendType>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }
    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    if let Err(e) = message_data.validate() {
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

    let transaction_res = app_state.database.begin().await;
    if transaction_res.is_err() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Issue starting the transaction".to_string(),
        });
    }

    let mut transaction = transaction_res.unwrap();

    let membership_channel_result = sqlx::query_as::<_, MembershipDb>(
        "select * from membership where user_id=$1 and channel_id=$2",
    )
    .bind(user_data.user_id)
    .bind(message_data.0.channel_id)
    .fetch_optional(transaction.as_mut())
    .await;

    if membership_channel_result.is_err() || membership_channel_result.as_ref().unwrap().is_none() {
        let rollback_res = transaction.rollback().await;

        if rollback_res.is_err() {
            return HttpResponse::InternalServerError().json(
                crate::responses::general_error::GeneralError {
                    message: "Issue rolling back the transaction".to_string(),
                },
            );
        }

        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue finding the channel".to_string(),
            },
        );
    }

    let send_message_result = sqlx::query_as::<_, MessagesDb>(
        "INSERT INTO messages (sender_id, channel_id, message) VALUES ($1, $2, $3) returning *",
    )
    .bind(user_data.user_id)
    .bind(message_data.0.channel_id)
    .bind(message_data.0.message)
    .fetch_optional(transaction.as_mut())
    .await;

    if send_message_result.is_err() || send_message_result.as_ref().unwrap().is_none() {
        let rollback_res = transaction.rollback().await;

        if rollback_res.is_err() {
            return HttpResponse::InternalServerError().json(
                crate::responses::general_error::GeneralError {
                    message: "Issue rolling back the transaction".to_string(),
                },
            );
        }

        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue sending the message".to_string(),
            },
        );
    }

    let commit_result = transaction.commit().await;
    if commit_result.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue committing the transaction".to_string(),
            },
        );
    }

    // TODO:: send message to the publisher
    HttpResponse::Ok().json(())
}
