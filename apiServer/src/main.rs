use actix_web::{
    middleware::{from_fn, Logger},
    web, App, HttpServer,
};
use log::info;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub mod dbcalls;
pub mod middlewares;
pub mod models;
pub mod responses;
pub mod routes;
pub mod tokens;
pub mod validators;

pub struct AppState {
    pub database: Pool<Postgres>,
    pub access_token_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Issue fetching the env");
    env_logger::Builder::new().parse_filters("info").init();

    let database_url = env::var("DATABASE_URL").expect("Issue finding the database url");
    let access_token_secret =
        env::var("ACCESS_TOKEN_SECRET").expect("Issue finding the access token secret");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Issue connecting to the database");

    info!("Starting Actix Web server...");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                database: pool.clone(),
                access_token_secret: access_token_secret.clone(),
            }))
            .service(
                web::scope("/api/v1/user")
                    .route(
                        "/create",
                        web::post().to(routes::user::create_user::create_user),
                    )
                    .route(
                        "/login",
                        web::post().to(routes::user::login_user::login_user),
                    )
                    .service(
                        web::scope("/protected")
                            .wrap(from_fn(middlewares::auth_middleware::auth_middleware))
                            .route(
                                "/currentUser",
                                web::get().to(routes::user::current_user::get_current_user),
                            ),
                    ),
            )
            .service(
                web::scope("/api/v1/channel").service(
                    web::scope("/protected")
                        .wrap(from_fn(middlewares::auth_middleware::auth_middleware))
                        .route(
                            "/createChannel",
                            web::post().to(routes::channel::create_channel::create_channel),
                        )
                        .route(
                            "/addMember",
                            web::post()
                                .to(routes::channel::add_user_to_channel::add_user_to_channel),
                        ),
                ),
            )
    })
    .bind(("127.0.0.1", 8000))
    .expect("Port is already taken")
    .run()
    .await
}
