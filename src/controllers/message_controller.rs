use actix_web::{HttpRequest, HttpResponse, web};
use mongodb::bson::oid::ObjectId;

use crate::services::conversation_service::ConversationService;

#[derive(serde::Deserialize)]
pub struct SendDirecMessage {
    pub recipient_id: ObjectId,
    pub content: String,
    pub conversation_id: Option<ObjectId>,
}

pub async fn send_direc_message(
    conversation_service: web::Data<ConversationService>,
    req: HttpRequest,
    body: web::Json<SendDirecMessage>,
) -> HttpResponse {
    // Logic to send a direct message
    HttpResponse::Ok().body("Direct message sent")
}

pub async fn send_group_message(
    req: HttpRequest,
) -> HttpResponse {
    // Logic to send a group message
    HttpResponse::Ok().body("Group message sent")
}