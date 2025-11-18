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