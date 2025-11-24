use std::vec;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::conversation_model::{Conversation, ConversationType, Participant};
use crate::models::message_model::Message;
use crate::services::conversation_service::ConversationService;

#[derive(Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub _type: ConversationType,
    pub name: Option<String>,
    pub participant_ids: Option<Vec<ObjectId>>,
}

pub async fn create_conversation(
    conversation_service: web::Data<ConversationService>,
    req: HttpRequest,
    payload: web::Json<CreateConversationRequest>,
) -> HttpResponse {
    let claims = match req
        .extensions()
        .get::<crate::services::auth_service::Claims>()
    {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Không tìm thấy thông tin người dùng",
            }));
        }
    };

    let user_id = &claims.user_id;

    let _type = &payload._type;
    let name = &payload.name;
    let participant_ids = &payload.participant_ids;

    if (_type == &ConversationType::Group && name.is_none())
        || participant_ids.is_none()
        || participant_ids.as_ref().unwrap().is_empty()
    {
        return HttpResponse::BadRequest().json(json!({
            "error": "Dữ liệu không hợp lệ",
        }));
    }

    let participant_ids = participant_ids.as_ref().unwrap();
    let mut conversation: Option<Conversation> = None;

    if _type == &ConversationType::Direct {
        let participant_id = &participant_ids[0];
        conversation = match conversation_service
            .find_with_participant(&user_id, participant_id)
            .await
        {
            Ok(Some(c)) => Some(c),
            Ok(None) => {
                let new_conversation =
                    Conversation::new(ConversationType::Direct, &user_id, participant_id);

                match conversation_service.create(&new_conversation).await {
                    Ok(_) => Some(new_conversation),
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(json!({
                            "error": "Lỗi khi tạo cuộc trò chuyện",
                        }));
                    }
                }
            }
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Lỗi khi tìm cuộc trò chuyện",
                }));
            }
        }
    }

    if _type == &ConversationType::Group {
        conversation = Some(Conversation {
            id: Some(ObjectId::new()),
            _type: ConversationType::Group,
            participant_ids: {
                let mut participants = vec![
                    Participant {
                        user_id: user_id.clone(),
                        joined_at: Some(mongodb::bson::DateTime::now()),
                    },
                ];
                for pid in participant_ids {
                    participants.push(Participant {
                        user_id: pid.clone(),
                        joined_at: Some(mongodb::bson::DateTime::now()),
                    });
                }
                participants
            },
            group: Some(crate::models::conversation_model::Group {
                name: name.clone(),
                created_by: Some(user_id.clone()),
            }),
            last_message_at: mongodb::bson::DateTime::now(),
            seen_by: vec![],
            last_message: None,
            unread_counts: std::collections::HashMap::new(),
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
            });

        match conversation_service.create(&conversation.as_ref().unwrap()).await {
            Ok(_) => {}
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Lỗi khi tạo cuộc trò chuyện nhóm",
                }));
            }
        }
    }

    if conversation.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Dữ liệu không hợp lệ",
        }));
    }

    HttpResponse::Ok().json(conversation.unwrap())
}

pub async fn get_conversations(req: HttpRequest) -> HttpResponse {
    // Implementation for retrieving conversations
    HttpResponse::Ok().finish()
}

pub async fn get_messages(req: HttpRequest) -> HttpResponse {
    // Implementation for retrieving messages in a conversation
    HttpResponse::Ok().finish()
}
