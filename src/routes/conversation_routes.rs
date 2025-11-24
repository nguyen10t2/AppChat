use actix_web::web;
use actix_web::middleware::from_fn;

use crate::middlewares::auth_middleware::verify_jwt;
use crate::controllers::conversation_controller::*;


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/conversations")
            .wrap(from_fn(verify_jwt))
            .route("/", web::get().to(get_conversations))
            .route("/", web::post().to(create_conversation))
            .route("/{id}/messages", web::get().to(get_messages))
    );
}