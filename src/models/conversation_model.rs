use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationType {
    #[serde(rename = "direct")]
    Direct,
    #[serde(rename = "group")]
    Group,
}

use mongodb::bson::Bson;
use std::convert::From;

impl From<ConversationType> for Bson {
    fn from(value: ConversationType) -> Self {
        let s = serde_json::to_string(&value).expect("Lỗi");
        Bson::String(s.trim_matches('"').to_string())
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Participant {
    pub user_id: ObjectId,
    pub joined_at: Option<BsonDateTime>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: Option<String>,
    pub created_by: Option<ObjectId>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct LastMessage {
    pub _id: ObjectId,
    #[serde(default = "Option::default")]
    pub content: Option<String>,
    pub sender_id: Option<ObjectId>,
    pub created_at: BsonDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Conversation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub _type: ConversationType,
    pub participant_ids: Vec<Participant>,
    pub group: Option<Group>,
    pub last_message_at: BsonDateTime,
    pub seen_by: Vec<ObjectId>,
    pub last_message: Option<LastMessage>,
    #[serde(default)]
    pub unread_counts: HashMap<ObjectId, i32>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

impl Conversation {
    pub fn new(
        _type: ConversationType,
        sender_id: &ObjectId,
        recipient_id: &ObjectId,
    ) -> Self {
        Conversation {
            id: Some(ObjectId::new()),
            _type,
            participant_ids: vec![
                Participant {
                    user_id: sender_id.clone(),
                    joined_at: Some(BsonDateTime::now()),
                },
                Participant {
                    user_id: recipient_id.clone(),
                    joined_at: Some(BsonDateTime::now()),
                },
            ],
            group: None,
            last_message_at: BsonDateTime::now(),
            seen_by: vec![],
            last_message: None,
            unread_counts: HashMap::new(),
            created_at: BsonDateTime::now(),
            updated_at: BsonDateTime::now(),
        }
    }
}