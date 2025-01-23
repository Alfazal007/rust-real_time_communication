use actix_web::{HttpResponse, Responder};

pub async fn url_test_response(path: actix_web::web::Path<String>) -> impl Responder {
    HttpResponse::Ok().json(format!("Working route, the url is {}", path.as_str()))
}
