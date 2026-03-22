use uuid::Uuid;

use crate::api::error;
use crate::modules::friend::model::{FriendRequestResponse, FriendResponse};
use crate::modules::friend::schema::{FriendEntity, FriendRequestEntity};

#[async_trait::async_trait]
pub trait FriendRepository {
    /// Finds friendship relation between 2 users (normalized pair).
    async fn find_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<Option<FriendEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Lists all friends of one user.
    async fn find_friends<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Creates friendship edge for 2 users.
    async fn create_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Removes friendship edge for 2 users.
    async fn delete_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;
}

#[async_trait::async_trait]
pub trait FriendRequestRepository {
    /// Finds friend request by user pair.
    async fn find_friend_request<'e, E>(
        &self,
        sender_id: &Uuid,
        receiver_id: &Uuid,
        tx: E,
    ) -> Result<Option<FriendRequestEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Finds one friend request by request id.
    async fn find_friend_request_by_id<'e, E>(
        &self,
        request_id: &Uuid,
        tx: E,
    ) -> Result<Option<FriendRequestEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Lists outgoing friend requests of a user.
    async fn find_friend_request_from_user<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendRequestResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Lists incoming friend requests of a user.
    async fn find_friend_request_to_user<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendRequestResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Creates a new friend request.
    async fn create_friend_request<'e, E>(
        &self,
        sender_id: &Uuid,
        receiver_id: &Uuid,
        message: &Option<String>,
        tx: E,
    ) -> Result<FriendRequestEntity, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Deletes friend request by id.
    async fn delete_friend_request<'e, E>(
        &self,
        request_id: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;
}

#[async_trait::async_trait]
/// Combined repository contract used by friend service.
pub trait FriendRepo: FriendRepository + FriendRequestRepository + Send + Sync {
    /// Returns Postgres pool for service-level transaction entry.
    fn get_pool(&self) -> &sqlx::PgPool;
}
