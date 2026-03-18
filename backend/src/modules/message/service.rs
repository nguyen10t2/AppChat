/// Message Service
///
/// Service layer xử lý business logic cho messages, bao gồm:
/// - Gửi tin nhắn (direct và group)
/// - Xóa và chỉnh sửa tin nhắn
/// - Broadcast real-time qua WebSocket
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::api::error;
use crate::configs::RedisCache;
use crate::METRICS;
use crate::modules::conversation::model::NewLastMessage;
use crate::modules::conversation::repository::{
    ConversationRepository, LastMessageRepository, ParticipantRepository,
};
use crate::modules::conversation::schema::ConversationType;
use crate::modules::message::model::{InsertMessage, SendDirectMessagePayload};
use crate::modules::message::repository::MessageRepository;
use crate::modules::message::schema::{MessageEntity, MessageType};
use crate::modules::websocket::message::{LastMessageInfo, SenderInfo, ServerMessage};
use crate::modules::websocket::server::WebSocketServer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MessageRoute {
    Group,
    Direct { recipient_id: Uuid },
}

/// Message service với generic repositories để dễ testing
#[derive(Clone)]
pub struct MessageService<M, C, P, L>
where
    M: MessageRepository + Send + Sync,
    C: ConversationRepository + Send + Sync,
    P: ParticipantRepository + Send + Sync,
    L: LastMessageRepository + Send + Sync,
{
    message_repo: Arc<M>,
    conversation_repo: Arc<C>,
    participant_repo: Arc<P>,
    last_message_repo: Arc<L>,
    cache: Arc<RedisCache>,
    ws_server: Arc<WebSocketServer>,
}

