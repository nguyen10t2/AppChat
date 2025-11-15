use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::Deserialize;

use crate::libs::hash::verify_password;
use crate::libs::otp::OtpCode;
use crate::libs::reset_token::ResetToken;
use crate::models::session_model::Session;
use crate::models::user_model::UserResponse;
use crate::services::auth_service::{AuthService, Claims};
use crate::services::email_service::EmailService;
use crate::services::otp_service::OtpService;
use crate::services::reset_token_service::ResetTokenService;
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

#[derive(Deserialize)]
pub struct EmailRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPassword {
    pub email: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}

pub async fn register(
    user_service: web::Data<UserService>,
    otp_service: web::Data<OtpService>,
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

    if let Ok(Some(_)) = user_service.find_by_email(&req.email).await {
        return HttpResponse::Conflict().json(json!({
            "error": "Email đã được sử dụng"
        }));
    }

    if let Err(e) = user_service
        .create_user(&req.fullname, &req.email, &req.password)
        .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi tạo người dùng: {}", e)
        }));
    }

    let otp_code = OtpCode::new();

    let email_owned = req.email.clone();
    let plain_otp_owned = otp_code.plain_otp.clone();

    if let Err(e) = otp_service
        .create_otp(&req.email, &otp_code.hashed_otp, otp_code.expires_at)
        .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi tạo mã OTP: {}", e)
        }));
    }

    actix_web::rt::spawn(async move {
        if let Err(e) = EmailService::new()
            .send_otp_email(&email_owned, &plain_otp_owned)
            .await
        {
            eprintln!("Lỗi khi gửi email OTP: {}", e);
        }
    });

    HttpResponse::Ok().json(json!({
        "message": "Đăng ký thành công. Vui lòng kiểm tra email để xác thực OTP."
    }))
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

    if !verify_password(&user.password, &req.password)
        .unwrap_or(false)
    {
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
        .path("/api/auth")
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
            .path("/api/auth")
            .finish();
        HttpResponse::Ok().cookie(expired_cookie).json(json!({
            "message": "Đăng xuất thành công"
        }))
    } else {
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
    } else {
        HttpResponse::Unauthorized().json(json!({
            "error": "Phiên làm việc không hợp lệ"
        }))
    }
}

pub async fn verify_otp(
    user_service: web::Data<UserService>,
    otp_service: web::Data<OtpService>,
    req: web::Json<VerifyOtpRequest>,
) -> HttpResponse {
    let email = &req.email;
    let otp = &req.otp;
    if !validation_email(email) || !validation_otp(otp) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Thông tin không hợp lệ"
        }));
    }
    match user_service.is_exists(email).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Người dùng không tồn tại"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi kiểm tra người dùng: {}", e)
            }));
        }
    };

    let otp_record = match otp_service.get_otp_record(email).await {
        Ok(Some(otp_record)) => otp_record,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Mã OTP không hợp lệ hoặc đã được sử dụng"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi lấy mã OTP: {}", e)
            }));
        }
    };

    let stored_otp = &otp_record.code;
    let expires_at = &otp_record.expires_at;
    let now = mongodb::bson::DateTime::from_system_time(chrono::Utc::now().into());
    if &now > expires_at {
        return HttpResponse::BadRequest().json(json!({
            "error": "Mã OTP đã hết hạn"
        }));
    }

    match verify_password(stored_otp, otp) {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Mã OTP không đúng"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi xác thực mã OTP: {}", e)
            }));
        }
    };

    let (user_res, otp_res) = tokio::join!(
        user_service.activate_user(email),
        otp_service.updated_otp(email)
    );

    if let Err(e) = user_res {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi kích hoạt người dùng: {}", e)
        }));
    }
    if let Err(e) = otp_res {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi cập nhật mã OTP: {}", e)
        }));
    }

    HttpResponse::Ok().json(json!({
        "message": "Xác thực OTP thành công"
    }))
}

pub async fn resend_otp(
    otp_service: web::Data<OtpService>,
    user_service: web::Data<UserService>,
    req: web::Json<EmailRequest>,
) -> HttpResponse {
    let email = req.email.clone();
    if !validation_email(&email) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Email không hợp lệ"
        }));
    }
    match user_service.is_exists(&email.clone()).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Người dùng không tồn tại"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi kiểm tra người dùng: {}", e)
            }));
        }
    };

    let last_otp = match otp_service.get_last_otp(&email).await {
        Ok(Some(otp)) => otp,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Không tìm thấy mã OTP trước đó"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi lấy mã OTP: {}", e)
            }));
        }
    };

    let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
        last_otp.created_at.timestamp_millis(),
    )
    .expect("Invalid timestamp");

    let elapsed = chrono::Utc::now() - created_at;

    if elapsed.num_seconds() < 30 {
        return HttpResponse::TooManyRequests().json(json!({
            "error": "Vui lòng chờ trước khi yêu cầu mã OTP mới",
            "retry_after": 30 - elapsed.num_seconds()
        }));
    }

    let resend_count = match otp_service.resend_count(&email).await {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi lấy số lần gửi lại mã OTP: {}", e)
            }));
        }
    };
    if resend_count >= 5 {
        return HttpResponse::TooManyRequests().json(json!({
            "error": "Đã vượt quá số lần gửi lại mã OTP trong ngày"
        }));
    }

    let otp_code = OtpCode::new();

    if let Err(e) = otp_service
        .create_otp(&email, &otp_code.hashed_otp, otp_code.expires_at)
        .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Lỗi khi tạo mã OTP: {}", e)
        }));
    }

    actix_web::rt::spawn(async move {
        if let Err(e) = EmailService::new()
            .send_otp_email(&email, &otp_code.plain_otp)
            .await
        {
            eprintln!("Lỗi khi gửi email OTP: {}", e);
        }
    });
    HttpResponse::Ok().json(json!({
        "message": "Gửi lại mã OTP thành công. Vui lòng kiểm tra email."
    }))
}

