use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use mongodb::bson::DateTime as BsonDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Otp {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub code: String,
    pub expires_at: BsonDateTime,
    pub is_used: bool,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}