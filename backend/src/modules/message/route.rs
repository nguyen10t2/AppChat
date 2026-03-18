use actix_web::{
    middleware::from_fn,
    web::{ServiceConfig, scope},
};

use crate::modules::message::handle::*;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/messages")
            .service(scope("/direct").service(send_direct_message))
            .service(scope("/group").service(send_group_message))
            .service(delete_message)
            .service(edit_message),
    );
}