pub async fn forget_password(
    otp_service: web::Data<OtpService>,
    reset_token_service: web::Data<ResetTokenService>,
    req: web::Json<EmailRequest>,
) -> HttpResponse {
    let email = req.email.clone();
    if !validation_email(&email) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Email không hợp lệ"
        }));
    }
    let otp_code = OtpCode::new();
    match otp_service
        .create_otp(&email, &otp_code.hashed_otp, otp_code.expires_at)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi tạo mã OTP: {}", e)
            }));
        }
    };
    
    let reset_token = ResetToken::new();
    match reset_token_service
        .create_reset_token(
            &email,
            &reset_token.hashed_token,
            reset_token.expires_at,
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi tạo reset token: {}", e)
            }));
        }
    };
    let reset_token_cookie = Cookie::build("reset_token", reset_token.hashed_token.clone())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::minutes(30))
        .path("/api/auth")
        .finish();

    actix_web::rt::spawn(async move {
        if let Err(e) = EmailService::new()
            .send_otp_email(&email.clone(), &otp_code.plain_otp)
            .await
        {
            eprintln!("Lỗi khi gửi email đặt lại mật khẩu: {}", e);
        }
    });

    HttpResponse::Ok().cookie(reset_token_cookie).json(json!({
        "message": "Yêu cầu đặt lại mật khẩu thành công. Vui lòng kiểm tra email."
    }))
}

pub async fn reset_password(
    reset_token_service: web::Data<ResetTokenService>,
    user_service: web::Data<UserService>,
    body: web::Json<ResetPassword>,
    req: HttpRequest,
) -> HttpResponse {
    let email = body.email.clone();
    let new_password = body.new_password.clone();
    if !validation_email(&email) || !validation_password(&new_password) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Thông tin không hợp lệ"
        }));
    }
    let reset_token = match req.cookie("reset_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Không tìm thấy reset token"
            }));
        }
    };
    let reset_token_record = match reset_token_service
        .find_one(&email)
        .await
    {
        Ok(Some(record)) => {
            if record.expires_at.to_system_time() < std::time::SystemTime::now() {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Reset token đã hết hạn"
                }));
            }
            record
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Reset token không hợp lệ hoặc đã được sử dụng"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi lấy reset token: {}", e)
            }));
        }
    };
    
    if !verify_password(&reset_token_record.token, &reset_token).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(json!({ "error": "Reset token không đúng" }));
    }

    if let Err(e) = user_service.update_user(&email, &new_password).await {
        return HttpResponse::InternalServerError().json(json!({ "error": format!("Lỗi khi cập nhật mật khẩu: {}", e) }));
    }

    if let Err(e) = reset_token_service.delete_one(&email).await {
        return HttpResponse::InternalServerError().json(json!({ "error": format!("Lỗi khi xóa reset token: {}", e) }));
    }

    let clear_cookie = Cookie::build("reset_token", "")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::seconds(0))
        .path("/api/auth")
        .finish();

    HttpResponse::Ok().cookie(clear_cookie).json(json!({
        "message": "Đặt lại mật khẩu thành công."
    }))
}

// use middleware to get user info from token

pub async fn change_password(
    user_service: web::Data<UserService>,
    body: web::Json<ChangePassword>,
    req: HttpRequest
) -> HttpResponse {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let email = &claims.email;
        let user = match user_service.find_by_email(email).await {
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
        if !validation_password(&body.current_password)
            || !validation_password(&body.new_password)
        {
            return HttpResponse::BadRequest().json(json!({
                "error": "Mật khẩu không hợp lệ"
            }));
        }
        if !verify_password(&user.password, &body.current_password)
            .unwrap_or(false)
        {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Mật khẩu hiện tại không đúng"
            }));
        }
        if let Err(e) = user_service.update_user(email, &body.new_password).await {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Lỗi khi cập nhật mật khẩu: {}", e)
            }));
        }
        HttpResponse::Ok().json(json!({
            "message": "Đổi mật khẩu thành công"
        }))
    }
    else {
        HttpResponse::Unauthorized().json(json!({
            "error": "Người dùng không hợp lệ"
        }))
    }
}