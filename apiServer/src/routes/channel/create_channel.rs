use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use validator::Validate;

use crate::{
    middlewares::auth_middleware::UserData, models::channel::ChannelDB,
    responses::general_error::GeneralError, validators::create_channel_type::Channel, AppState,
};

pub async fn create_channel(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    create_channel_data: web::Json<Channel>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if let Err(e) = create_channel_data.validate() {
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

    let channel_name = create_channel_data.0.channel_name;

    let existing_channel_with_same_name =
        sqlx::query_as::<_, ChannelDB>("select * from channel where name = $1")
            .bind(&channel_name)
            .fetch_optional(&app_state.database)
            .await;

    if existing_channel_with_same_name.is_err() {
        return HttpResponse::InternalServerError().json(
            crate::responses::general_error::GeneralError {
                message: "Issue talking to the database".to_string(),
            },
        );
    }

    if existing_channel_with_same_name.unwrap().is_some() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Channel with this name already exists".to_string(),
        });
    }

    let transaction_res = app_state.database.begin().await;
    if transaction_res.is_err() {
        return HttpResponse::BadRequest().json(crate::responses::general_error::GeneralError {
            message: "Issue starting the transaction".to_string(),
        });
    }

    let mut transaction = transaction_res.unwrap();

    let create_channel_result =
        sqlx::query_as::<_, ChannelDB>("INSERT INTO channel (name) VALUES ($1) returning *")
            .bind(channel_name)
            .fetch_optional(transaction.as_mut())
            .await;

    if create_channel_result.is_err() || create_channel_result.as_ref().unwrap().is_none() {
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
                message: "Issue creating the channel".to_string(),
            },
        );
    }

    let add_user_to_channel_result =
        sqlx::query("INSERT INTO membership (user_id, channel_id) VALUES ($1, $2)")
            .bind(user_data.user_id)
            .bind(create_channel_result.as_ref().unwrap().as_ref().unwrap().id)
            .execute(transaction.as_mut())
            .await;

    if add_user_to_channel_result.is_err() {
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
                message: "Issue creating the channel and adding user to it".to_string(),
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

    HttpResponse::Ok().json(create_channel_result.unwrap().unwrap())
}
