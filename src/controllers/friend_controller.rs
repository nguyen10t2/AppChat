use actix_web::{HttpResponse, HttpRequest, HttpMessage, web};
use serde_json::json;
use serde::Deserialize;
use mongodb::bson::oid::ObjectId;

use crate::services::friend_service::FriendService;
use crate::models::friend_model::Friend;
use crate::models::friend_request_model::FriendRequest;
use crate::services::user_service::{self, UserService};

pub struct FriendRequestParams {
    pub to_user_id: ObjectId,
    pub message: Option<String>,
}

pub async fn send_friend_request(
    user_service: web::Data<UserService>,
    req: HttpRequest,
    params: web::Json<FriendRequestParams>
) -> HttpResponse {
    let to_user_id = &params.to_user_id;
    let message = &params.message;
    
    let extensions = req.extensions();
    let claims = match extensions.get::<crate::services::auth_service::Claims>() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(json!({
            "error": "Không tìm thấy thông tin người dùng",
        }));
        }
    };

    let from_user_id = &claims.user_id;
    let email = &claims.email;
    if to_user_id == from_user_id {
        return HttpResponse::BadRequest().json(json!({"message": "Không thể gửi lời mời kết bạn cho chính mình"}));
    }

    match user_service.is_exists(&email).await {
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

    

    HttpResponse::Ok().json(json!({"message": "Gửi lời mời kết bạn thành công"}))
}

pub async fn accept_friend_request(req: HttpRequest, params: web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "not_implemented"}))
}

pub async fn decline_friend_request(req: HttpRequest, params: web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "not_implemented"}))
}

pub async fn list_friends(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "not_implemented"}))
}

pub async fn list_friend_requests(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "not_implemented"}))
}