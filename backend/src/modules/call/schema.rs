use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "call_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CallType {
    Audio,
    Video,
}

impl CallType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "call_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Initiated,
    Accepted,
    Rejected,
    Ended,
    Missed,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CallEntity {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiator_id: Uuid,
    #[sqlx(rename = "_type")]
    pub call_type: CallType,
    pub status: CallStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CallParticipantEntity {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
}
