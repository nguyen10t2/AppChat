#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use crate::api::error;
    use crate::configs::CacheStore;
    use crate::modules::user::model::{SignInModel, UpdateUser, UpdateUserModel};
    use crate::modules::user::repository::UserRepository;
    use crate::modules::user::schema::{UserEntity, UserRole};
    use crate::modules::user::service::UserService;

    #[derive(Clone, Default)]
    struct InMemoryCache {
        store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    #[async_trait::async_trait]
    impl CacheStore for InMemoryCache {
        async fn get<T>(&self, key: &str) -> Result<Option<T>, error::SystemError>
        where
            T: serde::de::DeserializeOwned + Send,
        {
            let store = self.store.lock().expect("cache mutex poisoned");
            match store.get(key) {
                Some(raw) => Ok(Some(serde_json::from_slice(raw)?)),
                None => Ok(None),
            }
        }

        async fn set<T>(
            &self,
            key: &str,
            value: &T,
            _expiration: usize,
        ) -> Result<(), error::SystemError>
        where
            T: serde::Serialize + Send + Sync,
        {
            let mut store = self.store.lock().expect("cache mutex poisoned");
            store.insert(key.to_string(), serde_json::to_vec(value)?);
            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), error::SystemError> {
            let mut store = self.store.lock().expect("cache mutex poisoned");
            store.remove(key);
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockUserRepo {
        users_by_id: Arc<Mutex<HashMap<Uuid, UserEntity>>>,
        users_by_username: Arc<Mutex<HashMap<String, UserEntity>>>,
        search_result: Arc<Mutex<Vec<UserEntity>>>,
        update_result: Arc<Mutex<Option<UserEntity>>>,
        last_search_limit: Arc<Mutex<Option<i32>>>,
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepo {
        async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserEntity>, error::SystemError> {
            let users = self.users_by_id.lock().expect("repo mutex poisoned");
            Ok(users.get(id).cloned())
        }

        async fn find_by_username(
            &self,
            username: &str,
        ) -> Result<Option<UserEntity>, error::SystemError> {
            let users = self
                .users_by_username
                .lock()
                .expect("repo mutex poisoned");
            Ok(users.get(username).cloned())
        }

        async fn create(&self, _user: &crate::modules::user::model::InsertUser) -> Result<Uuid, error::SystemError> {
            Ok(Uuid::now_v7())
        }

        async fn update(&self, _id: &Uuid, _user: &UpdateUser) -> Result<UserEntity, error::SystemError> {
            let updated = self.update_result.lock().expect("repo mutex poisoned").clone();
            updated.ok_or_else(|| error::SystemError::not_found("Không tìm thấy người dùng"))
        }

        async fn delete(&self, _id: &Uuid) -> Result<bool, error::SystemError> {
            Ok(true)
        }

        async fn search_users(
            &self,
            _query: &str,
            limit: i32,
        ) -> Result<Vec<UserEntity>, error::SystemError> {
            let mut last_limit = self
                .last_search_limit
                .lock()
                .expect("repo mutex poisoned");
            *last_limit = Some(limit);

            Ok(self.search_result.lock().expect("repo mutex poisoned").clone())
        }
    }

    fn build_user(id: Uuid, username: &str, hash_password: &str) -> UserEntity {
        UserEntity {
            id,
            username: username.to_string(),
            email: format!("{username}@appchat.local"),
            hash_password: hash_password.to_string(),
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

    async fn build_service(repo: MockUserRepo, cache: InMemoryCache) -> UserService<MockUserRepo, InMemoryCache> {
        UserService::with_dependencies(Arc::new(repo), Arc::new(cache))
    }

    #[tokio::test]
    async fn test_sign_in_returns_unauthorized_when_user_missing() {
        let service = build_service(MockUserRepo::default(), InMemoryCache::default()).await;

        let result = service
            .sign_in(SignInModel {
                username: "missing_user".to_string(),
                password: "secret123".to_string(),
            })
            .await;

        assert!(matches!(result, Err(error::SystemError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_sign_in_returns_unauthorized_when_password_invalid() {
        let user_id = Uuid::now_v7();
        let valid_hash = crate::utils::hash_password("correct_password".to_string())
            .await
            .expect("must hash password for test");

        let user = build_user(user_id, "alice", &valid_hash);
        let mut by_username = HashMap::new();
        by_username.insert("alice".to_string(), user.clone());

        let repo = MockUserRepo {
            users_by_id: Arc::new(Mutex::new(HashMap::from([(user_id, user)]))),
            users_by_username: Arc::new(Mutex::new(by_username)),
            ..Default::default()
        };

        let service = build_service(repo, InMemoryCache::default()).await;

        let result = service
            .sign_in(SignInModel {
                username: "alice".to_string(),
                password: "wrong_password".to_string(),
            })
            .await;

        assert!(matches!(result, Err(error::SystemError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_sign_in_and_refresh_rotate_refresh_token() {
        let user_id = Uuid::now_v7();
        let valid_hash = crate::utils::hash_password("correct_password".to_string())
            .await
            .expect("must hash password for test");

        let user = build_user(user_id, "bob", &valid_hash);
        let mut by_username = HashMap::new();
        by_username.insert("bob".to_string(), user.clone());

        let repo = MockUserRepo {
            users_by_id: Arc::new(Mutex::new(HashMap::from([(user_id, user)]))),
            users_by_username: Arc::new(Mutex::new(by_username)),
            ..Default::default()
        };

        let cache = InMemoryCache::default();
        let cache_ref = cache.clone();
        let service = build_service(repo, cache).await;

        let (_access_token, old_refresh_token) = service
            .sign_in(SignInModel {
                username: "bob".to_string(),
                password: "correct_password".to_string(),
            })
            .await
            .expect("sign in should succeed");

        let keys_after_sign_in: Vec<String> = {
            let store = cache_ref.store.lock().expect("cache mutex poisoned");
            store.keys().cloned().collect()
        };
        assert_eq!(keys_after_sign_in.len(), 1);
        let old_key = keys_after_sign_in[0].clone();

        let (_new_access, _new_refresh) = service
            .refresh(Some(old_refresh_token))
            .await
            .expect("refresh should succeed");

        let keys_after_refresh: Vec<String> = {
            let store = cache_ref.store.lock().expect("cache mutex poisoned");
            store.keys().cloned().collect()
        };

        assert_eq!(keys_after_refresh.len(), 1);
        assert_ne!(keys_after_refresh[0], old_key);
    }

    #[tokio::test]
    async fn test_search_users_validates_query_and_clamps_limit() {
        let user_id = Uuid::now_v7();
        let user = build_user(user_id, "carol", "hash");

        let repo = MockUserRepo {
            search_result: Arc::new(Mutex::new(vec![user])),
            ..Default::default()
        };

        let repo_ref = repo.clone();
        let service = build_service(repo, InMemoryCache::default()).await;

        let empty_query = service.search_users("   ", 10).await;
        assert!(matches!(empty_query, Err(error::SystemError::BadRequest(_))));

        let short_query = service.search_users("a", 10).await;
        assert!(matches!(short_query, Err(error::SystemError::BadRequest(_))));

        let users = service
            .search_users("car", 999)
            .await
            .expect("search should succeed");

        assert_eq!(users.len(), 1);

        let last_limit = repo_ref
            .last_search_limit
            .lock()
            .expect("repo mutex poisoned")
            .to_owned();
        assert_eq!(last_limit, Some(50));
    }

    #[tokio::test]
    async fn test_update_rejects_empty_payload() {
        let service = build_service(MockUserRepo::default(), InMemoryCache::default()).await;
        let user_id = Uuid::now_v7();

        let result = service
            .update(
                user_id,
                UpdateUserModel {
                    username: None,
                    email: None,
                    display_name: None,
                    avatar_url: None,
                    bio: None,
                    phone: None,
                },
            )
            .await;

        assert!(matches!(result, Err(error::SystemError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_update_success_sets_user_cache() {
        let user_id = Uuid::now_v7();
        let updated_user = build_user(user_id, "david", "hash");

        let repo = MockUserRepo {
            update_result: Arc::new(Mutex::new(Some(updated_user))),
            ..Default::default()
        };

        let cache = InMemoryCache::default();
        let cache_ref = cache.clone();
        let service = build_service(repo, cache).await;

        let response = service
            .update(
                user_id,
                UpdateUserModel {
                    username: None,
                    email: None,
                    display_name: Some("David".to_string()),
                    avatar_url: Some(Some("https://cdn/appchat/avatar.png".to_string())),
                    bio: Some(Some("hello".to_string())),
                    phone: Some(Some("0123456789".to_string())),
                },
            )
            .await
            .expect("update should succeed");

        assert_eq!(response.id, user_id);

        let cache_key = format!("user:{user_id}");
        let cached_user = cache_ref
            .get::<crate::modules::user::model::UserResponse>(&cache_key)
            .await
            .expect("cache read should succeed");

        assert!(cached_user.is_some());
    }
}
