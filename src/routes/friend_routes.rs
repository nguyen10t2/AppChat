use actix_web::web;

use crate::controllers::friend_controller::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/friends")
            .route("/requests", web::post().to(send_friend_request))
            .route("/request/{id}/accept", web::post().to(accept_friend_request))
            .route("/request/{id}/decline", web::post().to(decline_friend_request))
            .route("/", web::get().to(list_friends))
            .route("/requests", web::get().to(list_friend_requests)),
    );
}