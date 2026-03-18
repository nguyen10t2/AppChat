/// WebSocket Session Logic (Actorless)
///
/// Mỗi WebSocket connection liên kết với một `WebSocketSessionImpl`.
/// Struct này bao đóng các dependencies, trạng thái auth (user_id),
/// và cung cấp các async methods để xử lý messages từ client.
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::ENV;
use crate::modules::conversation::repository_pg::{
    ConversationPgRepository, LastMessagePgRepository, ParticipantPgRepository,
};
use crate::modules::friend::repository_pg::FriendRepositoryPg;
use crate::modules::message::repository_pg::MessageRepositoryPg;
use crate::modules::message::service::MessageService;
use crate::utils::{Claims, TypeClaims};

use super::message::{ClientMessage, ServerMessage};
use super::presence::PresenceService;
use super::server::WebSocketServer;

pub type MessageSvc = MessageService<
    MessageRepositoryPg,
    ConversationPgRepository,
    ParticipantPgRepository,
    LastMessagePgRepository,
>;

pub struct WebSocketSessionImpl {
    pub id: Uuid,
    pub correlation_id: String,
    pub user_id: Option<Uuid>,
    pub server: Arc<WebSocketServer>,
    pub tx: mpsc::UnboundedSender<String>,
    pub message_service: Option<Arc<MessageSvc>>,
    pub presence_service: Option<Arc<PresenceService>>,
    pub friend_repo: Option<Arc<FriendRepositoryPg>>,
    pub friend_ids: Vec<Uuid>,
}

