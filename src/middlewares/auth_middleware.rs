use actix_web::{
    Error, HttpMessage, web,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next
};
use serde_json::json;
use crate::services::{auth_service::AuthService, session_service::SessionService};

pub async fn verify_jwt<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|header| {
            header.strip_prefix("Bearer ").map(|s| s.to_string())
        })
        .ok_or_else(|| ErrorUnauthorized(json!({"error": "Không tìm thấy access token"})))?;

    let srv = req
        .app_data::<web::Data<AuthService>>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized(json!({"error": "Không tìm thấy dịch vụ xác thực"})))?;

    let claims = srv
        .verify_token(&token)
        .await
        .map_err(|_| ErrorUnauthorized(json!({"error": "Token không hợp lệ"})))?;

    req.extensions_mut().insert(claims);

    next.call(req).await
}

#[allow(dead_code)]
pub async fn verify_refresh_token<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    let refresh_token = req.cookie("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| ErrorUnauthorized(json!({"error": "Không tìm thấy refresh token"})))?;

    let srv = req
        .app_data::<web::Data<SessionService>>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized(json!({"error": "Không tìm thấy dịch vụ xác thực"})))?;

    let sesstion = srv.find_one(&refresh_token).await
        .map_err(|_| ErrorUnauthorized(json!({"error": "Lỗi khi truy xuất phiên"})))?
        .ok_or_else(|| ErrorUnauthorized(json!({"error": "Refresh token không hợp lệ"})))?;

    if sesstion.expires_at.to_system_time() < std::time::SystemTime::now() {
        return Err(ErrorUnauthorized(json!({"error": "Refresh token đã hết hạn"})));
    }

    req.extensions_mut().insert(sesstion);
    next.call(req).await
}