use actix_web::web::{ServiceConfig, scope};

use crate::modules::conversation::handle::*;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/conversations")
            .service(get_conversations)
            .service(get_messages)
            .service(mark_as_seen)
            .service(update_group)
            .service(add_member)
            .service(remove_member)
            .service(scope("").service(create_conversation)),
    );
}