impl<M, C, P, L> MessageService<M, C, P, L>
where
    C: ConversationRepository + Send + Sync,
    M: MessageRepository + Send + Sync,
    P: ParticipantRepository + Send + Sync,
    L: LastMessageRepository + Send + Sync,
{
    /// Tạo MessageService với các dependencies
    pub fn with_dependencies(
        conversation_repo: Arc<C>,
        message_repo: Arc<M>,
        participant_repo: Arc<P>,
        last_message_repo: Arc<L>,
        cache: Arc<RedisCache>,
        ws_server: Arc<WebSocketServer>,
    ) -> Self {
        MessageService {
            conversation_repo,
            message_repo,
            participant_repo,
            last_message_repo,
            cache,
            ws_server,
        }
    }

    /// Gửi tin nhắn vào một conversation đã có sẵn (dùng cho WebSocket)
    ///
    /// Flow:
    /// 1. Kiểm tra conversation tồn tại + sender là thành viên
    /// 2. Route theo loại conversation (direct/group)
    /// 3. Dùng cùng business flow với REST để tránh lệch logic
    pub async fn send_message_to_conversation(
        &self,
        sender_id: Uuid,
        conversation_id: Uuid,
        content: String,
    ) -> Result<MessageEntity, error::SystemError> {
        let (conversation, is_member) = self
            .conversation_repo
            .get_conversation_and_check_membership(
                &conversation_id,
                &sender_id,
                self.conversation_repo.get_pool(),
            )
            .await?;

        let conversation = conversation
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy cuộc trò chuyện"))?;

        if !is_member {
            return Err(error::SystemError::forbidden(
                "Bạn không phải thành viên của cuộc trò chuyện này",
            ));
        }

        let route = match conversation._type {
            ConversationType::Group => MessageRoute::Group,
            ConversationType::Direct => {
                let participants = self
                    .participant_repo
                    .find_participants_by_conversation_id(
                        &[conversation_id],
                        self.conversation_repo.get_pool(),
                    )
                    .await?;

                Self::resolve_message_route(
                    &ConversationType::Direct,
                    sender_id,
                    participants.iter().map(|participant| participant.user_id),
                )?
            }
        };

        match route {
            MessageRoute::Group => self.send_group_message(sender_id, content, conversation_id).await,
            MessageRoute::Direct { recipient_id } => {
                self.send_direct_message(sender_id, recipient_id, content, Some(conversation_id))
                    .await
            }
        }
    }

    pub(crate) fn resolve_message_route<I>(
        conversation_type: &ConversationType,
        sender_id: Uuid,
        participant_ids: I,
    ) -> Result<MessageRoute, error::SystemError>
    where
        I: IntoIterator<Item = Uuid>,
    {
        match conversation_type {
            ConversationType::Group => Ok(MessageRoute::Group),
            ConversationType::Direct => participant_ids
                .into_iter()
                .find(|user_id| *user_id != sender_id)
                .map(|recipient_id| MessageRoute::Direct { recipient_id })
                .ok_or_else(|| {
                    error::SystemError::bad_request(
                        "Không thể xác định người nhận trong cuộc trò chuyện trực tiếp",
                    )
                }),
        }
    }

    /// Gửi direct message giữa 2 users
    ///
    /// Flow:
    /// 1. Tìm hoặc tạo conversation
    /// 2. Tạo message trong DB
    /// 3. Increment unread count cho recipient
    /// 4. Upsert last message
    /// 5. Broadcast qua WebSocket
    pub async fn send_direct_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        content: String,
        conversation_id: Option<Uuid>,
    ) -> Result<MessageEntity, error::SystemError> {
        self.send_direct_message_payload(
            sender_id,
            recipient_id,
            SendDirectMessagePayload {
                conversation_id,
                content: Some(content),
                message_type: None,
                file_url: None,
                reply_to_id: None,
            },
        )
        .await
    }

    pub async fn send_direct_message_payload(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        payload: SendDirectMessagePayload,
    ) -> Result<MessageEntity, error::SystemError> {
        let started_at = Instant::now();
        let mut tx = self.conversation_repo.get_pool().begin().await?;

        let (message_type, content, file_url) =
            Self::normalize_message_input(payload.content, payload.message_type, payload.file_url)?;

        let conversation = match payload.conversation_id {
            Some(conv_id) => self
                .conversation_repo
                .find_by_id(&conv_id, self.conversation_repo.get_pool())
                .await?
                .ok_or_else(|| error::SystemError::not_found("Không tìm thấy cuộc trò chuyện"))?,
            None => self
                .conversation_repo
                .find_direct_between_users(&sender_id, &recipient_id, tx.as_mut())
                .await?
                .unwrap_or(
                    self.conversation_repo
                        .create_direct_conversation(&sender_id, &recipient_id, &mut tx)
                        .await?,
                ),
        };

        self.validate_reply_target(payload.reply_to_id, conversation.id, tx.as_mut())
            .await?;

        let message = self
            .message_repo
            .create(
                &InsertMessage {
                    conversation_id: conversation.id,
                    sender_id,
                    reply_to_id: payload.reply_to_id,
                    _type: message_type,
                    content: content.clone(),
                    file_url: file_url.clone(),
                },
                tx.as_mut(),
            )
            .await?;

        self.participant_repo
            .increment_unread_count(&conversation.id, &recipient_id, tx.as_mut())
            .await?;

        self.last_message_repo
            .upsert_last_message(
                &NewLastMessage {
                    conversation_id: conversation.id,
                    sender_id,
                    content: content.clone(),
                    created_at: message.created_at,
                },
                tx.as_mut(),
            )
            .await?;

        self.conversation_repo
            .update_timestamp(&conversation.id, tx.as_mut())
            .await?;

        // Get unread counts for all participants
        let unread_counts = self
            .participant_repo
            .get_unread_counts(&conversation.id, tx.as_mut())
            .await?;

        tx.commit().await?;

        let sender_info = self
            .build_sender_info(conversation.id, sender_id)
            .await
            .unwrap_or(SenderInfo {
                _id: sender_id,
                display_name: String::new(),
                avatar_url: None,
            });

        // Build and broadcast new message
        let server_message = self.build_new_message_event(&message, &unread_counts, sender_info);
        let participant_ids = vec![sender_id, recipient_id];
        self.ws_server
            .send_to_users(&participant_ids, &server_message);

        METRICS.record_message_send_latency(started_at.elapsed());

        Ok(message)
    }

    /// Gửi group message
    ///
    /// Flow:
    /// 1. Tạo message trong DB
    /// 2. Increment unread count cho tất cả participants (trừ sender)
    /// 3. Upsert last message
    /// 4. Broadcast qua WebSocket
    pub async fn send_group_message(
        &self,
        sender_id: Uuid,
        content: String,
        conversation_id: Uuid,
    ) -> Result<MessageEntity, error::SystemError> {
        self.send_group_message_payload(
            sender_id,
            conversation_id,
            Some(content),
            None,
            None,
            None,
        )
        .await
    }

    pub async fn send_group_message_payload(
        &self,
        sender_id: Uuid,
        conversation_id: Uuid,
        content: Option<String>,
        message_type: Option<MessageType>,
        file_url: Option<String>,
        reply_to_id: Option<Uuid>,
    ) -> Result<MessageEntity, error::SystemError> {
        let started_at = Instant::now();
        let mut tx = self.conversation_repo.get_pool().begin().await?;

        let (message_type, content, file_url) =
            Self::normalize_message_input(content, message_type, file_url)?;

        self.validate_reply_target(reply_to_id, conversation_id, tx.as_mut())
            .await?;

        let message = self
            .message_repo
            .create(
                &InsertMessage {
                    conversation_id,
                    sender_id,
                    reply_to_id,
                    _type: message_type,
                    content: content.clone(),
                    file_url: file_url.clone(),
                },
                tx.as_mut(),
            )
            .await?;

        self.participant_repo
            .increment_unread_count_for_others(&conversation_id, &sender_id, tx.as_mut())
            .await?;

        self.last_message_repo
            .upsert_last_message(
                &NewLastMessage {
                    conversation_id,
                    sender_id,
                    content,
                    created_at: message.created_at,
                },
                tx.as_mut(),
            )
            .await?;

        self.conversation_repo
            .update_timestamp(&conversation_id, tx.as_mut())
            .await?;

        // Get unread counts for all participants
        let unread_counts = self
            .participant_repo
            .get_unread_counts(&conversation_id, tx.as_mut())
            .await?;

        tx.commit().await?;

        let sender_info = self
            .build_sender_info(conversation_id, sender_id)
            .await
            .unwrap_or(SenderInfo {
                _id: sender_id,
                display_name: String::new(),
                avatar_url: None,
            });

        // Build and broadcast new message
        let server_message = self.build_new_message_event(&message, &unread_counts, sender_info);
        let participant_ids: Vec<Uuid> = unread_counts.keys().copied().collect();
        self.ws_server
            .send_to_users(&participant_ids, &server_message);

        METRICS.record_message_send_latency(started_at.elapsed());

        Ok(message)
    }

    /// Xóa message (soft delete)
    ///
    /// Chỉ sender mới có thể xóa message của mình
    pub async fn delete_message(
        &self,
        message_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), error::SystemError> {
        let mut tx = self.conversation_repo.get_pool().begin().await?;

        let message = self
            .message_repo
            .find_by_id(&message_id, tx.as_mut())
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy tin nhắn"))?;

        if message.sender_id != user_id {
            return Err(error::SystemError::forbidden(
                "Bạn chỉ có thể xóa tin nhắn của chính mình",
            ));
        }

        let deleted = self
            .message_repo
            .delete_message(&message_id, &user_id, tx.as_mut())
            .await?;

        if !deleted {
            return Err(error::SystemError::not_found(
                "Không tìm thấy tin nhắn hoặc tin nhắn đã bị xóa",
            ));
        }

        let participants = self
            .participant_repo
            .find_participants_by_conversation_id(&[message.conversation_id], tx.as_mut())
            .await?;
        let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

        tx.commit().await?;

        self.ws_server.send_to_users(
            &participant_ids,
            &ServerMessage::MessageDeleted {
                conversation_id: message.conversation_id,
                message_id,
            },
        );

        Ok(())
    }

    /// Chỉnh sửa message
    ///
    /// Chỉ sender mới có thể edit message của mình
    pub async fn edit_message(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        new_content: String,
    ) -> Result<MessageEntity, error::SystemError> {
        let mut tx = self.conversation_repo.get_pool().begin().await?;

        let message = self
            .message_repo
            .find_by_id(&message_id, tx.as_mut())
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy tin nhắn"))?;

        if message.sender_id != user_id {
            return Err(error::SystemError::forbidden(
                "Bạn chỉ có thể chỉnh sửa tin nhắn của chính mình",
            ));
        }

        let edited_message = self
            .message_repo
            .edit_message(&message_id, &user_id, &new_content, tx.as_mut())
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy tin nhắn"))?;

        let participants = self
            .participant_repo
            .find_participants_by_conversation_id(&[message.conversation_id], tx.as_mut())
            .await?;
        let participant_ids: Vec<Uuid> = participants.into_iter().map(|p| p.user_id).collect();

        tx.commit().await?;

        self.ws_server.send_to_users(
            &participant_ids,
            &ServerMessage::MessageEdited {
                conversation_id: message.conversation_id,
                message_id,
                new_content,
            },
        );

        Ok(edited_message)
    }

    /// Helper: Build new-message event với format tương thích Socket.IO
    async fn build_sender_info(
        &self,
        conversation_id: Uuid,
        sender_id: Uuid,
    ) -> Result<SenderInfo, error::SystemError> {
        let participants = self
            .participant_repo
            .find_participants_by_conversation_id(
                &[conversation_id],
                self.conversation_repo.get_pool(),
            )
            .await?;

        let sender = participants
            .into_iter()
            .find(|participant| participant.user_id == sender_id);

        Ok(match sender {
            Some(participant) => SenderInfo {
                _id: sender_id,
                display_name: participant.display_name,
                avatar_url: participant.avatar_url,
            },
            None => SenderInfo {
                _id: sender_id,
                display_name: String::new(),
                avatar_url: None,
            },
        })
    }

    /// Helper: Build new-message event với format tương thích Socket.IO
    fn build_new_message_event(
        &self,
        message: &MessageEntity,
        unread_counts: &HashMap<Uuid, i32>,
        sender_info: SenderInfo,
    ) -> ServerMessage {
        let message_json = serde_json::to_value(message).unwrap_or_default();

        let last_message = LastMessageInfo {
            _id: message.id,
            content: message.content.clone(),
            created_at: message.created_at.to_rfc3339(),
            sender: sender_info,
        };

        // Convert HashMap<Uuid, i32> to JSON object with string keys
        let unread_counts_json: serde_json::Value = unread_counts
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::Number((*v).into())))
            .collect();

        ServerMessage::new_message(
            message_json,
            message.conversation_id,
            last_message,
            message.created_at.to_rfc3339(),
            unread_counts_json,
        )
    }

    async fn validate_reply_target<'e, E>(
        &self,
        reply_to_id: Option<Uuid>,
        conversation_id: Uuid,
        tx: E,
    ) -> Result<(), error::SystemError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let Some(reply_id) = reply_to_id else {
            return Ok(());
        };

        let reply_message = self
            .message_repo
            .find_by_id(&reply_id, tx)
            .await?
            .ok_or_else(|| error::SystemError::bad_request("Tin nhắn được trả lời không tồn tại"))?;

        if reply_message.conversation_id != conversation_id {
            return Err(error::SystemError::bad_request(
                "Tin nhắn được trả lời không thuộc cuộc trò chuyện này",
            ));
        }

        Ok(())
    }

    pub(crate) fn normalize_message_input(
        content: Option<String>,
        message_type: Option<MessageType>,
        file_url: Option<String>,
    ) -> Result<(MessageType, Option<String>, Option<String>), error::SystemError> {
        let normalized_content = content
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let normalized_file_url = file_url
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());

        if normalized_content.is_none() && normalized_file_url.is_none() {
            return Err(error::SystemError::bad_request(
                "Tin nhắn phải có nội dung hoặc tệp đính kèm",
            ));
        }

        let resolved_type = message_type.unwrap_or_else(|| {
            if normalized_file_url.is_some() {
                MessageType::File
            } else {
                MessageType::Text
            }
        });

        if matches!(resolved_type, MessageType::Image | MessageType::Video | MessageType::File)
            && normalized_file_url.is_none()
        {
            return Err(error::SystemError::bad_request(
                "Loại tin nhắn này yêu cầu file_url",
            ));
        }

        if matches!(resolved_type, MessageType::Text | MessageType::System)
            && normalized_content.is_none()
        {
            return Err(error::SystemError::bad_request(
                "Tin nhắn văn bản yêu cầu nội dung",
            ));
        }

        Ok((resolved_type, normalized_content, normalized_file_url))
    }
}
