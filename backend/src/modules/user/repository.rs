use uuid::Uuid;

use crate::{
    api::error, modules::user::model::InsertUser, modules::user::model::UpdateUser,
    modules::user::schema::UserEntity,
};

#[async_trait::async_trait]
pub trait UserRepository {
    /// Finds one active user by id.
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserEntity>, error::SystemError>;

    /// Finds one active user by username (case-insensitive).
    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, error::SystemError>;

    /// Creates a new user and returns generated id.
    async fn create(&self, user: &InsertUser) -> Result<Uuid, error::SystemError>;

    /// Updates mutable fields of a user.
    #[allow(unused)]
    async fn update(&self, id: &Uuid, user: &UpdateUser) -> Result<UserEntity, error::SystemError>;

    /// Soft-deletes user by id.
    async fn delete(&self, id: &Uuid) -> Result<bool, error::SystemError>;

    /// Search users by username or display name (case-insensitive, partial match)
    async fn search_users(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<Vec<UserEntity>, error::SystemError>;
}
