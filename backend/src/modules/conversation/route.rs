use actix_web::{Scope, web};

use crate::modules::conversation::handle::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(conversation_scope());
}

fn conversation_scope() -> Scope {
    web::scope("/conversations")
        .service(get_conversations)
        .service(get_messages)
        .service(mark_as_seen)
        .service(update_group)
        .service(add_member)
        .service(remove_member)
        .service(create_conversation)
}
