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

    async fn begin_tx(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, error::SystemError> {
        self.friend_repo
            .get_pool()
            .begin()
            .await
            .map_err(Into::into)
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
        ensure_not_self_friend_request(sender_id, receiver_id)?;

        if self.user_repo.find_by_id(&receiver_id).await?.is_none() {
            return Err(error::SystemError::not_found_key(
                messages::i18n::Key::FriendReceiverNotFound,
            ));
        }

        let (u1, u2) = normalize_friend_pair(sender_id, receiver_id);

        let pool = self.friend_repo.get_pool();

        let (friends, requests): (Option<FriendEntity>, Option<FriendRequestEntity>) = tokio::try_join!(
            self.friend_repo.find_friendship(&u1, &u2, pool),
            self.friend_repo
                .find_friend_request(&sender_id, &receiver_id, pool),
        )?;

        if friends.is_some() {
            return Err(error::SystemError::bad_request_key(
                messages::i18n::Key::AlreadyFriends,
            ));
        }

        if requests.is_some() {
            return Err(error::SystemError::bad_request_key(
                messages::i18n::Key::FriendRequestAlreadyExists,
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
                error::SystemError::not_found_key(messages::i18n::Key::FriendRequestNotFound)
            })?;

        ensure_friend_request_receiver(
            request.to_user_id,
            user_id,
            messages::i18n::Key::ForbiddenAcceptFriendRequest,
        )?;

        let mut tx = self.begin_tx().await?;

        let (u1, u2) = normalize_friend_pair(request.from_user_id, request.to_user_id);

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
            .ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::UserInfoNotFound)
            })?;

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
                error::SystemError::not_found_key(messages::i18n::Key::FriendRequestNotFound)
            })?;

        ensure_friend_request_receiver(
            request.to_user_id,
            user_id,
            messages::i18n::Key::ForbiddenDeclineFriendRequest,
        )?;

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

fn ensure_not_self_friend_request(
    sender_id: Uuid,
    receiver_id: Uuid,
) -> Result<(), error::SystemError> {
    if sender_id != receiver_id {
        return Ok(());
    }

    Err(error::SystemError::bad_request_key(
        messages::i18n::Key::SelfFriendRequestNotAllowed,
    ))
}

fn normalize_friend_pair(user_a: Uuid, user_b: Uuid) -> (Uuid, Uuid) {
    if user_a <= user_b {
        (user_a, user_b)
    } else {
        (user_b, user_a)
    }
}

fn ensure_friend_request_receiver(
    request_receiver_id: Uuid,
    acting_user_id: Uuid,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if request_receiver_id == acting_user_id {
        return Ok(());
    }

    Err(error::SystemError::forbidden_key(error_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_not_self_friend_request_rejects_same_user() {
        let user_id = Uuid::now_v7();
        let result = ensure_not_self_friend_request(user_id, user_id);
        assert!(matches!(
            result,
            Err(error::SystemError::BadRequest(_) | error::SystemError::BadRequestKey(_))
        ));
    }

    #[test]
    fn normalize_friend_pair_returns_sorted_ids() {
        let high = Uuid::from_u128(2);
        let low = Uuid::from_u128(1);

        let (left, right) = normalize_friend_pair(high, low);
        assert_eq!(left, low);
        assert_eq!(right, high);
    }

    #[test]
    fn ensure_friend_request_receiver_rejects_non_target_user() {
        let result = ensure_friend_request_receiver(
            Uuid::now_v7(),
            Uuid::now_v7(),
            messages::i18n::Key::ForbiddenDeclineFriendRequest,
        );

        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }
}
