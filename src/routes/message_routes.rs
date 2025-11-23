use actix_web::{middleware::from_fn, web};

use crate::controllers::message_controller::*;
use crate::middlewares::auth_middleware::verify_jwt;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/messages")
        .wrap(from_fn(verify_jwt))
            .route("/direct", web::post().to(send_direc_message))
            .route("/group", web::post().to(send_group_message)),
    );
}