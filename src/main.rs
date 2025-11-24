use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use dotenvy::dotenv;
use std::env;

use crate::routes::{auth_routes, conversation_routes, friend_routes, message_routes, user_routes};

use crate::services::auth_service::AuthService;
use crate::services::conversation_service::ConversationService;
use crate::services::friend_request_service::FriendRequestService;
use crate::services::friend_service::FriendService;
use crate::services::message_service::MessageService;
use crate::services::otp_service::OtpService;
use crate::services::reset_token_service::ResetTokenService;
use crate::services::session_service::SessionService;
use crate::services::user_service::UserService;

mod controllers;
mod libs;
mod middlewares;
mod models;
mod routes;
mod services;
mod validations;
mod helpers;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

pub const REFRESH_TOKEN_TTL: i64 = 7 * 24 * 60 * 60;
pub const ACCESS_TOKEN_TTL: i64 = 15 * 60;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let ip_address = env::var("IP_ADDRESS").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());

    let _db = libs::db::get_database().await;

    let secret_key = env::var("ACCESS_TOKEN_SECRET").unwrap_or_else(|_| "secret".into());

    let auth_service = web::Data::new(AuthService {
        secret_key,
    });
    let user_service = web::Data::new(UserService { db: _db.clone() });
    let session_service = web::Data::new(SessionService {
        db: _db.clone(),
        refresh_token_ttl: REFRESH_TOKEN_TTL,
    });
    let otp_service = web::Data::new(OtpService { db: _db.clone() });
    let reset_token_service = web::Data::new(ResetTokenService {db: _db.clone()});
    let friend_service = web::Data::new(FriendService { db: _db.clone() });
    let friend_request_service = web::Data::new(FriendRequestService { db: _db.clone() });
    let conversation_service = web::Data::new(ConversationService { db: _db.clone() });
    let message_service = web::Data::new(MessageService { db: _db.clone() });

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
    reset_token_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong reset_tokens");
    friend_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong friends");
    friend_request_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong friend_requests");
    conversation_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong conversations");
    message_service
        .init_indexes()
        .await
        .expect("Lỗi khi đánh index trong messages");

    println!("Máy chủ đang chạy tại http://{}:{}", ip_address, port);

    tokio::spawn(libs::clear_rubbish::start_cleanup_task(otp_service.clone()));

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(_db.clone()))
            .app_data(auth_service.clone())
            .app_data(user_service.clone())
            .app_data(session_service.clone())
            .app_data(otp_service.clone())
            .app_data(reset_token_service.clone())
            .app_data(friend_service.clone())
            .app_data(friend_request_service.clone())
            .app_data(conversation_service.clone())
            .app_data(message_service.clone())
            .configure(auth_routes::config)
            .configure(user_routes::config)
            .configure(friend_routes::config)
            .configure(message_routes::config)
            .configure(conversation_routes::config)
            .service(hello)
    })
    .bind(format!("{}:{}", ip_address, port))?
    .run()
    .await
}
