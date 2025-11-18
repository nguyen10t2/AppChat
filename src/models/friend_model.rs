use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;

#[derive(Serialize, Deserialize)]
pub struct Friend {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_a_id: ObjectId,
    pub user_b_id: ObjectId,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

impl Friend {
    pub fn new(user_a: ObjectId, user_b: ObjectId) -> Self {
        let now = BsonDateTime::from_system_time(chrono::Utc::now().into());
        let (a, b) = if user_a.to_string() < user_b.to_string() {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        Friend {
            id: None,
            user_a_id: a,
            user_b_id: b,
            created_at: now,
            updated_at: now,
        }
    }
}