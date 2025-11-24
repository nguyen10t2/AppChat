use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use mongodb::bson::oid::ObjectId;
use serde_json::json;

use crate::models::friend_model::{Friend, FriendPreview};
use crate::models::friend_request_model::FriendRequest;
use crate::services::friend_request_service::FriendRequestService;
use crate::services::friend_service::FriendService;
use crate::services::user_service::UserService;

#[derive(serde::Deserialize)]
pub struct FriendRequestParams {
    pub to_user_id: ObjectId,
    pub message: Option<String>,
}

pub async fn send_friend_request(
    user_service: web::Data<UserService>,
    friend_request_service: web::Data<FriendRequestService>,
    friend_service: web::Data<FriendService>,
    req: HttpRequest,
    params: web::Json<FriendRequestParams>,
) -> HttpResponse {
    let to_user_id = &params.to_user_id;
    let message = &params.message;

    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };

    let from_user_id = &claims.user_id;
    if to_user_id == from_user_id {
        return HttpResponse::BadRequest()
            .json(json!({"message": "Không thể gửi lời mời kết bạn cho chính mình"}));
    }

    match user_service.find_by_id(to_user_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(json!({"message": "Người dùng đích không tồn tại"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi tìm người dùng đích: {}", e)}));
        }
    }

    let (already_friend, existing_request) = tokio::join!(
        friend_service.find_one(from_user_id, to_user_id),
        friend_request_service.find_one(from_user_id, to_user_id),
    );

    if let Ok(Some(_)) = already_friend {
        return HttpResponse::BadRequest().json(json!({"message": "Hai người đã là bạn bè"}));
    }

    if let Ok(Some(req)) = existing_request {
        if &req.from == from_user_id {
            return HttpResponse::BadRequest()
                .json(json!({"message": "Lời mời kết bạn đã được gửi trước đó"}));
        } else {
            return HttpResponse::BadRequest()
                .json(json!({"message": "Bạn đã nhận được lời mời kết bạn từ người này"}));
        }
    }

    let new_request = FriendRequest::new(from_user_id.clone(), to_user_id.clone(), message.clone());
    match friend_request_service.create(&new_request).await {
        Ok(_) => HttpResponse::Created()
            .json(json!({"message": "Gửi lời mời kết bạn thành công", "new_request": new_request})),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({"error": format!("Lỗi khi gửi lời mời kết bạn: {}", e)})),
    }
}

pub async fn accept_friend_request(
    user_service: web::Data<UserService>,
    friend_request_service: web::Data<FriendRequestService>,
    friend_service: web::Data<FriendService>,
    req: HttpRequest,
    params: web::Path<String>,
) -> HttpResponse {
    let request_id = match ObjectId::parse_str(&params.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(json!({"message": "ID yêu cầu kết bạn không hợp lệ"}));
        }
    };

    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };

    let user_id = &claims.user_id;
    println!("User ID: {}", user_id);
    let request = match friend_request_service.find_by_id_from_request(&request_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(json!({"message": "Yêu cầu kết bạn không tồn tại"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi tìm yêu cầu kết bạn: {}", e)}));
        }
    };
    
    if &request.to != user_id {
        return HttpResponse::Forbidden()
            .json(json!({"message": "Không có quyền"}));
    }

    let new_friend = Friend::new(request.from.clone(), request.to.clone());
    if let Err(e) = friend_service.create(&new_friend).await {
        return HttpResponse::InternalServerError()
            .json(json!({"error": format!("Lỗi khi tạo kết bạn: {}", e)}));
    }
    
    let (delete_result, from_user) = tokio::join!(
        friend_request_service.delete_by_id(&request_id),
        user_service.find_by_id_preview(&request.from),
    );

    if let Err(e) = delete_result {
        return HttpResponse::InternalServerError()
            .json(json!({"error": format!("Lỗi khi xóa yêu cầu kết bạn: {}", e)}));
    }

    let from_user = match from_user {
        Ok(Some(u)) => u,
        _ => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Lỗi khi lấy thông tin người gửi yêu cầu"}));
        }
    };

    HttpResponse::Ok().json(json!({
        "message": "Chấp nhận lời mời kết bạn thành công",
        "new_friend": {
            "user_id": from_user.id,
            "fullname": from_user.fullname,
            "avatar_url": from_user.avatar_url,
        }
    }))
}

pub async fn decline_friend_request(    
    friend_request_service: web::Data<FriendRequestService>,
    req: HttpRequest,
    params: web::Path<String>,
) -> HttpResponse {
    let request_id = match ObjectId::parse_str(&params.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(json!({"message": "ID yêu cầu kết bạn không hợp lệ"}));
        }
    };

    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };

    let user_id = &claims.user_id;
    
    let request = match friend_request_service.find_by_id_from_request(&request_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(json!({"message": "Yêu cầu kết bạn không tồn tại"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi tìm yêu cầu kết bạn: {}", e)}));
        }
    };

    if &request.to != user_id {
        return HttpResponse::Forbidden()
            .json(json!({"message": "Không có quyền"}));
    }

    if let Err(e) = friend_request_service.delete_by_id(&request_id).await {
        return HttpResponse::InternalServerError()
            .json(json!({"error": format!("Lỗi khi xóa yêu cầu kết bạn: {}", e)}));
    }

    // 204
    HttpResponse::NoContent().finish()
}

pub async fn list_friends(
    friend_service: web::Data<FriendService>,
    req: HttpRequest
) -> HttpResponse {
    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };
    let user_id = &claims.user_id;

    let friendships = match friend_service.get_friendships(user_id).await {
        Ok(fs) => fs,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi lấy danh sách bạn bè: {}", e)}));
        }
    };
    
    let friends: Vec<FriendPreview> = friendships.into_iter().map(|f| {
        if &f.user_a.id == user_id {
            f.user_b
        } else {
            f.user_a
        }
    }).collect();

    HttpResponse::Ok().json(json!({"friends": friends}))
}

pub async fn list_friend_requests(
    friend_request_service: web::Data<FriendRequestService>,
    req: HttpRequest
) -> HttpResponse {
    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };
    let user_id = &claims.user_id;
    
    let (sent, received) = tokio::join!(
        friend_request_service.find_by_id_from_request(&user_id),
        friend_request_service.find_by_id_to_request(&user_id),
    );

    let send_requests = match sent {
        Ok(reqs) => reqs,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi lấy danh sách lời mời kết bạn đã gửi: {}", e)}));
        }
    };

    let received_requests = match received {
        Ok(reqs) => reqs,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi lấy danh sách lời mời kết bạn đã nhận: {}", e)}));
        }
    };

    HttpResponse::Ok().json(json!({
        "sent_requests": send_requests,
        "received_requests": received_requests,
    }))
}
