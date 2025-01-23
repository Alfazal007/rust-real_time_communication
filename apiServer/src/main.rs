use actix_web::{
    middleware::{from_fn, Logger},
    web, App, HttpServer,
};
use log::info;
use redis::Client;
use sqlx::{postgres::PgPoolOptions, Postgres};
use std::env;

pub mod dbcalls;
pub mod middlewares;
pub mod models;
pub mod responses;
pub mod routes;
pub mod tokens;
pub mod validators;

pub struct AppState {
    pub database: sqlx::Pool<Postgres>,
    pub access_token_secret: String,
    pub redis_pool: r2d2::Pool<Client>,
    pub api_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Issue fetching the env");
    env_logger::Builder::new().parse_filters("info").init();

    let port = env::var("PORT").expect("Issue finding the port");
    let database_url = env::var("DATABASE_URL").expect("Issue finding the database url");
    let api_secret = env::var("API_SECRET").expect("Issue finding the api secret");
    let access_token_secret =
        env::var("ACCESS_TOKEN_SECRET").expect("Issue finding the access token secret");

    let redis_client =
        redis::Client::open("redis://127.0.0.1/").expect("Issue creating redis client");

    let redis_pool = r2d2::Pool::builder()
        .max_size(5)
        .build(redis_client)
        .expect("Issue establishing the redis connection pool");

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
                redis_pool: redis_pool.clone(),
                api_secret: api_secret.clone(),
            }))
            .route(
                "/",
                web::get().to(routes::test::hello_response::hello_response),
            )
            .route(
                "/{dummy}",
                web::get().to(routes::test::test_url_change::url_test_response),
            )
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
            .service(
                web::scope("/api/v1/message").service(
                    web::scope("/protected")
                        .wrap(from_fn(middlewares::auth_middleware::auth_middleware))
                        .route(
                            "/send",
                            web::post().to(routes::messages::send_message::send_message),
                        ),
                ),
            )
            .route(
                "/websocket/isValidUser",
                web::post().to(routes::user::current_user_for_socket::current_user_for_socket),
            )
            .route(
                "/websocket/channels",
                web::post().to(routes::channel::get_user_channels::current_user_for_socket),
            )
    })
    .bind(("127.0.0.1", port.parse().expect("Invalid port")))
    .expect("Port is already taken")
    .run()
    .await
}
