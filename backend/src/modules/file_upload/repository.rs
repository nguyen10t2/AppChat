use uuid::Uuid;

use crate::{
    api::error,
    modules::file_upload::{model::NewFile, schema::FileEntity},
};

#[async_trait::async_trait]
pub trait FileRepository {
    /// Returns Postgres pool for service-level transaction entry.
    fn get_pool(&self) -> &sqlx::Pool<sqlx::Postgres>;

    /// Creates a file metadata record.
    async fn create<'e, E>(&self, file: &NewFile, tx: E) -> Result<FileEntity, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    /// Finds file metadata by id.
    async fn find_by_id(&self, file_id: &Uuid) -> Result<Option<FileEntity>, error::SystemError>;

    /// Deletes file metadata by id.
    async fn delete<'e, E>(&self, file_id: &Uuid, tx: E) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;
}
