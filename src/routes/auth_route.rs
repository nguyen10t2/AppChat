use actix_web::{middleware::from_fn, web};

use crate::{
    controllers::auth_controller::*,
    middlewares::auth_middleware::{verify_jwt, verify_refresh_token},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/verify-otp", web::post().to(verify_otp))
            .route("/resend-otp", web::post().to(resend_otp))
            .route("/forget-password", web::post().to(forget_password))
            .route("/reset-password", web::post().to(reset_password))
            // middleware chỉ áp dụng cho /change-password
            .service(
                web::scope("/change-password")
                    .wrap(from_fn(verify_jwt))
                    .route("", web::post().to(change_password)),
            )
            .service(
                web::scope("/refresh")
                    .wrap(from_fn(verify_refresh_token))
                    .route("", web::post().to(refresh_token)),
            ),
    );
}
