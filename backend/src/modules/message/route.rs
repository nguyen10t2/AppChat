use actix_web::{Scope, web};

use crate::modules::message::handle::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(message_scope());
}

fn message_scope() -> Scope {
    web::scope("/messages")
        .service(direct_scope())
        .service(group_scope())
        .service(delete_message)
        .service(edit_message)
}

fn direct_scope() -> Scope {
    web::scope("/direct").service(send_direct_message)
}

fn group_scope() -> Scope {
    web::scope("/group").service(send_group_message)
}
