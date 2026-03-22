/// Conversation Service
///
/// Service layer xử lý business logic cho conversations.
/// Bao gồm tạo conversation, lấy danh sách, mark as seen, và WebSocket notifications.
use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{
    api::{error, messages},
    modules::{
        conversation::{
            model::{ConversationDetail, ParticipantDetailWithConversation, ParticipantRow},
            repository::{ConversationRepository, ParticipantRepository},
            schema::{ConversationEntity, ConversationType},
        },
        message::{model::MessageQuery, repository::MessageRepository, schema::MessageEntity},
        websocket::{
            message::{LastMessageInfo, SenderInfo, ServerMessage},
            server::WebSocketServer,
        },
    },
    observability::AppMetrics,
};

/// ConversationService với generic repositories để dễ testing và decoupling
#[derive(Clone)]
pub struct ConversationService<R, P, L>
where
    R: ConversationRepository + Send + Sync,
    P: ParticipantRepository + Send + Sync,
    L: MessageRepository + Send + Sync,
{
    conversation_repo: Arc<R>,
    participant_repo: Arc<P>,
    message_repo: Arc<L>,
    ws_server: Arc<WebSocketServer>,
    metrics: Arc<AppMetrics>,
}

impl<R, P, L> ConversationService<R, P, L>
where
    R: ConversationRepository + Send + Sync,
    P: ParticipantRepository + Send + Sync,
    L: MessageRepository + Send + Sync,
{
    /// Tạo ConversationService với tất cả dependencies
    pub fn with_dependencies(
        conversation_repo: Arc<R>,
        participant_repo: Arc<P>,
        message_repo: Arc<L>,
        ws_server: Arc<WebSocketServer>,
    ) -> Self {
        Self::with_dependencies_and_metrics(
            conversation_repo,
            participant_repo,
            message_repo,
            ws_server,
            Arc::new(AppMetrics::default()),
        )
    }

    pub fn with_dependencies_and_metrics(
        conversation_repo: Arc<R>,
        participant_repo: Arc<P>,
        message_repo: Arc<L>,
        ws_server: Arc<WebSocketServer>,
        metrics: Arc<AppMetrics>,
    ) -> Self {
        ConversationService {
            conversation_repo,
            participant_repo,
            message_repo,
            ws_server,
            metrics,
        }
    }

    async fn begin_tx(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, error::SystemError> {
        self.conversation_repo
            .get_pool()
            .begin()
            .await
            .map_err(Into::into)
    }

    /// Lấy conversation theo ID
    pub async fn get_by_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<ConversationEntity, error::SystemError> {
        let conversation = self
            .conversation_repo
            .find_by_id(&conversation_id, self.conversation_repo.get_pool())
            .await?
            .ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::ConversationNotFound)
            })?;

        Ok(conversation)
    }

    /// Tạo conversation mới (direct hoặc group)
    ///
    /// Với direct: tạo hoặc trả về conversation hiện có giữa 2 users
    /// Với group: tạo group mới và notify tất cả members
    pub async fn create_conversation(
        &self,
        _type: ConversationType,
        name: String,
        member_ids: Vec<Uuid>,
        user_id: Uuid,
    ) -> Result<Option<ConversationDetail>, error::SystemError> {
        let mut tx = self.begin_tx().await?;
        let mut did_create_conversation = false;

        let participant = member_ids.first().ok_or_else(|| {
            error::SystemError::bad_request_key(messages::i18n::Key::ConversationMemberRequired)
        })?;

        let conversation = match _type {
            ConversationType::Direct => {
                if let Some(conv) = self
                    .conversation_repo
                    .find_direct_between_users(&user_id, participant, tx.as_mut())
                    .await?
                {
                    conv
                } else {
                    did_create_conversation = true;
                    self.conversation_repo
                        .create_direct_conversation(&user_id, participant, &mut tx)
                        .await?
                }
            }

            ConversationType::Group => {
                let mut all_members = member_ids.clone();
                if !all_members.contains(&user_id) {
                    all_members.push(user_id);
                }
                did_create_conversation = true;
                self.conversation_repo
                    .create_group_conversation(&name, &all_members, &user_id, &mut tx)
                    .await?
            }
        };

        tx.commit().await?;

        let conversation_detail = self
            .conversation_repo
            .find_one_conversation_detail(&conversation.id)
            .await?;

        // Serialize conversation for WebSocket broadcast
        let conversation_json = serde_json::to_value(&conversation_detail).map_err(|e| {
            error::SystemError::internal_error(format!(
                "Lỗi khi xử lý dữ liệu cuộc trò chuyện: {}",
                e
            ))
        })?;

        // Broadcast dựa trên type
        match _type {
            ConversationType::Group => {
                // Gửi new-group event tới tất cả members (trừ creator)
                // Format tương thích với Socket.IO client
                self.ws_server.send_to_users(
                    &member_ids,
                    &ServerMessage::NewGroup {
                        conversation: conversation_json,
                    },
                );
            }
            ConversationType::Direct => {
                // Direct message không cần broadcast khi tạo mới
                // Sẽ broadcast khi có message đầu tiên
            }
        }

        if did_create_conversation {
            self.metrics.inc_conversation_create();
        }

        Ok(conversation_detail)
    }

    /// Lấy tất cả conversations của user
    pub async fn get_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ConversationDetail>, error::SystemError> {
        let pool = self.conversation_repo.get_pool();
        let conversations = self
            .conversation_repo
            .find_all_conversation_with_details_by_user(&user_id, pool)
            .await?;

        let conversation_ids: Vec<Uuid> = conversations
            .iter()
            .map(|conv_row| conv_row.conversation_id)
            .collect();

        let participants = self
            .participant_repo
            .find_participants_by_conversation_id(&conversation_ids, pool)
            .await?;

        let participant_map = participants.into_iter().fold(
            HashMap::<Uuid, Vec<ParticipantDetailWithConversation>>::new(),
            |mut acc, participant| {
                acc.entry(participant.conversation_id)
                    .or_insert_with(Vec::new)
                    .push(participant);
                acc
            },
        );

        let res = conversations.into_iter().map(|conv| {
            let participants: Vec<ParticipantRow> = participant_map
                .get(&conv.conversation_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|p| ParticipantRow {
                    user_id: p.user_id,
                    display_name: p.display_name,
                    avatar_url: p.avatar_url,
                    unread_count: p.unread_count,
                    joined_at: p.joined_at,
                })
                .collect();

            ConversationDetail {
                conversation_id: conv.conversation_id,
                _type: conv._type,
                group_info: conv.group_info,
                last_message: conv.last_message,
                participants,
                created_at: conv.created_at,
                updated_at: conv.updated_at,
            }
        });

        Ok(res.collect())
    }

    /// Lấy messages của conversation với cursor-based pagination
    pub async fn get_message(
        &self,
        conversation_id: Uuid,
        limit: i32,
        cursor: Option<String>,
    ) -> Result<(Vec<MessageEntity>, Option<String>), error::SystemError> {
        let created_at = match cursor {
            Some(c) => Some(
                chrono::DateTime::parse_from_rfc3339(&c)
                    .map_err(|_| {
                        error::SystemError::bad_request_key(
                            messages::i18n::Key::InvalidPaginationCursor,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
            ),
            None => None,
        };

        let mut messages = self
            .message_repo
            .find_by_query(
                &MessageQuery {
                    conversation_id,
                    created_at,
                },
                limit,
                self.message_repo.get_pool(),
            )
            .await?;

        let next_cursor = if messages.len() > limit as usize {
            messages.pop().map(|m| m.created_at)
        } else {
            None
        };

        messages.reverse();
        Ok((messages, next_cursor.map(|c| c.to_rfc3339())))
    }

    /// Lấy participants của conversation
    pub async fn get_participants_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<ParticipantDetailWithConversation>, error::SystemError> {
        let participants = self
            .participant_repo
            .find_participants_by_conversation_id(
                &[conversation_id],
                self.conversation_repo.get_pool(),
            )
            .await?;

        Ok(participants)
    }

    /// Kiểm tra user có phải member của conversation không
    pub async fn get_conversation_and_check_membership(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(Option<ConversationEntity>, bool), error::SystemError> {
        self.conversation_repo
            .get_conversation_and_check_membership(
                &conversation_id,
                &user_id,
                self.conversation_repo.get_pool(),
            )
            .await
    }

    /// Mark messages as seen
    ///
    /// Cập nhật last_seen_message_id và reset unread count
    /// Broadcast read-message event tới conversation room
    pub async fn mark_as_seen(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), error::SystemError> {
        let mut tx = self.begin_tx().await?;

        // Verify user is a participant of the conversation
        let (_, is_member) = self
            .conversation_repo
            .get_conversation_and_check_membership(&conversation_id, &user_id, tx.as_mut())
            .await?;

        ensure_conversation_member(is_member)?;

        // Get last message of the conversation
        let last_message = self
            .message_repo
            .get_last_message_by_conversation(&conversation_id, tx.as_mut())
            .await?;

        if let Some(msg) = last_message {
            // Check if user is the sender of the last message
            if msg.sender_id == user_id {
                // Sender doesn't need to mark as seen
                tx.commit().await?;
                return Ok(());
            }

            // Mark as seen with the last message ID
            self.participant_repo
                .mark_as_seen(&conversation_id, &user_id, &msg.id, tx.as_mut())
                .await?;

            let unread_counts = self
                .participant_repo
                .get_unread_counts(&conversation_id, tx.as_mut())
                .await?;

            let participants = self
                .participant_repo
                .find_participants_by_conversation_id(&[conversation_id], tx.as_mut())
                .await?;

            tx.commit().await?;

            let sender_info = participants
                .iter()
                .find(|participant| participant.user_id == msg.sender_id)
                .map(|participant| SenderInfo {
                    _id: msg.sender_id,
                    display_name: participant.display_name.clone(),
                    avatar_url: participant.avatar_url.clone(),
                })
                .unwrap_or(SenderInfo {
                    _id: msg.sender_id,
                    display_name: String::new(),
                    avatar_url: None,
                });

            // Broadcast read-message event với format tương thích Socket.IO
            let last_message_info = LastMessageInfo {
                _id: msg.id,
                content: msg.content.clone(),
                created_at: msg.created_at.to_rfc3339(),
                sender: sender_info,
            };

            let unread_counts_json: serde_json::Value = unread_counts
                .iter()
                .map(|(k, v)| (k.to_string(), serde_json::Value::Number((*v).into())))
                .collect();

            // Tạo conversation update info
            let conversation_update = serde_json::json!({
                "_id": conversation_id,
                "unreadCounts": unread_counts_json,
                "seenBy": [user_id]
            });

            self.ws_server.broadcast_to_room(
                conversation_id,
                &ServerMessage::read_message(conversation_update, last_message_info),
                None,
            );

            self.metrics.inc_conversation_mark_seen();
        }
        Ok(())
    }

    /// Cập nhật thông tin nhóm (tên, avatar) - Chỉ trưởng nhóm mới có quyền
    pub async fn update_group_info(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        avatar_url: Option<Option<String>>,
    ) -> Result<(), error::SystemError> {
        let mut tx = self.begin_tx().await?;

        // 1. Kiểm tra conversation-type và membership
        let (conv, is_member) = self
            .conversation_repo
            .get_conversation_and_check_membership(&conversation_id, &user_id, tx.as_mut())
            .await?;

        let conv = conv.ok_or_else(|| {
            error::SystemError::not_found_key(messages::i18n::Key::ConversationNotFound)
        })?;
        ensure_group_conversation(
            conv._type,
            messages::i18n::Key::GroupUpdateOnlyForGroup,
        )?;
        ensure_conversation_member(is_member)?;

        // 2. Kiểm tra quyền trưởng nhóm (creator)
        let creator_id = self
            .conversation_repo
            .get_group_creator(&conversation_id, tx.as_mut())
            .await?
            .ok_or_else(|| {
                error::SystemError::internal_error_key(messages::i18n::Key::GroupCreatorMissing)
            })?;

        ensure_group_owner(creator_id, user_id, messages::i18n::Key::GroupOwnerOnlyUpdate)?;

        // 3. Thực hiện cập nhật
        self.conversation_repo
            .update_group_info(
                &conversation_id,
                name.as_deref(),
                avatar_url.as_ref().map(|opt| opt.as_deref()),
                tx.as_mut(),
            )
            .await?;

        self.conversation_repo
            .update_timestamp(&conversation_id, tx.as_mut())
            .await?;

        tx.commit().await?;

        // 4. Broadcast WS tới tất cả thành viên trong nhóm
        self.ws_server.broadcast_to_room(
            conversation_id,
            &ServerMessage::GroupUpdated {
                conversation_id,
                name,
                avatar_url,
            },
            None,
        );

        self.metrics.inc_conversation_group_update();

        Ok(())
    }

    /// Thêm thành viên vào nhóm (Chỉ trưởng nhóm mới có quyền)
    pub async fn add_member(
        &self,
        conversation_id: Uuid,
        requester_id: Uuid,
        new_user_id: Uuid,
        is_friend: bool,
    ) -> Result<(), error::SystemError> {
        if !is_friend {
            return Err(error::SystemError::forbidden_key(
                messages::i18n::Key::GroupAddOnlyFriends,
            ));
        }

        let mut tx = self.begin_tx().await?;

        // 1. Kiểm tra quyền trưởng nhóm
        let creator_id = self
            .conversation_repo
            .get_group_creator(&conversation_id, tx.as_mut())
            .await?
            .ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::GroupNotFoundOrInvalid)
            })?;

        ensure_group_owner(
            creator_id,
            requester_id,
            messages::i18n::Key::GroupOwnerOnlyAdd,
        )?;

        // 2. Thêm thành viên (UPSERT)
        self.conversation_repo
            .add_participant(&conversation_id, &new_user_id, tx.as_mut())
            .await?;

        self.conversation_repo
            .update_timestamp(&conversation_id, tx.as_mut())
            .await?;

        // Lấy thông tin user mới để broadcast
        let user_info = sqlx::query_as::<_, ParticipantRow>(
            "SELECT display_name, avatar_url FROM users WHERE id = $1",
        )
        .bind(new_user_id)
        .fetch_one(tx.as_mut())
        .await
        .map_err(|_| error::SystemError::not_found_key(messages::i18n::Key::AddedUserNotFound))?;

        tx.commit().await?;

        // 3. Broadcast WS
        let conversation_detail = self
            .conversation_repo
            .find_one_conversation_detail(&conversation_id)
            .await?;

        if let Some(conversation_detail) = conversation_detail {
            let conversation_json = serde_json::to_value(&conversation_detail).map_err(|e| {
                error::SystemError::internal_error(format!(
                    "Lỗi khi xử lý dữ liệu cuộc trò chuyện: {}",
                    e
                ))
            })?;

            self.ws_server.send_to_users(
                &[new_user_id],
                &ServerMessage::NewGroup {
                    conversation: conversation_json,
                },
            );
        }

        self.ws_server.broadcast_to_room(
            conversation_id,
            &ServerMessage::MemberAdded {
                conversation_id,
                user_id: new_user_id,
                display_name: user_info.display_name,
                avatar_url: user_info.avatar_url,
            },
            None,
        );

        self.metrics.inc_conversation_member_add();

        Ok(())
    }

    /// Xóa thành viên hoặc tự rời nhóm
    pub async fn remove_member(
        &self,
        conversation_id: Uuid,
        requester_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), error::SystemError> {
        let mut tx = self.begin_tx().await?;

        // 1. Kiểm tra conversation-type
        let conv = self
            .conversation_repo
            .find_by_id(&conversation_id, tx.as_mut())
            .await?
            .ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::ConversationNotFound)
            })?;

        ensure_group_conversation(conv._type, messages::i18n::Key::GroupRemoveOnlyGroup)?;

        // 2. Kiểm tra quyền
        let creator_id = self
            .conversation_repo
            .get_group_creator(&conversation_id, tx.as_mut())
            .await?
            .ok_or_else(|| {
                error::SystemError::internal_error_key(messages::i18n::Key::GroupDataError)
            })?;

        ensure_member_removal_permission(requester_id, target_user_id, creator_id)?;

        // 3. Soft delete participant
        self.conversation_repo
            .remove_participant(&conversation_id, &target_user_id, tx.as_mut())
            .await?;

        self.conversation_repo
            .update_timestamp(&conversation_id, tx.as_mut())
            .await?;

        tx.commit().await?;

        // 4. Broadcast WS
        self.ws_server.broadcast_to_room(
            conversation_id,
            &ServerMessage::MemberRemoved {
                conversation_id,
                user_id: target_user_id,
            },
            None,
        );

        self.metrics.inc_conversation_member_remove();

        Ok(())
    }
}

