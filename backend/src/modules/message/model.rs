use crate::modules::message::schema::MessageEntity;
use crate::modules::message::schema::MessageType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone)]
pub struct InsertMessage {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub reply_to_id: Option<Uuid>,
    pub _type: MessageType,
    pub content: Option<String>,
    pub file_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageQuery {
    pub conversation_id: Uuid,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetMessageResponse {
    pub messages: Vec<MessageEntity>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendDirectMessage {
    pub conversation_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(rename = "type", default)]
    pub _type: Option<MessageType>,
    #[serde(default)]
    pub file_url: Option<String>,
    #[serde(default)]
    pub reply_to_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct SendDirectMessagePayload {
    pub conversation_id: Option<Uuid>,
    pub content: Option<String>,
    pub message_type: Option<MessageType>,
    pub file_url: Option<String>,
    pub reply_to_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendGroupMessage {
    pub conversation_id: Uuid,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(rename = "type", default)]
    pub _type: Option<MessageType>,
    #[serde(default)]
    pub file_url: Option<String>,
    #[serde(default)]
    pub reply_to_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct EditMessageRequest {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "Content must be between 1 and 5000 characters"
    ))]
    pub content: String,
}
