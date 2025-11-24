use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub coversation_id: ObjectId,
    pub sender_id: ObjectId,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

impl Message {
    pub fn new(
        conversation_id: &ObjectId,
        sender_id: &ObjectId,
        content: Option<String>,
    ) -> Self {
        let now = BsonDateTime::now();
        Message {
            id: Some(ObjectId::new()),
            coversation_id: conversation_id.clone(),
            sender_id: sender_id.clone(),
            content,
            image_url: None,
            created_at: now,
            updated_at: now,
        }
    }
}