use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum ConversationType {
    Single,
    Group,
}

#[derive(Serialize, Deserialize)]
pub struct Participant {
    pub user_id: ObjectId,
    pub joined_at: BsonDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: Option<String>,
    pub created_by: Option<ObjectId>,
}

#[derive(Serialize, Deserialize)]
pub struct LastMessage {
    pub _id: String,
    #[serde(default = "Option::default")]
    pub content: Option<String>,
    pub sender_id: Option<ObjectId>,
    #[serde(default = "Option::default")]
    pub created_at: Option<BsonDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct Conversation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub _type: ConversationType,
    pub participant_ids: Vec<Participant>,
    pub group: Option<Group>,
    pub last_message_at: Option<BsonDateTime>,
    pub seen_by: Option<Vec<ObjectId>>,
    #[serde(default = "Option::default")]
    pub last_message: Option<LastMessage>,
    #[serde(default)]
    pub unread_counts: HashMap<String, i32>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}