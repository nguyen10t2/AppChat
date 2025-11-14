use actix_web::{
    Error, HttpMessage, web,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next
};
use serde_json::json;
use crate::services::auth_service::AuthService;

pub async fn verify_jwt<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {

    let auth_header = req
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
        .verify_token(&auth_header)
        .await
        .map_err(|_| ErrorUnauthorized(json!({"error": "Token không hợp lệ"})))?;

    req.extensions_mut().insert(claims);

    next.call(req).await
}
