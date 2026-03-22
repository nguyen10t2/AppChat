use uuid::Uuid;

use crate::{
    api::error,
    modules::friend::{
        model::{FriendRequestResponse, FriendResponse, FriendUserRow, IdOrInfo},
        repository::{FriendRepo, FriendRepository, FriendRequestRepository},
        schema::{FriendEntity, FriendRequestEntity},
    },
};

const SQL_FIND_FRIENDSHIP: &str = "SELECT * FROM friends WHERE user_a = $1 AND user_b = $2";
const SQL_FIND_FRIENDS: &str = r#"
        SELECT
            u.id,
            u.username,
            u.display_name,
            u.avatar_url,
            u.avatar_id
        FROM friends f
        JOIN users u
            ON u.id = CASE
                WHEN f.user_a = $1 THEN f.user_b
                ELSE f.user_a
            END
        WHERE f.user_a = $1
           OR f.user_b = $1
        "#;
const SQL_CREATE_FRIENDSHIP: &str =
    "INSERT INTO friends (user_a, user_b) VALUES ($1, $2) ON CONFLICT DO NOTHING";
const SQL_DELETE_FRIENDSHIP: &str = "DELETE FROM friends WHERE user_a = $1 AND user_b = $2";
const SQL_FIND_REQUEST_BY_PAIR: &str = r#"
            SELECT *
            FROM friend_requests
            WHERE
                (from_user_id = $1 AND to_user_id = $2)
            OR (from_user_id = $2 AND to_user_id = $1)
            "#;
const SQL_FIND_REQUEST_BY_ID: &str = "SELECT * FROM friend_requests WHERE id = $1";
const SQL_FIND_REQUEST_FROM_USER: &str = r#"
            SELECT
                fr.id AS req_id,
                u.id AS user_id,
                u.username,
                u.display_name,
                u.avatar_url,
                u.avatar_id,
                fr.message,
                fr.created_at
            FROM friend_requests fr
            JOIN users u
                ON fr.to_user_id = u.id
            WHERE fr.from_user_id = $1
            "#;
const SQL_FIND_REQUEST_TO_USER: &str = r#"
            SELECT
                fr.id AS req_id,
                u.id AS user_id,
                u.username,
                u.display_name,
                u.avatar_url,
                u.avatar_id,
                fr.message,
                fr.created_at
            FROM friend_requests fr
            JOIN users u
                ON fr.from_user_id = u.id
            WHERE fr.to_user_id = $1
            "#;
const SQL_CREATE_FRIEND_REQUEST: &str = r#"
            INSERT INTO friend_requests (id, from_user_id, to_user_id, message)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#;
const SQL_DELETE_FRIEND_REQUEST: &str = "DELETE FROM friend_requests WHERE id = $1";
const SQL_FIND_FRIEND_IDS: &str = r#"
            SELECT CASE WHEN f.user_a = $1 THEN f.user_b ELSE f.user_a END
            FROM friends f
            WHERE f.user_a = $1 OR f.user_b = $1
            "#;

fn normalized_friend_pair<'a>(user_id_a: &'a Uuid, user_id_b: &'a Uuid) -> (&'a Uuid, &'a Uuid) {
    if user_id_a <= user_id_b {
        (user_id_a, user_id_b)
    } else {
        (user_id_b, user_id_a)
    }
}

fn to_friend_request_responses(
    rows: Vec<FriendUserRow>,
    user_id: &Uuid,
    is_from_user: bool,
) -> Vec<FriendRequestResponse> {
    rows.into_iter()
        .map(|row| {
            let friend_info = FriendResponse {
                id: row.user_id,
                username: row.username,
                display_name: row.display_name,
                avatar_url: row.avatar_url,
            };

            if is_from_user {
                FriendRequestResponse {
                    id: row.req_id,
                    from: IdOrInfo::Id(*user_id),
                    to: IdOrInfo::Info(friend_info),
                    message: row.message,
                    created_at: row.created_at,
                }
            } else {
                FriendRequestResponse {
                    id: row.req_id,
                    from: IdOrInfo::Info(friend_info),
                    to: IdOrInfo::Id(*user_id),
                    message: row.message,
                    created_at: row.created_at,
                }
            }
        })
        .collect()
}

#[derive(Clone)]
pub struct FriendRepositoryPg {
    pool: sqlx::PgPool,
}

