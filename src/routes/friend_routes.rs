use actix_web::{middleware::from_fn, web};

use crate::{controllers::friend_controller::*, middlewares::auth_middleware::verify_jwt};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/friends")
        .wrap(from_fn(verify_jwt))
            .route("/requests", web::post().to(send_friend_request))
            .route("/requests/{id}/accept", web::post().to(accept_friend_request))
            .route("/requests/{id}/decline", web::post().to(decline_friend_request))
            .route("/", web::get().to(list_friends))
            .route("/requests", web::get().to(list_friend_requests)),
    );
}