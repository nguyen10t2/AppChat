use actix_web::{HttpRequest, HttpResponse, web, HttpMessage};
use mongodb::bson::oid::ObjectId;
use serde_json::json;

use crate::models::conversation_model::{Conversation, ConversationType};
use crate::models::message_model::Message;
use crate::services::friend_service::{self, FriendService};
use crate::services::{conversation_service::ConversationService, message_service::MessageService};
use crate::helpers::message_helper::update_conversation_after_create_message;
use crate::helpers::friend_helper::verify_friendship;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct SendDirecMessage {
    pub recipient_id: ObjectId,
    pub content: String,
    pub conversation_id: Option<ObjectId>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct SendGroupMessage {
    pub group_id: ObjectId,
    pub content: String,
}

pub async fn send_direct_message(
    friend_service: web::Data<FriendService>,
    message_service: web::Data<MessageService>,
    conversation_service: web::Data<ConversationService>,
    body: web::Json<SendDirecMessage>,
    req: HttpRequest,
) -> HttpResponse {
    let content = &body.content;
    if content.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Nội dung tin nhắn không được để trống"}));
    }
    let conversation_id = &body.conversation_id;
    let recipient_id = &body.recipient_id;

    let claims = match req.extensions().get::<crate::services::auth_service::Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };

    let sender_id = &claims.user_id;

    if let Err(_) = verify_friendship(&friend_service, sender_id, recipient_id).await {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Hai người không phải là bạn bè",
        }));
    }

    let mut conversation = match conversation_id {
        Some(cid) => {
            match conversation_service.find_conversation_by_id(cid).await {
                Ok(Some(conv)) => conv,
                Ok(None) => {
                    return HttpResponse::NotFound()
                        .json(json!({"error": "Cuộc trò chuyện không tồn tại"}));
                }
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": format!("Lỗi khi tìm cuộc trò chuyện: {}", e)}));
                }
            }
        }
        None => {
            let new_conversation = Conversation::new(ConversationType::Direct, sender_id, recipient_id);
            if let Err(e) = conversation_service.create(&new_conversation).await {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": format!("Lỗi khi tạo cuộc trò chuyện: {}", e)}));
            }
            new_conversation
        }
    };

    let new_message = Message::new(&conversation.id.unwrap(), sender_id, Some(content.clone()));
    match message_service.create(&new_message).await {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi gửi tin nhắn: {}", e)}));
        }
    };

    update_conversation_after_create_message(&mut conversation, &new_message, sender_id).await;

    match conversation_service.update(&conversation).await {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Lỗi khi cập nhật cuộc trò chuyện: {}", e)}));
        }
    };

    HttpResponse::Created().json(json!({"message": "Tin nhắn đã được gửi thành công"}))
}

pub async fn send_group_message(
    
) -> HttpResponse {
    // Logic to send a group message
    HttpResponse::Ok().body("Group message sent")
}