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
    /// Indicates whether this repository supports explicit SQL transactions.
    ///
    /// Test doubles can override this to `false` to use non-transactional flows.
    fn supports_transactions(&self) -> bool {
        true
    }

    /// Returns the underlying Postgres pool used for transaction entry points.
    fn get_pool(&self) -> &sqlx::PgPool;

    /// Creates a new call record without an explicit transaction context.
    async fn create_call(
        &self,
        initiator_id: Uuid,
        conversation_id: Uuid,
        call_type: CallType,
    ) -> Result<CallEntity, error::SystemError>;

    /// Creates a new call record inside an existing transaction.
    async fn create_call_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        initiator_id: Uuid,
        conversation_id: Uuid,
        call_type: CallType,
    ) -> Result<CallEntity, error::SystemError>;

    /// Finds a call by id.
    async fn find_by_id(&self, call_id: Uuid) -> Result<Option<CallEntity>, error::SystemError>;

    /// Updates call status without an explicit transaction.
    async fn update_call_status(
        &self,
        call_id: Uuid,
        status: CallStatus,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    /// Updates call status inside an existing transaction.
    async fn update_call_status_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        call_id: Uuid,
        status: CallStatus,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    /// Marks call as ended and persists duration without an explicit transaction.
    async fn end_call(
        &self,
        call_id: Uuid,
        duration_seconds: i32,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    /// Marks call as ended and persists duration inside an existing transaction.
    async fn end_call_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        call_id: Uuid,
        duration_seconds: i32,
    ) -> Result<Option<CallEntity>, error::SystemError>;

    /// Returns active member ids of a conversation.
    async fn get_conversation_member_ids(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Uuid>, error::SystemError>;

    /// Checks whether a user is an active member of a conversation.
    async fn is_user_in_conversation(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError>;

    /// Returns paginated call history for one user.
    async fn get_user_calls(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<Vec<CallWithDetails>, error::SystemError>;

    /// Persists a call-related message without an explicit transaction.
    async fn create_call_message(
        &self,
        conversation_id: Uuid,
        sender_id: Uuid,
        message_type: MessageType,
        content: Option<String>,
    ) -> Result<(), error::SystemError>;

    /// Persists a call-related message inside an existing transaction.
    async fn create_call_message_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        conversation_id: Uuid,
        sender_id: Uuid,
        message_type: MessageType,
        content: Option<String>,
    ) -> Result<(), error::SystemError>;
}

#[async_trait::async_trait]
pub trait CallParticipantRepository {
    /// Adds a call participant without an explicit transaction.
    async fn add_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<CallParticipantEntity, error::SystemError>;

    /// Adds a call participant inside an existing transaction.
    async fn add_participant_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<CallParticipantEntity, error::SystemError>;

    /// Marks participant as left without an explicit transaction.
    async fn mark_left(&self, call_id: Uuid, user_id: Uuid) -> Result<(), error::SystemError>;

    /// Marks participant as left inside an existing transaction.
    async fn mark_left_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), error::SystemError>;

    /// Checks whether user is a participant of the given call.
    async fn is_call_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError>;
}