impl WebSocketSessionImpl {
    pub fn new(
        server: Arc<WebSocketServer>,
        tx: mpsc::UnboundedSender<String>,
        correlation_id: String,
        message_service: Option<Arc<MessageSvc>>,
        presence_service: Option<Arc<PresenceService>>,
        friend_repo: Option<Arc<FriendRepositoryPg>>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            correlation_id,
            user_id: None,
            server,
            tx,
            message_service,
            presence_service,
            friend_repo,
            friend_ids: Vec::new(),
        }
    }

    /// Gửi một message tới client thông qua bộ đệm channel
    fn send_to_client(&self, msg: &ServerMessage) {
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = self.tx.send(json);
        } else {
            tracing::error!(
                correlation_id = %self.correlation_id,
                session_id = %self.id,
                "Không thể serialize ServerMessage"
            );
        }
    }

    /// Gửi error message
    fn send_error(&self, message: &str) {
        self.send_to_client(&ServerMessage::Error {
            message: message.to_string(),
        });
    }

    /// Trả về user_id nếu session đã xác thực
    fn require_auth(&self) -> Option<Uuid> {
        if self.user_id.is_none() {
            self.send_error("Bạn cần xác thực trước khi thực hiện thao tác này");
        }
        self.user_id
    }

    /// Xử lý một message nhận từ client
    pub async fn handle_client_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Auth { token } => {
                self.handle_auth(&token).await;
            }
            ClientMessage::SendMessage {
                conversation_id,
                content,
            } => {
                self.handle_send_message(conversation_id, content).await;
            }
            ClientMessage::JoinConversation { conversation_id } => {
                self.handle_join_conversation(conversation_id);
            }
            ClientMessage::LeaveConversation { conversation_id } => {
                self.handle_leave_conversation(conversation_id);
            }
            ClientMessage::TypingStart { conversation_id } => {
                self.handle_typing_start(conversation_id);
            }
            ClientMessage::TypingStop { conversation_id } => {
                self.handle_typing_stop(conversation_id);
            }
            ClientMessage::Ping => {
                // Heartbeat được quản lý riêng bằng text/ping qua ws frame.
                // Hàm này chỉ để tương thích với protocol gửi theo text.
                self.send_to_client(&ServerMessage::Pong);
            }
        }
    }

    /// Xác thực user với JWT, cập nhật trạng thái Redis và thông báo bạn bè
    async fn handle_auth(&mut self, token: &str) {
        if self.user_id.is_some() {
            self.send_error("Session đã được xác thực");
            return;
        }

        let claims = match Claims::decode(token, ENV.jwt_secret.as_ref()) {
            Ok(claims) => claims,
            Err(e) => {
                tracing::warn!(
                    correlation_id = %self.correlation_id,
                    session_id = %self.id,
                    error = %e,
                    "JWT verification thất bại"
                );
                self.send_to_client(&ServerMessage::AuthFailed {
                    reason: "Token không hợp lệ hoặc đã hết hạn".to_string(),
                });
                return;
            }
        };

        if claims._type.as_ref() != Some(&TypeClaims::AccessToken) {
            self.send_to_client(&ServerMessage::AuthFailed {
                reason: "Chỉ chấp nhận access token".to_string(),
            });
            return;
        }

        let user_id = claims.sub;
        self.user_id = Some(user_id);
        self.server.authenticate(self.id, user_id);
        self.send_to_client(&ServerMessage::AuthSuccess { user_id });

        tracing::info!(
            correlation_id = %self.correlation_id,
            session_id = %self.id,
            user_id = %user_id,
            "WebSocket auth thành công"
        );

        let friend_ids = if let Some(repo) = &self.friend_repo {
            repo.find_friend_ids(&user_id).await.unwrap_or_else(|e| {
                tracing::error!("Lỗi load friend IDs cho user {}: {}", user_id, e);
                vec![]
            })
        } else {
            vec![]
        };

        self.friend_ids = friend_ids.clone();

        if let Some(presence) = &self.presence_service
            && let Err(e) = presence.set_online(user_id).await
        {
            tracing::error!("Lỗi set Redis presence cho user {}: {}", user_id, e);
        }

        if !self.friend_ids.is_empty() {
            self.server
                .user_presence_changed(user_id, true, &self.friend_ids, None);
            self.server
                .send_initial_presence(&user_id, &self.friend_ids);
        }
    }

    /// Xử lý gửi tin nhắn, lưu DB và broadcast
    async fn handle_send_message(&self, conversation_id: Uuid, content: String) {
        let Some(user_id) = self.require_auth() else {
            return;
        };

        let Some(service) = &self.message_service else {
            self.send_error("Message service không khả dụng");
            return;
        };

        match service
            .send_message_to_conversation(user_id, conversation_id, content)
            .await
        {
            Ok(msg_entity) => {
                tracing::info!(
                    correlation_id = %self.correlation_id,
                    session_id = %self.id,
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    message_id = %msg_entity.id,
                    "Message đã được xử lý qua flow thống nhất"
                );
            }
            Err(e) => {
                tracing::error!(
                    correlation_id = %self.correlation_id,
                    session_id = %self.id,
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    error = %e,
                    "Lỗi lưu message"
                );
                self.send_error("Không thể gửi tin nhắn. Vui lòng thử lại.");
            }
        }
    }

    fn handle_join_conversation(&self, conversation_id: Uuid) {
        if let Some(user_id) = self.require_auth() {
            self.server.join_room(user_id, conversation_id);
        }
    }

    fn handle_leave_conversation(&self, conversation_id: Uuid) {
        if let Some(user_id) = self.require_auth() {
            self.server.leave_room(user_id, conversation_id);
        }
    }

    fn handle_typing_start(&self, conversation_id: Uuid) {
        if let Some(user_id) = self.require_auth() {
            self.broadcast_typing_event(conversation_id, user_id, true);
        }
    }

    fn handle_typing_stop(&self, conversation_id: Uuid) {
        if let Some(user_id) = self.require_auth() {
            self.broadcast_typing_event(conversation_id, user_id, false);
        }
    }

    fn broadcast_typing_event(&self, conversation_id: Uuid, user_id: Uuid, is_typing: bool) {
        let message = if is_typing {
            ServerMessage::UserTyping {
                conversation_id,
                user_id,
            }
        } else {
            ServerMessage::UserStoppedTyping {
                conversation_id,
                user_id,
            }
        };

        self.server
            .broadcast_to_room(conversation_id, &message, Some(user_id));
    }
}
