use actix_web::{middleware::from_fn, web};

use crate::{controllers::auth_controller::*, middlewares::auth_middleware::verify_refresh_token};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/verify-otp", web::post().to(verify_otp))
            .route("/resend-otp", web::post().to(resend_otp))
            .route("/refresh",
                web::post().wrap(from_fn(verify_refresh_token))
                .to(refresh_token)
            )
    );
}