use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    api::error,
    modules::{
        call::{
            model::CallWithDetails,
            schema::{CallEntity, CallParticipantEntity, CallStatus, CallType},
        },
        message::schema::MessageType,
    },
};

#[async_trait::async_trait]
pub trait CallRepository {
    fn get_pool(&self) -> &sqlx::PgPool;

    async fn create_call(
        &self,
        initiator_id: Uuid,
        conversation_id: Uuid,
        call_type: CallType,
    ) -> Result<CallEntity, error::SystemError>;

    async fn find_by_id(&self, call_id: Uuid) -> Result<Option<CallEntity>, error::SystemError>;

    async fn update_call_status(
        &self,
        call_id: Uuid,
        status: CallStatus,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    async fn end_call(
        &self,
        call_id: Uuid,
        duration_seconds: i32,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    async fn get_conversation_member_ids(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Uuid>, error::SystemError>;

    async fn is_user_in_conversation(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError>;

    async fn get_user_calls(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<Vec<CallWithDetails>, error::SystemError>;

    async fn create_call_message(
        &self,
        conversation_id: Uuid,
        sender_id: Uuid,
        message_type: MessageType,
        content: Option<String>,
    ) -> Result<(), error::SystemError>;
}

#[async_trait::async_trait]
pub trait CallParticipantRepository {
    async fn add_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<CallParticipantEntity, error::SystemError>;

    async fn mark_left(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), error::SystemError>;

    async fn is_call_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError>;
}
