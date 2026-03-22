use actix_web::{Scope, web};

use crate::modules::friend::handle::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(friend_scope());
}

fn friend_scope() -> Scope {
    web::scope("/friends")
        .service(send_friend_request)
        .service(accept_friend_request)
        .service(decline_friend_request)
        .service(list_friends)
        .service(list_friend_requests)
        .service(remove_friend)
}
