use actix_web::{HttpResponse, HttpRequest, HttpMessage, web};
use serde_json::json;

use crate::{models::user_model::UserResponse, services::{auth_service::Claims, user_service::UserService}};

pub async fn get_user_profile(
    req: HttpRequest,
    user_service: web::Data<UserService>,
) -> HttpResponse {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let user_result = user_service.find_by_email(&claims.email).await;
        
        match user_result {
            Ok(Some(user)) => {
                let user_response = UserResponse::from(user);
                HttpResponse::Ok().json(user_response)
            }
            Ok(None) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Không tìm thấy người dùng",
                }))
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": format!("Không thể truy xuất người dùng: {}", e),
                }))
            }
        }
    } else {
        HttpResponse::Unauthorized().json(json!({
            "error": "Không tìm thấy thông tin người dùng",
        }))
    }
}