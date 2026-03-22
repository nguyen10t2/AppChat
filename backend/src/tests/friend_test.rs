#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use crate::api::error;
    use crate::modules::friend::repository::{FriendRepo, FriendRepository, FriendRequestRepository};
    use crate::modules::friend::schema::{FriendEntity, FriendRequestEntity};
    use crate::modules::friend::service::FriendService;
    use crate::modules::user::model::{InsertUser, UpdateUser};
    use crate::modules::user::repository::UserRepository;
    use crate::modules::user::schema::{UserEntity, UserRole};
    use crate::tests::mock::database::MockDatabase;

    fn build_user(id: Uuid, username: &str) -> UserEntity {
        UserEntity {
            id,
            username: username.to_string(),
            email: format!("{username}@appchat.local"),
            hash_password: "hash".to_string(),
            role: UserRole::User,
            display_name: username.to_string(),
            avatar_url: None,
            bio: None,
            phone: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Clone, Default)]
    struct MockUserRepo {
        users: Arc<Mutex<HashMap<Uuid, UserEntity>>>,
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepo {
        async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserEntity>, error::SystemError> {
            let users = self.users.lock().expect("user repo mutex poisoned");
            Ok(users.get(id).cloned())
        }

        async fn find_by_username(
            &self,
            _username: &str,
        ) -> Result<Option<UserEntity>, error::SystemError> {
            Ok(None)
        }

        async fn create(&self, _user: &InsertUser) -> Result<Uuid, error::SystemError> {
            Ok(Uuid::now_v7())
        }

        async fn update(&self, _id: &Uuid, _user: &UpdateUser) -> Result<UserEntity, error::SystemError> {
            Err(error::SystemError::internal_error("not used"))
        }

        async fn delete(&self, _id: &Uuid) -> Result<bool, error::SystemError> {
            Ok(true)
        }

        async fn search_users(
            &self,
            _query: &str,
            _limit: i32,
        ) -> Result<Vec<UserEntity>, error::SystemError> {
            Ok(vec![])
        }
    }

    #[derive(Clone)]
    struct MockFriendRepo {
        pool: sqlx::PgPool,
        friendship: Arc<Mutex<Option<FriendEntity>>>,
        pending_request: Arc<Mutex<Option<FriendRequestEntity>>>,
        request_by_id: Arc<Mutex<Option<FriendRequestEntity>>>,
        create_request_calls: Arc<Mutex<u32>>,
        delete_request_calls: Arc<Mutex<u32>>,
        delete_friendship_calls: Arc<Mutex<u32>>,
    }

    impl Default for MockFriendRepo {
        fn default() -> Self {
            Self {
                pool: MockDatabase::new().pool(),
                friendship: Arc::new(Mutex::new(None)),
                pending_request: Arc::new(Mutex::new(None)),
                request_by_id: Arc::new(Mutex::new(None)),
                create_request_calls: Arc::new(Mutex::new(0)),
                delete_request_calls: Arc::new(Mutex::new(0)),
                delete_friendship_calls: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl FriendRepository for MockFriendRepo {
        async fn find_friendship<'e, E>(
            &self,
            _user_id_a: &Uuid,
            _user_id_b: &Uuid,
            _tx: E,
        ) -> Result<Option<FriendEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(self
                .friendship
                .lock()
                .expect("friend repo mutex poisoned")
                .clone())
        }

        async fn find_friends<'e, E>(
            &self,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<Vec<crate::modules::friend::model::FriendResponse>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(vec![])
        }

        async fn create_friendship<'e, E>(
            &self,
            _user_id_a: &Uuid,
            _user_id_b: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(())
        }

        async fn delete_friendship<'e, E>(
            &self,
            _user_id_a: &Uuid,
            _user_id_b: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            let mut calls = self
                .delete_friendship_calls
                .lock()
                .expect("friend repo mutex poisoned");
            *calls += 1;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl FriendRequestRepository for MockFriendRepo {
        async fn find_friend_request<'e, E>(
            &self,
            _sender_id: &Uuid,
            _receiver_id: &Uuid,
            _tx: E,
        ) -> Result<Option<FriendRequestEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(self
                .pending_request
                .lock()
                .expect("friend repo mutex poisoned")
                .clone())
        }

        async fn find_friend_request_by_id<'e, E>(
            &self,
            _request_id: &Uuid,
            _tx: E,
        ) -> Result<Option<FriendRequestEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(self
                .request_by_id
                .lock()
                .expect("friend repo mutex poisoned")
                .clone())
        }

        async fn find_friend_request_from_user<'e, E>(
            &self,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<Vec<crate::modules::friend::model::FriendRequestResponse>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(vec![])
        }

        async fn find_friend_request_to_user<'e, E>(
            &self,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<Vec<crate::modules::friend::model::FriendRequestResponse>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(vec![])
        }

        async fn create_friend_request<'e, E>(
            &self,
            sender_id: &Uuid,
            receiver_id: &Uuid,
            message: &Option<String>,
            _tx: E,
        ) -> Result<FriendRequestEntity, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            let mut calls = self
                .create_request_calls
                .lock()
                .expect("friend repo mutex poisoned");
            *calls += 1;

            Ok(FriendRequestEntity {
                id: Uuid::now_v7(),
                from_user_id: *sender_id,
                to_user_id: *receiver_id,
                message: message.clone(),
                created_at: Utc::now(),
            })
        }

        async fn delete_friend_request<'e, E>(
            &self,
            _request_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            let mut calls = self
                .delete_request_calls
                .lock()
                .expect("friend repo mutex poisoned");
            *calls += 1;
            Ok(())
        }
    }

    impl FriendRepo for MockFriendRepo {
        fn get_pool(&self) -> &sqlx::PgPool {
            &self.pool
        }
    }

    fn build_service(friend_repo: MockFriendRepo, user_repo: MockUserRepo) -> FriendService<MockFriendRepo, MockUserRepo> {
        FriendService::with_dependencies(Arc::new(friend_repo), Arc::new(user_repo))
    }

    #[tokio::test]
    async fn test_send_friend_request_rejects_self_request() {
        let service = build_service(MockFriendRepo::default(), MockUserRepo::default());
        let user_id = Uuid::now_v7();

        let result = service.send_friend_request(user_id, user_id, None).await;

        assert!(matches!(result, Err(error::SystemError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_send_friend_request_rejects_missing_receiver() {
        let service = build_service(MockFriendRepo::default(), MockUserRepo::default());

        let result = service
            .send_friend_request(Uuid::now_v7(), Uuid::now_v7(), Some("hi".to_string()))
            .await;

        assert!(matches!(result, Err(error::SystemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_send_friend_request_rejects_existing_friendship() {
        let sender_id = Uuid::now_v7();
        let receiver_id = Uuid::now_v7();

        let mut users = HashMap::new();
        users.insert(receiver_id, build_user(receiver_id, "receiver"));

        let friend_repo = MockFriendRepo {
            friendship: Arc::new(Mutex::new(Some(FriendEntity {
                user_a: sender_id.min(receiver_id),
                user_b: sender_id.max(receiver_id),
                deleted_at: None,
                created_at: Utc::now(),
            }))),
            ..Default::default()
        };

        let user_repo = MockUserRepo {
            users: Arc::new(Mutex::new(users)),
        };

        let service = build_service(friend_repo, user_repo);

        let result = service
            .send_friend_request(sender_id, receiver_id, Some("hello".to_string()))
            .await;

        assert!(matches!(result, Err(error::SystemError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_send_friend_request_success_creates_request() {
        let sender_id = Uuid::now_v7();
        let receiver_id = Uuid::now_v7();

        let mut users = HashMap::new();
        users.insert(receiver_id, build_user(receiver_id, "receiver"));

        let friend_repo = MockFriendRepo::default();
        let friend_repo_ref = friend_repo.clone();
        let user_repo = MockUserRepo {
            users: Arc::new(Mutex::new(users)),
        };
        let service = build_service(friend_repo, user_repo);

        let result = service
            .send_friend_request(sender_id, receiver_id, Some("hello".to_string()))
            .await
            .expect("send friend request should succeed");

        assert_eq!(result.from_user_id, sender_id);
        assert_eq!(result.to_user_id, receiver_id);

        let create_calls = friend_repo_ref
            .create_request_calls
            .lock()
            .expect("friend repo mutex poisoned")
            .to_owned();
        assert_eq!(create_calls, 1);
    }

    #[tokio::test]
    async fn test_accept_friend_request_rejects_not_found() {
        let service = build_service(MockFriendRepo::default(), MockUserRepo::default());

        let result = service
            .accept_friend_request(Uuid::now_v7(), Uuid::now_v7())
            .await;

        assert!(matches!(result, Err(error::SystemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_accept_friend_request_rejects_forbidden_user() {
        let owner_id = Uuid::now_v7();
        let another_user_id = Uuid::now_v7();
        let request_id = Uuid::now_v7();

        let friend_repo = MockFriendRepo {
            request_by_id: Arc::new(Mutex::new(Some(FriendRequestEntity {
                id: request_id,
                from_user_id: Uuid::now_v7(),
                to_user_id: owner_id,
                message: None,
                created_at: Utc::now(),
            }))),
            ..Default::default()
        };

        let service = build_service(friend_repo, MockUserRepo::default());

        let result = service.accept_friend_request(another_user_id, request_id).await;

        assert!(matches!(result, Err(error::SystemError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_decline_friend_request_rejects_forbidden_user() {
        let owner_id = Uuid::now_v7();
        let another_user_id = Uuid::now_v7();
        let request_id = Uuid::now_v7();

        let friend_repo = MockFriendRepo {
            request_by_id: Arc::new(Mutex::new(Some(FriendRequestEntity {
                id: request_id,
                from_user_id: Uuid::now_v7(),
                to_user_id: owner_id,
                message: None,
                created_at: Utc::now(),
            }))),
            ..Default::default()
        };
        let friend_repo_ref = friend_repo.clone();
        let service = build_service(friend_repo, MockUserRepo::default());

        let result = service.decline_friend_request(another_user_id, request_id).await;

        assert!(matches!(result, Err(error::SystemError::Forbidden(_))));

        let delete_calls = friend_repo_ref
            .delete_request_calls
            .lock()
            .expect("friend repo mutex poisoned")
            .to_owned();
        assert_eq!(delete_calls, 0);
    }

    #[tokio::test]
    async fn test_decline_friend_request_success_deletes_request() {
        let owner_id = Uuid::now_v7();
        let request_id = Uuid::now_v7();

        let friend_repo = MockFriendRepo {
            request_by_id: Arc::new(Mutex::new(Some(FriendRequestEntity {
                id: request_id,
                from_user_id: Uuid::now_v7(),
                to_user_id: owner_id,
                message: Some("yo".to_string()),
                created_at: Utc::now(),
            }))),
            ..Default::default()
        };
        let friend_repo_ref = friend_repo.clone();
        let service = build_service(friend_repo, MockUserRepo::default());

        let result = service.decline_friend_request(owner_id, request_id).await;

        assert!(result.is_ok());

        let delete_calls = friend_repo_ref
            .delete_request_calls
            .lock()
            .expect("friend repo mutex poisoned")
            .to_owned();
        assert_eq!(delete_calls, 1);
    }

    #[tokio::test]
    async fn test_remove_friend_calls_repository_once() {
        let friend_repo = MockFriendRepo::default();
        let friend_repo_ref = friend_repo.clone();
        let service = build_service(friend_repo, MockUserRepo::default());

        let result = service.remove_friend(Uuid::now_v7(), Uuid::now_v7()).await;

        assert!(result.is_ok());

        let delete_calls = friend_repo_ref
            .delete_friendship_calls
            .lock()
            .expect("friend repo mutex poisoned")
            .to_owned();
        assert_eq!(delete_calls, 1);
    }
}
