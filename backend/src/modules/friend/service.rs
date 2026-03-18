use std::sync::Arc;

use uuid::Uuid;

use crate::{
    api::{error, messages},
    modules::{
        friend::{
            model::{FriendRequestResponse, FriendResponse},
            repository::FriendRepo,
            schema::{FriendEntity, FriendRequestEntity},
        },
        user::repository::UserRepository,
    },
};

/// Dịch vụ xử lý logic liên quan đến bạn bè (Thêm, Xóa, Đồng ý, Từ chối)
#[derive(Clone)]
pub struct FriendService<R, U>
where
    R: FriendRepo + Send + Sync,
    U: UserRepository + Send + Sync,
{
    friend_repo: Arc<R>,
    user_repo: Arc<U>,
}

impl<R, U> FriendService<R, U>
where
    R: FriendRepo + Send + Sync,
    U: UserRepository + Send + Sync,
{
    pub fn with_dependencies(friend_repo: Arc<R>, user_repo: Arc<U>) -> Self {
        FriendService {
            friend_repo,
            user_repo,
        }
    }

    /// Kiểm tra xem 2 user có phải là bạn bè hay không
    pub async fn is_friend(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<bool, error::SystemError> {
        let friendship = self
            .friend_repo
            .find_friendship(&user_id, &friend_id, self.friend_repo.get_pool())
            .await?;
        Ok(friendship.is_some())
    }

    /// Lấy danh sách bạn bè của một user
    pub async fn get_friends(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendResponse>, error::SystemError> {
        let friends = self
            .friend_repo
            .find_friends(&user_id, self.friend_repo.get_pool())
            .await?;
        Ok(friends)
    }

    /// Chấm dứt mối quan hệ bạn bè giữa 2 user
    pub async fn remove_friend(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<(), error::SystemError> {
        self.friend_repo
            .delete_friendship(&user_id, &friend_id, self.friend_repo.get_pool())
            .await
    }

    /// Gửi một lời mời kết bạn mới
    pub async fn send_friend_request(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
        message: Option<String>,
    ) -> Result<FriendRequestEntity, error::SystemError> {
        if receiver_id == sender_id {
            return Err(error::SystemError::bad_request(
                "Không thể tự gửi yêu cầu kết bạn cho chính mình",
            ));
        }

        if self.user_repo.find_by_id(&receiver_id).await?.is_none() {
            return Err(error::SystemError::not_found(
                messages::error::FRIEND_RECEIVER_NOT_FOUND,
            ));
        }

        let (u1, u2) = if sender_id <= receiver_id {
            (sender_id, receiver_id)
        } else {
            (receiver_id, sender_id)
        };

        let pool = self.friend_repo.get_pool();

        let (friends, requests): (Option<FriendEntity>, Option<FriendRequestEntity>) = tokio::try_join!(
            self.friend_repo.find_friendship(&u1, &u2, pool),
            self.friend_repo
                .find_friend_request(&sender_id, &receiver_id, pool),
        )?;

        if friends.is_some() {
            return Err(error::SystemError::bad_request("Hai người đã là bạn bè"));
        }

        if requests.is_some() {
            return Err(error::SystemError::bad_request(
                "Yêu cầu kết bạn đã tồn tại",
            ));
        }

        let friend_request = self
            .friend_repo
            .create_friend_request(&sender_id, &receiver_id, &message, pool)
            .await?;

        Ok(friend_request)
    }

    /// Chấp nhận yêu cầu kết bạn
    pub async fn accept_friend_request(
        &self,
        user_id: Uuid,
        request_id: Uuid,
    ) -> Result<FriendResponse, error::SystemError> {
        let pool = self.friend_repo.get_pool();

        let request = self
            .friend_repo
            .find_friend_request_by_id(&request_id, pool)
            .await?
            .ok_or_else(|| {
                error::SystemError::not_found(messages::error::FRIEND_REQUEST_NOT_FOUND)
            })?;

        if request.to_user_id != user_id {
            return Err(error::SystemError::forbidden(
                messages::error::FORBIDDEN_ACCEPT_FRIEND_REQUEST,
            ));
        }

        let mut tx = pool.begin().await?;

        let (u1, u2) = if request.from_user_id <= request.to_user_id {
            (request.from_user_id, request.to_user_id)
        } else {
            (request.to_user_id, request.from_user_id)
        };

        self.friend_repo
            .create_friendship(&u1, &u2, tx.as_mut())
            .await?;

        self.friend_repo
            .delete_friend_request(&request_id, tx.as_mut())
            .await?;

        tx.commit().await?;

        let from_user = self
            .user_repo
            .find_by_id(&request.from_user_id)
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy thông tin người dùng"))?;

        Ok(FriendResponse::from(from_user))
    }

    /// Từ chối yêu cầu kết bạn
    pub async fn decline_friend_request(
        &self,
        user_id: Uuid,
        request_id: Uuid,
    ) -> Result<(), error::SystemError> {
        let pool = self.friend_repo.get_pool();

        let request = self
            .friend_repo
            .find_friend_request_by_id(&request_id, pool)
            .await?
            .ok_or_else(|| {
                error::SystemError::not_found(messages::error::FRIEND_REQUEST_NOT_FOUND)
            })?;

        if request.to_user_id != user_id {
            return Err(error::SystemError::forbidden(
                messages::error::FORBIDDEN_DECLINE_FRIEND_REQUEST,
            ));
        }

        self.friend_repo
            .delete_friend_request(&request_id, pool)
            .await?;

        Ok(())
    }

    /// Lấy danh sách bạn bè và lời mời kết bạn (đến và đi)
    pub async fn get_friend_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequestResponse>, error::SystemError> {
        let pool = self.friend_repo.get_pool();
        let (requests_to, requests_from) = tokio::try_join!(
            self.friend_repo.find_friend_request_to_user(&user_id, pool),
            self.friend_repo
                .find_friend_request_from_user(&user_id, pool),
        )?;

        let mut all = Vec::with_capacity(requests_to.len() + requests_from.len());
        all.extend(requests_to);
        all.extend(requests_from);
        Ok(all)
    }
}
