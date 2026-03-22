use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{error, messages},
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
    observability::AppMetrics,
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
    metrics: Arc<AppMetrics>,
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
        Self::with_dependencies_and_metrics(
            call_repo,
            participant_repo,
            ws_server,
            Arc::new(AppMetrics::default()),
        )
    }

    pub fn with_dependencies_and_metrics(
        call_repo: Arc<C>,
        participant_repo: Arc<P>,
        ws_server: Arc<WebSocketServer>,
        metrics: Arc<AppMetrics>,
    ) -> Self {
        Self {
            call_repo,
            participant_repo,
            ws_server,
            metrics,
        }
    }

    async fn begin_tx(
        &self,
    ) -> Result<Option<sqlx::Transaction<'_, sqlx::Postgres>>, error::SystemError> {
        if !self.call_repo.supports_transactions() {
            return Ok(None);
        }

        self.call_repo
            .get_pool()
            .begin()
            .await
            .map(Some)
            .map_err(Into::into)
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

        ensure_call_member(
            is_member,
            messages::i18n::Key::NotConversationMember,
        )?;

        let call = if let Some(mut tx) = self.begin_tx().await? {
            let call = self
                .call_repo
                .create_call_with_tx(
                    &mut tx,
                    user_id,
                    request.conversation_id,
                    request.call_type.clone(),
                )
                .await?;

            self.participant_repo
                .add_participant_with_tx(&mut tx, call.id, user_id)
                .await?;

            tx.commit().await?;
            call
        } else {
            let call = self
                .call_repo
                .create_call(user_id, request.conversation_id, request.call_type.clone())
                .await?;

            self.participant_repo
                .add_participant(call.id, user_id)
                .await?;

            call
        };

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

        self.metrics.inc_call_initiate();

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
        let call =
            self.call_repo.find_by_id(call_id).await?.ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::CallNotFound)
            })?;

        let is_member = self
            .call_repo
            .is_user_in_conversation(call.conversation_id, user_id)
            .await?;

        ensure_call_member(is_member, messages::i18n::Key::CallResponseNotAllowed)?;
        ensure_call_status(
            call.status,
            CallStatus::Initiated,
            messages::i18n::Key::CallNotAwaitingResponse,
        )?;

        let member_ids = self
            .call_repo
            .get_conversation_member_ids(call.conversation_id)
            .await?;

        if request.accept {
            if let Some(mut tx) = self.begin_tx().await? {
                self.call_repo
                    .update_call_status_with_tx(&mut tx, call_id, CallStatus::Accepted)
                    .await?;

                self.participant_repo
                    .add_participant_with_tx(&mut tx, call_id, user_id)
                    .await?;

                tx.commit().await?;
            } else {
                self.call_repo
                    .update_call_status(call_id, CallStatus::Accepted)
                    .await?;

                self.participant_repo
                    .add_participant(call_id, user_id)
                    .await?;
            }

            self.ws_server.send_to_users(
                &member_ids,
                &ServerMessage::CallAccept {
                    call_id,
                    responder_id: user_id,
                },
            );

            self.metrics.inc_call_accept();
        } else {
            if let Some(mut tx) = self.begin_tx().await? {
                self.call_repo
                    .update_call_status_with_tx(&mut tx, call_id, CallStatus::Rejected)
                    .await?;

                self.call_repo
                    .create_call_message_with_tx(
                        &mut tx,
                        call.conversation_id,
                        user_id,
                        MessageType::CallReject,
                        Some(build_call_reject_message(
                            &call.call_type,
                            request.reason.as_deref(),
                        )),
                    )
                    .await?;

                tx.commit().await?;
            } else {
                self.call_repo
                    .update_call_status(call_id, CallStatus::Rejected)
                    .await?;

                self.call_repo
                    .create_call_message(
                        call.conversation_id,
                        user_id,
                        MessageType::CallReject,
                        Some(build_call_reject_message(
                            &call.call_type,
                            request.reason.as_deref(),
                        )),
                    )
                    .await?;
            }

            self.ws_server.send_to_users(
                &member_ids,
                &ServerMessage::CallReject {
                    call_id,
                    reason: request.reason,
                    rejected_by: user_id,
                },
            );

            self.metrics.inc_call_reject();
        }

        Ok(())
    }

    pub async fn cancel_call(
        &self,
        user_id: Uuid,
        call_id: Uuid,
    ) -> Result<(), error::SystemError> {
        let call =
            self.call_repo.find_by_id(call_id).await?.ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::CallNotFound)
            })?;

        ensure_call_initiator(
            call.initiator_id,
            user_id,
            messages::i18n::Key::CallCancelInitiatorOnly,
        )?;
        ensure_call_status(
            call.status,
            CallStatus::Initiated,
            messages::i18n::Key::CallCancelInvalidStatus,
        )?;

        if let Some(mut tx) = self.begin_tx().await? {
            self.call_repo.end_call_with_tx(&mut tx, call_id, 0).await?;

            self.call_repo
                .create_call_message_with_tx(
                    &mut tx,
                    call.conversation_id,
                    user_id,
                    MessageType::CallCancel,
                    Some(format!(
                        "Cuộc gọi {} đã bị hủy",
                        call_type_label(&call.call_type)
                    )),
                )
                .await?;

            tx.commit().await?;
        } else {
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
        }

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

        self.metrics.inc_call_cancel();

        Ok(())
    }

    pub async fn end_call(&self, user_id: Uuid, call_id: Uuid) -> Result<(), error::SystemError> {
        let call =
            self.call_repo.find_by_id(call_id).await?.ok_or_else(|| {
                error::SystemError::not_found_key(messages::i18n::Key::CallNotFound)
            })?;

        let is_member = self
            .call_repo
            .is_user_in_conversation(call.conversation_id, user_id)
            .await?;

        ensure_call_member(is_member, messages::i18n::Key::CallEndNotAllowed)?;

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

        if let Some(mut tx) = self.begin_tx().await? {
            self.call_repo
                .end_call_with_tx(&mut tx, call_id, duration_seconds)
                .await?;
            self.participant_repo
                .mark_left_with_tx(&mut tx, call_id, user_id)
                .await?;

            self.call_repo
                .create_call_message_with_tx(
                    &mut tx,
                    call.conversation_id,
                    user_id,
                    MessageType::CallEnd,
                    Some(build_call_end_message(&call.call_type, duration_seconds)),
                )
                .await?;

            tx.commit().await?;
        } else {
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
        }

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

        self.metrics.inc_call_end();

        Ok(())
    }

    pub async fn get_call_history(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<CallHistoryResponse, error::SystemError> {
        let safe_limit = limit.clamp(1, 50);
        let calls = self
            .call_repo
            .get_user_calls(user_id, safe_limit, cursor)
            .await?;

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

fn ensure_call_member(
    is_member: bool,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if is_member {
        return Ok(());
    }

    Err(error::SystemError::forbidden_key(error_key))
}

fn ensure_call_status(
    actual: CallStatus,
    expected: CallStatus,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if actual == expected {
        return Ok(());
    }

    Err(error::SystemError::bad_request_key(error_key))
}

fn ensure_call_initiator(
    initiator_id: Uuid,
    user_id: Uuid,
    error_key: messages::i18n::Key,
) -> Result<(), error::SystemError> {
    if initiator_id == user_id {
        return Ok(());
    }

    Err(error::SystemError::forbidden_key(error_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_call_member_rejects_non_member() {
        let result = ensure_call_member(false, messages::i18n::Key::CallEndNotAllowed);
        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }

    #[test]
    fn ensure_call_status_rejects_unexpected_state() {
        let result = ensure_call_status(
            CallStatus::Accepted,
            CallStatus::Initiated,
            messages::i18n::Key::CallNotAwaitingResponse,
        );
        assert!(matches!(
            result,
            Err(error::SystemError::BadRequest(_) | error::SystemError::BadRequestKey(_))
        ));
    }

    #[test]
    fn ensure_call_initiator_rejects_non_initiator() {
        let result = ensure_call_initiator(
            Uuid::now_v7(),
            Uuid::now_v7(),
            messages::i18n::Key::CallCancelInitiatorOnly,
        );
        assert!(matches!(
            result,
            Err(error::SystemError::Forbidden(_) | error::SystemError::ForbiddenKey(_))
        ));
    }
}
