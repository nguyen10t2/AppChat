use actix_web::{web, Scope};

use crate::modules::call::{
    handler::{cancel_call, end_call, get_call_history, initiate_call, respond_call},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(call_scope());
}

fn call_scope() -> Scope {
    web::scope("/calls")
        .route("", web::post().to(initiate_call))
    .route("/", web::post().to(initiate_call))
        .route("/{call_id}/respond", web::post().to(respond_call))
        .route("/{call_id}/cancel", web::post().to(cancel_call))
        .route("/{call_id}/end", web::post().to(end_call))
        .route("/history", web::get().to(get_call_history))
}
