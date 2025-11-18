use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use dotenvy::dotenv;
use std::env;

use crate::routes::{auth_route, user_route};

use crate::services::auth_service::AuthService;
use crate::services::otp_service::OtpService;
use crate::services::session_service::SessionService;
use crate::services::user_service::UserService;

mod controllers;
mod libs;
mod middlewares;
mod models;
mod routes;
mod services;
mod validations;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

const REFRESH_TOKEN_TTL: i64 = 7 * 24 * 60 * 60;
const ACCESS_TOKEN_TTL: i64 = 15 * 60;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let ip_address = env::var("IP_ADDRESS").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());

    let _db = libs::db::get_database().await;

    let secret_key = env::var("ACCESS_TOKEN_SECRET").unwrap_or_else(|_| "secret".into());

    let auth_service = web::Data::new(AuthService {
        secret_key,
        access_token_ttl: ACCESS_TOKEN_TTL,
    });
    let user_service = web::Data::new(UserService { db: _db.clone() });
    let session_service = web::Data::new(SessionService {
        db: _db.clone(),
        refresh_token_ttl: REFRESH_TOKEN_TTL,
    });
    let otp_service = web::Data::new(OtpService { db: _db.clone() });

    user_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong users");
    session_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong sessions");
    otp_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong otps");

    println!("Máy chủ đang chạy tại http://{}:{}", ip_address, port);

    tokio::spawn(libs::clear_rubbish::start_cleanup_task(
        otp_service.clone(),  
    ));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(_db.clone()))
            .app_data(auth_service.clone())
            .app_data(user_service.clone())
            .app_data(session_service.clone())
            .app_data(otp_service.clone())
            .configure(auth_route::config)
            .configure(user_route::config)
            .service(hello)
    })
    .bind(format!("{}:{}", ip_address, port))?
    .run()
    .await
}
