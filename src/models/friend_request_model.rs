use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FriendRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub from: ObjectId,
    pub to: ObjectId,
    pub message: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

impl FriendRequest {
    pub fn new(from: ObjectId, to: ObjectId, message: Option<String>) -> Self {
        let now = BsonDateTime::now();
        FriendRequest {
            id: None,
            from,
            to,
            message,
            created_at: now,
            updated_at: now,
        }
    }
}