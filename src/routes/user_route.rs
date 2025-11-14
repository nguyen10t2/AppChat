use actix_web::{middleware::from_fn, web};

use crate::{controllers::user_controller::*, middlewares::auth_middleware::verify_jwt};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/users")
        .wrap(from_fn(verify_jwt))
        .route("/me", web::get().to(get_user_profile))
    );
}