use std::sync::Arc;
use uuid::Uuid;

use crate::ENV;
use crate::api::error;
use crate::api::messages;
use crate::configs::{CacheStore, RedisCache};
use crate::modules::CACHE_TTL;
use crate::modules::user::model::{
    SignInModel, SignUpModel, UpdateUser, UpdateUserModel, UserResponse,
};
use crate::modules::user::{model::InsertUser, repository::UserRepository};
use crate::utils::{Claims, TypeClaims, hash_password, verify_password};

/// Dịch vụ quản lý người dùng (Đăng ký, Đăng nhập, Tìm kiếm, Cập nhật thông tin)
#[derive(Clone)]
pub struct UserService<U, C = RedisCache>
where
    U: UserRepository + Send + Sync,
    C: CacheStore + Send + Sync,
{
    repo: Arc<U>,
    cache: Arc<C>,
}

impl<U, C> UserService<U, C>
where
    U: UserRepository + Send + Sync,
    C: CacheStore + Send + Sync,
{
    pub fn with_dependencies(repo: Arc<U>, cache: Arc<C>) -> Self {
        UserService { repo, cache }
    }

    /// Lấy thông tin người dùng theo ID (có cache qua Redis)
    pub async fn get_by_id(&self, id: Uuid) -> Result<UserResponse, error::SystemError> {
        let key = format!("user:{}", id);
        if let Some(cached_user) = self.cache.get::<UserResponse>(&key).await? {
            return Ok(cached_user);
        }
        let user_entity = self.repo.find_by_id(&id).await?;
        if let Some(entity) = user_entity {
            self.cache
                .set(&key, &UserResponse::from(entity.clone()), CACHE_TTL)
                .await?;
            Ok(UserResponse::from(entity))
        } else {
            Err(error::SystemError::not_found(messages::error::USER_NOT_FOUND))
        }
    }

    /// Cập nhật thông tin người dùng (tên hiển thị, avatar, bio, v.v)
    pub async fn update(
        &self,
        id: Uuid,
        user: UpdateUserModel,
    ) -> Result<UserResponse, error::SystemError> {
        if user.is_empty() {
            return Err(error::SystemError::bad_request(
                messages::error::UPDATE_EMPTY_PAYLOAD,
            ));
        }

        let update_user = UpdateUser {
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            phone: user.phone,
        };

        let updated_user = self.repo.update(&id, &update_user).await?;

        let key = format!("user:{}", id);
        let response = UserResponse::from(updated_user);
        self.cache.set(&key, &response, CACHE_TTL).await?;

        Ok(response)
    }

    /// Xóa tài khoản người dùng
    pub async fn delete(&self, id: Uuid) -> Result<(), error::SystemError> {
        let deleted = self.repo.delete(&id).await?;
        if !deleted {
            return Err(error::SystemError::not_found(messages::error::USER_NOT_FOUND));
        }
        Ok(())
    }

    /// Đăng ký tài khoản mới (Sign up)
    pub async fn sign_up(&self, user: SignUpModel) -> Result<uuid::Uuid, error::SystemError> {
        let hash_password = hash_password(user.password).await?;

        let new_user = InsertUser {
            username: user.username,
            email: user.email,
            hash_password,
            display_name: user.display_name,
        };

        let user_id = self.repo.create(&new_user).await?;
        Ok(user_id)
    }

    /// Đăng nhập (Sign in) và trả về Access Token + Refresh Token
    pub async fn sign_in(&self, user: SignInModel) -> Result<(String, String), error::SystemError> {
        let user_entity = self
            .repo
            .find_by_username(&user.username)
            .await?
            .ok_or_else(|| {
                error::SystemError::unauthorized("Tài khoản hoặc mật khẩu không chính xác")
            })?;

        let valid = verify_password(user_entity.hash_password.clone(), user.password).await?;
        if !valid {
            return Err(error::SystemError::unauthorized(
                "Tài khoản hoặc mật khẩu không chính xác",
            ));
        }

        let access_token = Claims::new(
            &user_entity.id,
            &user_entity.role,
            ENV.access_token_expiration,
        )
        .with_type(TypeClaims::AccessToken)
        .encode(ENV.jwt_secret.as_ref())?;

        let jti = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));

        let refresh_token = Claims::new(
            &user_entity.id,
            &user_entity.role,
            ENV.refresh_token_expiration,
        )
        .with_jti(jti)
        .with_type(TypeClaims::RefreshToken)
        .encode(ENV.jwt_secret.as_ref())?;

        let refresh_key = format!("refresh_token:{jti}");
        self.cache
            .set(
                &refresh_key,
                &user_entity.id,
                ENV.refresh_token_expiration as usize,
            )
            .await?;

        Ok((access_token, refresh_token))
    }

    /// Đăng xuất tài khoản (Xóa Refresh Token)
    pub async fn sign_out(&self, refresh_token: Option<String>) -> Result<(), error::SystemError> {
        let Some(token) = refresh_token else {
            return Ok(());
        };

        let payload = Claims::decode(&token, ENV.jwt_secret.as_ref())?;

        let Some(TypeClaims::RefreshToken) = payload._type else {
            return Ok(());
        };

        let Some(jti) = payload.jti else {
            return Ok(());
        };

        let refresh_key = format!("refresh_token:{jti}");
        self.cache.delete(&refresh_key).await?;

        Ok(())
    }

    /// Cấp lại Access Token mới dựa vào Refresh Token
    pub async fn refresh(
        &self,
        old_refresh_token: Option<String>,
    ) -> Result<(String, String), error::SystemError> {
        let invalid = || error::SystemError::unauthorized(messages::error::INVALID_TOKEN);

        let Some(old_refresh_token) = old_refresh_token else {
            return Err(invalid());
        };

        let payload = Claims::decode(&old_refresh_token, ENV.jwt_secret.as_ref())?;

        let Some(TypeClaims::RefreshToken) = payload._type else {
            return Err(invalid());
        };

        let Some(jti) = payload.jti else {
            return Err(invalid());
        };

        let old_key = format!("refresh_token:{jti}");

        if self.cache.get::<String>(&old_key).await?.is_none() {
            return Err(invalid());
        }

        self.cache.delete(&old_key).await?;

        let new_jti = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let new_key = format!("refresh_token:{new_jti}");

        let new_access_token =
            Claims::new(&payload.sub, &payload.role, ENV.access_token_expiration)
                .with_type(TypeClaims::AccessToken)
                .encode(ENV.jwt_secret.as_ref())?;

        let new_refresh_token =
            Claims::new(&payload.sub, &payload.role, ENV.refresh_token_expiration)
                .with_jti(new_jti)
                .with_type(TypeClaims::RefreshToken)
                .encode(ENV.jwt_secret.as_ref())?;

        self.cache
            .set(
                &new_key,
                &payload.sub,
                ENV.refresh_token_expiration as usize,
            )
            .await?;

        Ok((new_access_token, new_refresh_token))
    }

    /// Tìm kiếm user theo username hoặc tên hiển thị
    pub async fn search_users(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<Vec<UserResponse>, error::SystemError> {
        // Validate query length
        if query.trim().is_empty() {
            return Err(error::SystemError::bad_request(
                "Từ khóa tìm kiếm không được để trống",
            ));
        }

        if query.len() < 2 {
            return Err(error::SystemError::bad_request(
                "Từ khóa tìm kiếm phải có ít nhất 2 ký tự",
            ));
        }

        // Validate limit
        let limit = limit.clamp(1, 50); // Limit between 1 and 50

        let users = self.repo.search_users(query, limit).await?;

        let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

        Ok(responses)
    }
}
