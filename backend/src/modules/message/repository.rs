use crate::modules::message::model::{InsertMessage, MessageQuery};
use crate::{api::error, modules::message::schema::MessageEntity};

#[async_trait::async_trait]
pub trait MessageRepository {
    /// Returns Postgres pool for service-level transaction entry.
    fn get_pool(&self) -> &sqlx::PgPool;

    /// Finds one non-deleted message by id.
    async fn find_by_id<'e, E>(
        &self,
        message_id: &uuid::Uuid,
        tx: E,
    ) -> Result<Option<MessageEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Persists a new message record.
    async fn create<'e, E>(
        &self,
        message: &InsertMessage,
        tx: E,
    ) -> Result<MessageEntity, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Returns paginated messages for one conversation.
    async fn find_by_query<'e, E>(
        &self,
        query: &MessageQuery,
        limit: i32,
        tx: E,
    ) -> Result<Vec<MessageEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Delete a message by ID (soft delete)
    async fn delete_message<'e, E>(
        &self,
        message_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        tx: E,
    ) -> Result<bool, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Edit a message by ID (only content can be edited)
    async fn edit_message<'e, E>(
        &self,
        message_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        new_content: &str,
        tx: E,
    ) -> Result<Option<MessageEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Get the last message of a conversation
    async fn get_last_message_by_conversation<'e, E>(
        &self,
        conversation_id: &uuid::Uuid,
        tx: E,
    ) -> Result<Option<MessageEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;
}
