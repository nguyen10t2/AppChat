use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub fullname: String,
    pub email: String,
    pub password: String,
    pub avatar_url: Option<String>,
    pub avatar_id: Option<String>,
    pub bio: Option<String>,
    pub phone: Option<String>,
    #[serde(default)]
    pub is_active: bool,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub fullname: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub avatar_id: Option<String>,
    pub bio: Option<String>,
    pub phone: Option<String>,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreview {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub fullname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[allow(dead_code)]
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            fullname: user.fullname,
            email: user.email,
            avatar_url: user.avatar_url,
            avatar_id: user.avatar_id,
            bio: user.bio,
            phone: user.phone,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
