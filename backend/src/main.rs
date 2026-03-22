use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    self, App, HttpServer,
    http::header,
    middleware::{Logger, from_fn},
    web,
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    configs::{AppConfig, RedisCache, connect_database_with_config},
    middlewares::{authentication, authorization},
    modules::{
        call::{
            handler::CallHandler,
            repository_pg::{CallParticipantPgRepository, CallPgRepository},
            service::CallService,
        },
        conversation::{
            repository_pg::{
                ConversationPgRepository, LastMessagePgRepository, ParticipantPgRepository,
            },
            service::ConversationService,
        },
        file_upload::{repository_pg::FilePgRepository, service::FileUploadService},
        friend::{repository_pg::FriendRepositoryPg, service::FriendService},
        message::{repository_pg::MessageRepositoryPg, service::MessageService},
        user::{repository_pg::UserRepositoryPg, schema::UserRole, service::UserService},
        websocket::{
            handler::websocket_handler, presence::PresenceService, server::WebSocketServer,
        },
    },
};

mod api;
mod app_state;
mod configs;
mod middlewares;
pub mod modules;
mod observability;
mod utils;

#[cfg(test)]
mod tests;

#[actix_web::get("/")]
async fn health_check(_db_pool: web::Data<sqlx::PgPool>) -> &'static str {
    "Server is running"
}

#[actix_web::get("/metrics")]
async fn metrics(app_state: web::Data<AppState>) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(app_state.metrics.prometheus_text())
}

#[actix_web::get("/metrics/json")]
async fn metrics_json(app_state: web::Data<AppState>) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(app_state.metrics.snapshot())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();

    tracing::info!("Tracing initialized");
    tracing::info!("Environment variables loaded from .env file");

    let app_config = AppConfig::from_env().map_err(|e| {
        eprintln!("Config error: {e}");
        std::io::Error::other("Configuration error")
    })?;

    let app_state = AppState::new(app_config.clone(), observability::AppMetrics::default());

    let db_pool = connect_database_with_config(&app_config)
        .await
        .map_err(|_| {
            eprintln!("Database connection error");
            std::io::Error::other("Database connection error")
        })?;
    println!("Database connection successful");

    let redis_pool = RedisCache::new_with_config(&app_config)
        .await
        .map_err(|_| {
            eprintln!("Redis connection error");
            std::io::Error::other("Redis connection error")
        })?;
    println!("Redis connection successful");

    let user_repo = UserRepositoryPg::new(db_pool.clone());
    let friend_repo = FriendRepositoryPg::new(db_pool.clone());
    let presence_service = PresenceService::new(redis_pool.get_pool().clone());
    let participant_repo = ParticipantPgRepository::default();
    let message_repo = MessageRepositoryPg::new(db_pool.clone());
    let conversation_repo =
        ConversationPgRepository::new(db_pool.clone(), participant_repo.clone());
    let last_message_repo = LastMessagePgRepository::default();
    let file_repo = FilePgRepository::new(db_pool.clone());
    let ws_server = Arc::new(WebSocketServer::with_metrics(app_state.metrics.clone()));
    let user_service = UserService::with_dependencies_and_config(
        Arc::new(user_repo.clone()),
        Arc::new(redis_pool.clone()),
        app_state.config.clone(),
    );
    let friend_service = FriendService::with_dependencies(
        Arc::new(friend_repo.clone()),
        Arc::new(user_repo.clone()),
    );
    let file_upload_service = FileUploadService::with_defaults_and_settings(
        Arc::new(file_repo),
        app_state.metrics.clone(),
        app_state.config.clone(),
    );
    let conversation_service = ConversationService::with_dependencies_and_metrics(
        Arc::new(conversation_repo.clone()),
        Arc::new(participant_repo.clone()),
        Arc::new(message_repo.clone()),
        ws_server.clone(),
        app_state.metrics.clone(),
    );
    let message_service = MessageService::with_dependencies_and_metrics(
        Arc::new(conversation_repo.clone()),
        Arc::new(message_repo),
        Arc::new(participant_repo),
        Arc::new(last_message_repo),
        Arc::new(redis_pool),
        ws_server.clone(),
        app_state.metrics.clone(),
    );

    // Call module
    let call_repo = Arc::new(CallPgRepository::new(db_pool.clone()));
    let call_participant_repo = Arc::new(CallParticipantPgRepository::new(db_pool.clone()));
    let call_service = Arc::new(CallService::with_dependencies_and_metrics(
        call_repo.clone(),
        call_participant_repo.clone(),
        ws_server.clone(),
        app_state.metrics.clone(),
    ));
    let call_handler = Arc::new(CallHandler::new(
        call_service.clone(),
        Arc::new(user_repo.clone()),
    ));

    tracing::info!(
        "Starting HTTP server at http://{}:{}",
        app_state.config.ip.as_str(),
        app_state.config.port
    );

    let bind_ip = app_state.config.ip.clone();
    let bind_port = app_state.config.port;
    let app_state_data = app_state.clone();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin(&app_state_data.config.frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(from_fn(middlewares::request_context))
            .app_data(web::Data::new(app_state_data.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(friend_service.clone()))
            .app_data(web::Data::new(file_upload_service.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(conversation_service.clone()))
            .app_data(web::Data::new(message_service.clone()))
            .app_data(web::Data::new(ws_server.clone())) // WebSocket server
            .app_data(web::Data::new(presence_service.clone())) // Presence service
            .app_data(web::Data::new(friend_repo.clone())) // Friend repo for WS presence
            .app_data(web::Data::new(call_handler.clone())) // Call handler
            .service(health_check)
            .service(metrics)
            .service(metrics_json)
            .service(Files::new("/uploads", "./uploads").prefer_utf8(true))
            // WebSocket endpoint (không cần authentication - auth trong WS handshake)
            .route("/ws", web::get().to(websocket_handler))
            .service(
                web::scope("/api")
                    .default_service(
                        web::route()
                            .guard(actix_web::guard::Method(actix_web::http::Method::OPTIONS))
                            .to(|| async { actix_web::HttpResponse::Ok().finish() }),
                    )
                    .configure(modules::user::route::public_api_configure)
                    .service(
                        web::scope("")
                            .wrap(from_fn(authorization(vec![UserRole::User])))
                            .wrap(from_fn(authentication))
                            .configure(modules::user::route::configure)
                            .configure(modules::friend::route::configure)
                            .configure(modules::conversation::route::configure)
                            .configure(modules::message::route::configure)
                            .configure(modules::file_upload::route::configure::<FilePgRepository>)
                            .configure(modules::call::route::configure),
                    ),
            )
    })
    .bind((bind_ip.as_str(), bind_port))?
    .run()
    .await
}
