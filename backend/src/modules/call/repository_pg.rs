use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    api::error,
    modules::{
        call::{
            model::CallWithDetails,
            repository::{CallParticipantRepository, CallRepository},
            schema::{CallEntity, CallParticipantEntity, CallStatus, CallType},
        },
        message::schema::MessageType,
    },
};

#[derive(Clone)]
pub struct CallPgRepository {
    pool: sqlx::PgPool,
}

impl CallPgRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CallRepository for CallPgRepository {
    fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    async fn create_call(
        &self,
        initiator_id: Uuid,
        conversation_id: Uuid,
        call_type: CallType,
    ) -> Result<CallEntity, error::SystemError> {
        let call = sqlx::query_as::<_, CallEntity>(
            r#"
            INSERT INTO calls (conversation_id, initiator_id, _type, status)
            VALUES ($1, $2, $3, 'initiated')
            RETURNING *
            "#,
        )
        .bind(conversation_id)
        .bind(initiator_id)
        .bind(call_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(call)
    }

    async fn find_by_id(&self, call_id: Uuid) -> Result<Option<CallEntity>, error::SystemError> {
        let call =
            sqlx::query_as::<_, CallEntity>("SELECT * FROM calls WHERE id = $1")
                .bind(call_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(call)
    }

    async fn update_call_status(
        &self,
        call_id: Uuid,
        status: CallStatus,
    ) -> Result<Option<CallEntity>, error::SystemError> {
        let call = sqlx::query_as::<_, CallEntity>(
            r#"
            UPDATE calls
            SET
                status = $2,
                started_at = CASE
                    WHEN $2::call_status = 'accepted'::call_status AND started_at IS NULL THEN NOW()
                    ELSE started_at
                END,
                ended_at = CASE
                    WHEN $2::call_status IN ('rejected'::call_status, 'missed'::call_status) AND ended_at IS NULL THEN NOW()
                    ELSE ended_at
                END
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(call_id)
        .bind(status)
        .fetch_optional(&self.pool)
        .await?;

        Ok(call)
    }

    async fn end_call(
        &self,
        call_id: Uuid,
        duration_seconds: i32,
    ) -> Result<Option<CallEntity>, error::SystemError> {
        let call = sqlx::query_as::<_, CallEntity>(
            r#"
            UPDATE calls
            SET
                status = 'ended',
                ended_at = COALESCE(ended_at, NOW()),
                duration_seconds = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(call_id)
        .bind(duration_seconds)
        .fetch_optional(&self.pool)
        .await?;

        Ok(call)
    }

    async fn get_conversation_member_ids(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Uuid>, error::SystemError> {
        let rows = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT user_id
            FROM participants
            WHERE conversation_id = $1
              AND deleted_at IS NULL
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn is_user_in_conversation(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM participants
                WHERE conversation_id = $1
                  AND user_id = $2
                  AND deleted_at IS NULL
            )
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn get_user_calls(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<Vec<CallWithDetails>, error::SystemError> {
        let rows = sqlx::query_as::<_, CallWithDetails>(
            r#"
            SELECT
                c.id,
                c.conversation_id,
                c.initiator_id,
                u.display_name AS initiator_name,
                u.avatar_url AS initiator_avatar,
                c._type AS call_type,
                c.status,
                c.duration_seconds,
                c.started_at,
                c.ended_at,
                c.created_at
            FROM calls c
            JOIN users u ON u.id = c.initiator_id
            WHERE EXISTS (
                SELECT 1
                FROM participants p
                WHERE p.conversation_id = c.conversation_id
                  AND p.user_id = $1
                  AND p.deleted_at IS NULL
            )
              AND ($3::timestamptz IS NULL OR c.created_at < $3)
            ORDER BY c.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(cursor)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn create_call_message(
        &self,
        conversation_id: Uuid,
        sender_id: Uuid,
        message_type: MessageType,
        content: Option<String>,
    ) -> Result<(), error::SystemError> {
        sqlx::query(
            r#"
            INSERT INTO messages (conversation_id, sender_id, type, content)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(conversation_id)
        .bind(sender_id)
        .bind(message_type)
        .bind(content)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct CallParticipantPgRepository {
    pool: sqlx::PgPool,
}

impl CallParticipantPgRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CallParticipantRepository for CallParticipantPgRepository {
    async fn add_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<CallParticipantEntity, error::SystemError> {
        let participant = sqlx::query_as::<_, CallParticipantEntity>(
            r#"
            INSERT INTO call_participants (call_id, user_id, joined_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (call_id, user_id)
            DO UPDATE SET joined_at = COALESCE(call_participants.joined_at, NOW())
            RETURNING *
            "#,
        )
        .bind(call_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(participant)
    }

    async fn mark_left(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), error::SystemError> {
        sqlx::query(
            r#"
            UPDATE call_participants
            SET left_at = COALESCE(left_at, NOW())
            WHERE call_id = $1 AND user_id = $2
            "#,
        )
        .bind(call_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn is_call_participant(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, error::SystemError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM call_participants
                WHERE call_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(call_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
