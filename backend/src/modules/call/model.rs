use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::modules::call::schema::{CallStatus, CallType};

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct InitiateCallRequest {
    pub conversation_id: Uuid,
    pub call_type: CallType,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitiateCallResponse {
    pub call_id: Uuid,
    pub status: CallStatus,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RespondCallRequest {
    pub accept: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallHistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub cursor: Option<String>,
}

const fn default_limit() -> i64 {
    20
}

#[derive(Debug, Clone, Serialize)]
pub struct CallHistoryResponse {
    pub calls: Vec<CallWithDetails>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CallWithDetails {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiator_id: Uuid,
    pub initiator_name: String,
    pub initiator_avatar: Option<String>,
    pub call_type: CallType,
    pub status: CallStatus,
    pub duration_seconds: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
