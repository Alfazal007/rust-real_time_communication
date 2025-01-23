use actix_web::{HttpResponse, Responder};

pub async fn hello_response() -> impl Responder {
    HttpResponse::Ok().json("Working route")
}