impl FriendRepositoryPg {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FriendRepository for FriendRepositoryPg {
    async fn find_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<Option<FriendEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let (user_a, user_b) = normalized_friend_pair(user_id_a, user_id_b);

        let friendship = sqlx::query_as::<_, FriendEntity>(SQL_FIND_FRIENDSHIP)
            .bind(user_a)
            .bind(user_b)
            .fetch_optional(tx)
            .await?;

        Ok(friendship)
    }

    async fn find_friends<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let friends = sqlx::query_as::<_, FriendResponse>(SQL_FIND_FRIENDS)
            .bind(user_id)
            .fetch_all(tx)
            .await?;

        Ok(friends)
    }

    async fn create_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let (user_a, user_b) = normalized_friend_pair(user_id_a, user_id_b);

        sqlx::query(SQL_CREATE_FRIENDSHIP)
            .bind(user_a)
            .bind(user_b)
            .execute(tx)
            .await?;

        Ok(())
    }

    async fn delete_friendship<'e, E>(
        &self,
        user_id_a: &Uuid,
        user_id_b: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let (user_a, user_b) = normalized_friend_pair(user_id_a, user_id_b);

        sqlx::query(SQL_DELETE_FRIENDSHIP)
            .bind(user_a)
            .bind(user_b)
            .execute(tx)
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl FriendRequestRepository for FriendRepositoryPg {
    async fn find_friend_request<'e, E>(
        &self,
        sender_id: &Uuid,
        receiver_id: &Uuid,
        tx: E,
    ) -> Result<Option<FriendRequestEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let request = sqlx::query_as::<_, FriendRequestEntity>(SQL_FIND_REQUEST_BY_PAIR)
            .bind(sender_id)
            .bind(receiver_id)
            .fetch_optional(tx)
            .await?;

        Ok(request)
    }

    async fn find_friend_request_by_id<'e, E>(
        &self,
        request_id: &Uuid,
        tx: E,
    ) -> Result<Option<FriendRequestEntity>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let request = sqlx::query_as::<_, FriendRequestEntity>(SQL_FIND_REQUEST_BY_ID)
            .bind(request_id)
            .fetch_optional(tx)
            .await?;

        Ok(request)
    }

    async fn find_friend_request_from_user<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendRequestResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let rows = sqlx::query_as::<_, FriendUserRow>(SQL_FIND_REQUEST_FROM_USER)
            .bind(user_id)
            .fetch_all(tx)
            .await?;

        Ok(to_friend_request_responses(rows, user_id, true))
    }

    async fn find_friend_request_to_user<'e, E>(
        &self,
        user_id: &Uuid,
        tx: E,
    ) -> Result<Vec<FriendRequestResponse>, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let rows = sqlx::query_as::<_, FriendUserRow>(SQL_FIND_REQUEST_TO_USER)
            .bind(user_id)
            .fetch_all(tx)
            .await?;

        Ok(to_friend_request_responses(rows, user_id, false))
    }

    async fn create_friend_request<'e, E>(
        &self,
        sender_id: &Uuid,
        receiver_id: &Uuid,
        message: &Option<String>,
        tx: E,
    ) -> Result<FriendRequestEntity, error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id = Uuid::now_v7();
        let request = sqlx::query_as::<_, FriendRequestEntity>(SQL_CREATE_FRIEND_REQUEST)
            .bind(id)
            .bind(sender_id)
            .bind(receiver_id)
            .bind(message)
            .fetch_one(tx)
            .await?;

        Ok(request)
    }

    async fn delete_friend_request<'e, E>(
        &self,
        request_id: &Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query(SQL_DELETE_FRIEND_REQUEST)
            .bind(request_id)
            .execute(tx)
            .await?;

        Ok(())
    }
}

impl FriendRepositoryPg {
    /// Lấy danh sách friend IDs (lightweight, không join users table)
    /// Dùng cho presence notifications - chỉ cần IDs, không cần thông tin chi tiết
    pub async fn find_friend_ids(&self, user_id: &Uuid) -> Result<Vec<Uuid>, error::SystemError> {
        let ids = sqlx::query_scalar::<_, Uuid>(SQL_FIND_FRIEND_IDS)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(ids)
    }
}

#[async_trait::async_trait]
impl FriendRepo for FriendRepositoryPg {
    fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
