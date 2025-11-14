use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::{HttpRequest, HttpResponse, web, HttpMessage};
use serde::Deserialize;

use crate::libs::hash::verify_password;
use crate::models::session_model::Session;
use crate::models::user_model::UserResponse;
use crate::services::auth_service::AuthService;
use crate::services::session_service::SessionService;
use crate::services::user_service::UserService;
use crate::validations::validation::*;
use serde_json::json;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub fullname: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
}

pub async fn register(
    user_service: web::Data<UserService>,
    req: web::Json<RegisterRequest>,
) -> HttpResponse {
    if !validation_fullname(&req.fullname)
        || !validation_email(&req.email)
        || !validation_password(&req.password)
    {
        return HttpResponse::BadRequest().json(json!({
            "error": "Thông tin đăng ký không hợp lệ"
        }));
    }

    match user_service.is_exists(&req.email).await {
        Ok(true) => HttpResponse::Conflict().json(json!({
            "error": "Người dùng đã tồn tại"
        })),
        Ok(false) => {
            match user_service
                .create_user(&req.fullname, &req.email, &req.password)
                .await
            {
                Ok(user) => {
                    let user_response = UserResponse::from(user);
                    HttpResponse::Created().json(user_response)
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "error": format!("Lỗi khi tạo người dùng: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi hệ thống: {}", e)
        })),
    }
}

pub async fn login(
    user_service: web::Data<UserService>,
    auth_service: web::Data<AuthService>,
    session_service: web::Data<SessionService>,
    req: web::Json<LoginRequest>,
) -> HttpResponse {
    if !validation_email(&req.email) || !validation_password(&req.password) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Thông tin đăng nhập không hợp lệ"
        }));
    }

    let user = match user_service.find_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Thông tin đăng nhập không đúng"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi lấy người dùng: {}", e)
            }));
        }
    };

    if !verify_password(&user.password, &req.password).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Thông tin đăng nhập không đúng"
        }));
    }

    let access_token = match auth_service
        .create_access_token(&user.id.unwrap().to_string(), &user.email)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi tạo token: {}", e)
            }));
        }
    };

    let refresh_token = match auth_service
        .create_refresh_token(&user.id.unwrap().to_string(), &user.email)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi tạo refresh token: {}", e)
            }));
        }
    };

    if let Err(e) = session_service
        .create_session(user.id.unwrap(), user.email.clone(), refresh_token.clone())
        .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi tạo phiên làm việc: {}", e)
        }));
    }

    let refresh_token_cookie = Cookie::build("refresh_token", refresh_token.clone())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::seconds(session_service.refresh_token_ttl))
        .finish();

    let user_response = UserResponse::from(user);

    HttpResponse::Ok().cookie(refresh_token_cookie).json(json!({
        "access_token": access_token,
        "user": user_response
    }))
}

pub async fn logout(
    session_service: web::Data<SessionService>,
    req: HttpRequest
) -> HttpResponse {
    let token = req.cookie("refresh_token");
    if let Some(cookie) = token {
        let refresh_token = cookie.value();
        if let Err(e) = session_service.delete_session(refresh_token).await {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi xóa phiên làm việc: {}", e)
            }));
        }
        let expired_cookie = Cookie::build("refresh_token", "")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::None)
            .max_age(time::Duration::seconds(0))
            .finish();
        HttpResponse::Ok().cookie(expired_cookie).json(json!({
            "message": "Đăng xuất thành công"
        }))
    }
    else {
        HttpResponse::BadRequest().json(json!({
            "error": "Không tìm thấy phiên làm việc"
        }))
    }
}

pub async fn refresh_token(
    user_service: web::Data<UserService>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    if let Some(session) = req.extensions().get::<Session>() {
        let user = match user_service.find_by_email(&session.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return HttpResponse::NotFound().json(json!({
                    "error": "Người dùng không tồn tại"
                }));
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Lỗi khi lấy người dùng: {}", e)
                }));
            }
        };

        let access_token = match auth_service
            .create_access_token(&user.id.unwrap().to_string(), &user.email)
            .await
        {
            Ok(token) => token,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Lỗi khi tạo token: {}", e)
                }));
            }
        };
        HttpResponse::Ok().json(json!({
            "access_token": access_token
        }))
    }
    else {
        HttpResponse::Unauthorized().json(json!({
            "error": "Phiên làm việc không hợp lệ"
        }))
    }
}

pub async fn verify_otp(
    user_service: web::Data<UserService>,

) -> HttpResponse {
    HttpResponse::Ok().finish()
}