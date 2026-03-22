use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::error,
    modules::{
        call::{
            model::{
                CallHistoryResponse, CallWithDetails, InitiateCallRequest, InitiateCallResponse,
                RespondCallRequest,
            },
            repository::{CallParticipantRepository, CallRepository},
            schema::{CallStatus, CallType},
        },
        message::schema::MessageType,
        websocket::{message::ServerMessage, server::WebSocketServer},
    },
};

#[derive(Clone)]
pub struct CallService<C, P>
where
    C: CallRepository + Send + Sync,
    P: CallParticipantRepository + Send + Sync,
{
    call_repo: Arc<C>,
    participant_repo: Arc<P>,
    ws_server: Arc<WebSocketServer>,
}

impl<C, P> CallService<C, P>
where
    C: CallRepository + Send + Sync,
    P: CallParticipantRepository + Send + Sync,
{
    pub fn with_dependencies(
        call_repo: Arc<C>,
        participant_repo: Arc<P>,
        ws_server: Arc<WebSocketServer>,
    ) -> Self {
        Self {
            call_repo,
            participant_repo,
            ws_server,
        }
    }

    pub async fn initiate_call(
        &self,
        user_id: Uuid,
        request: InitiateCallRequest,
        initiator_name: String,
        initiator_avatar: Option<String>,
    ) -> Result<InitiateCallResponse, error::SystemError> {
        let is_member = self
            .call_repo
            .is_user_in_conversation(request.conversation_id, user_id)
            .await?;

        if !is_member {
            return Err(error::SystemError::forbidden(
                "Bạn không phải thành viên của cuộc trò chuyện này",
            ));
        }

        let call = self
            .call_repo
            .create_call(user_id, request.conversation_id, request.call_type.clone())
            .await?;

        self.participant_repo.add_participant(call.id, user_id).await?;

        let member_ids = self
            .call_repo
            .get_conversation_member_ids(request.conversation_id)
            .await?;

        let receivers: Vec<Uuid> = member_ids.into_iter().filter(|id| *id != user_id).collect();

        if !receivers.is_empty() {
            self.ws_server.send_to_users(
                &receivers,
                &ServerMessage::CallRequest {
                    call_id: call.id,
                    conversation_id: request.conversation_id,
                    call_type: request.call_type.as_str().to_string(),
                    initiator_id: user_id,
                    initiator_name,
                    initiator_avatar,
                },
            );
        }

        Ok(InitiateCallResponse {
            call_id: call.id,
            status: CallStatus::Initiated,
        })
    }

    pub async fn respond_call(
        &self,
        user_id: Uuid,
        call_id: Uuid,
        request: RespondCallRequest,
    ) -> Result<(), error::SystemError> {
        let call = self
            .call_repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy cuộc gọi"))?;

        let is_member = self
            .call_repo
            .is_user_in_conversation(call.conversation_id, user_id)
            .await?;

        if !is_member {
            return Err(error::SystemError::forbidden(
                "Bạn không thể phản hồi cuộc gọi này",
            ));
        }

        if call.status != CallStatus::Initiated {
            return Err(error::SystemError::bad_request(
                "Cuộc gọi không còn ở trạng thái chờ phản hồi",
            ));
        }

        let member_ids = self
            .call_repo
            .get_conversation_member_ids(call.conversation_id)
            .await?;

        if request.accept {
            self.call_repo
                .update_call_status(call_id, CallStatus::Accepted)
                .await?;

            self.participant_repo.add_participant(call_id, user_id).await?;

            self.ws_server.send_to_users(
                &member_ids,
                &ServerMessage::CallAccept {
                    call_id,
                    responder_id: user_id,
                },
            );
        } else {
            self.call_repo
                .update_call_status(call_id, CallStatus::Rejected)
                .await?;

            self.call_repo
                .create_call_message(
                    call.conversation_id,
                    user_id,
                    MessageType::CallReject,
                    Some(build_call_reject_message(&call.call_type, request.reason.as_deref())),
                )
                .await?;

            self.ws_server.send_to_users(
                &member_ids,
                &ServerMessage::CallReject {
                    call_id,
                    reason: request.reason,
                    rejected_by: user_id,
                },
            );
        }

        Ok(())
    }

    pub async fn cancel_call(&self, user_id: Uuid, call_id: Uuid) -> Result<(), error::SystemError> {
        let call = self
            .call_repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy cuộc gọi"))?;

        if call.initiator_id != user_id {
            return Err(error::SystemError::forbidden(
                "Chỉ người gọi mới có thể hủy cuộc gọi",
            ));
        }

        if call.status != CallStatus::Initiated {
            return Err(error::SystemError::bad_request(
                "Không thể hủy cuộc gọi ở trạng thái hiện tại",
            ));
        }

        self.call_repo.end_call(call_id, 0).await?;

        self.call_repo
            .create_call_message(
                call.conversation_id,
                user_id,
                MessageType::CallCancel,
                Some(format!(
                    "Cuộc gọi {} đã bị hủy",
                    call_type_label(&call.call_type)
                )),
            )
            .await?;

        let member_ids = self
            .call_repo
            .get_conversation_member_ids(call.conversation_id)
            .await?;

        self.ws_server.send_to_users(
            &member_ids,
            &ServerMessage::CallCancel {
                call_id,
                canceled_by: user_id,
            },
        );

        Ok(())
    }

    pub async fn end_call(&self, user_id: Uuid, call_id: Uuid) -> Result<(), error::SystemError> {
        let call = self
            .call_repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy cuộc gọi"))?;

        let is_member = self
            .call_repo
            .is_user_in_conversation(call.conversation_id, user_id)
            .await?;

        if !is_member {
            return Err(error::SystemError::forbidden(
                "Bạn không thể kết thúc cuộc gọi này",
            ));
        }

        if call.status == CallStatus::Ended {
            return Ok(());
        }

        let duration_seconds = call
            .started_at
            .map(|started| {
                let elapsed = (Utc::now() - started).num_seconds();
                elapsed.clamp(0, i64::from(i32::MAX)) as i32
            })
            .unwrap_or(0);

        self.call_repo.end_call(call_id, duration_seconds).await?;
        self.participant_repo.mark_left(call_id, user_id).await?;

        self.call_repo
            .create_call_message(
                call.conversation_id,
                user_id,
                MessageType::CallEnd,
                Some(build_call_end_message(&call.call_type, duration_seconds)),
            )
            .await?;

        let member_ids = self
            .call_repo
            .get_conversation_member_ids(call.conversation_id)
            .await?;

        self.ws_server.send_to_users(
            &member_ids,
            &ServerMessage::CallEnd {
                call_id,
                duration_seconds,
                ended_by: user_id,
            },
        );

        Ok(())
    }

    pub async fn get_call_history(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<CallHistoryResponse, error::SystemError> {
        let safe_limit = limit.clamp(1, 50);
        let calls = self.call_repo.get_user_calls(user_id, safe_limit, cursor).await?;

        let next_cursor = calls
            .last()
            .map(|call: &CallWithDetails| call.created_at.to_rfc3339());

        Ok(CallHistoryResponse {
            calls,
            cursor: next_cursor,
        })
    }
}

fn call_type_label(call_type: &CallType) -> &'static str {
    match call_type {
        CallType::Audio => "thoại",
        CallType::Video => "video",
    }
}

fn build_call_reject_message(call_type: &CallType, reason: Option<&str>) -> String {
    let base = format!("Cuộc gọi {} đã bị từ chối", call_type_label(call_type));
    match reason.map(str::trim) {
        Some(value) if !value.is_empty() => format!("{base}: {value}"),
        _ => base,
    }
}

fn build_call_end_message(call_type: &CallType, duration_seconds: i32) -> String {
    if duration_seconds > 0 {
        return format!(
            "Cuộc gọi {} đã kết thúc • {}",
            call_type_label(call_type),
            format_duration(duration_seconds)
        );
    }

    format!("Cuộc gọi {} đã kết thúc", call_type_label(call_type))
}

fn format_duration(duration_seconds: i32) -> String {
    let minutes = duration_seconds / 60;
    let seconds = duration_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
