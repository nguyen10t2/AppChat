use serde::{Serialize, Deserialize};
use mongodb::bson::{DateTime as BsonDateTime, oid::ObjectId};

#[derive(Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>,
    pub email: String,
    pub refresh_token: String,
    pub expires_at: BsonDateTime,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}