fn ensure_conversation_member(is_member: bool) -> Result<(), error::SystemError> {
    if is_member {
        return Ok(());
    }

    Err(error::SystemError::forbidden_key(
        messages::i18n::Key::NotConversationMember,
    ))
}

fn ensure_group_conversation(
    conversation_type: ConversationType,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if conversation_type == ConversationType::Group {
        return Ok(());
    }

    Err(error::SystemError::bad_request_key(error_key))
}

fn ensure_group_owner(
    creator_id: Uuid,
    requester_id: Uuid,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if creator_id == requester_id {
        return Ok(());
    }

    Err(error::SystemError::forbidden_key(error_key))
}

fn ensure_member_removal_permission(
    requester_id: Uuid,
    target_user_id: Uuid,
    creator_id: Uuid,
) -> Result<(), error::SystemError> {
    if requester_id != target_user_id && requester_id != creator_id {
        return Err(error::SystemError::forbidden_key(
            messages::i18n::Key::AccessDenied,
        ));
    }

    if target_user_id == creator_id {
        return Err(error::SystemError::bad_request_key(
            messages::i18n::Key::CannotRemoveGroupOwner,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_conversation_member_rejects_non_member() {
        let result = ensure_conversation_member(false);
        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }

    #[test]
    fn ensure_group_conversation_rejects_non_group_type() {
        let result = ensure_group_conversation(
            ConversationType::Direct,
            messages::i18n::Key::GroupUpdateOnlyForGroup,
        );
        assert!(matches!(
            result,
            Err(error::SystemError::BadRequest(_) | error::SystemError::BadRequestKey(_))
        ));
    }

    #[test]
    fn ensure_group_owner_rejects_non_owner() {
        let result = ensure_group_owner(
            Uuid::now_v7(),
            Uuid::now_v7(),
            messages::i18n::Key::GroupOwnerOnlyAdd,
        );
        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }

    #[test]
    fn ensure_member_removal_permission_rejects_unauthorized_requester() {
        let creator_id = Uuid::now_v7();
        let requester_id = Uuid::now_v7();
        let target_id = Uuid::now_v7();

        let result = ensure_member_removal_permission(requester_id, target_id, creator_id);
        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }

    #[test]
    fn ensure_member_removal_permission_rejects_removing_creator() {
        let creator_id = Uuid::now_v7();

        let result = ensure_member_removal_permission(creator_id, creator_id, creator_id);
        assert!(matches!(
            result,
            Err(error::SystemError::BadRequest(_) | error::SystemError::BadRequestKey(_))
        ));
    }
}
