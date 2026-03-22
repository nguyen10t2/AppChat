#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::{Duration as ChronoDuration, Utc};
    use serde_json::Value;
    use tokio::sync::mpsc;
    use tokio::time::{Duration, timeout};
    use uuid::Uuid;

    use crate::api::error;
    use crate::modules::call::model::{CallWithDetails, InitiateCallRequest, RespondCallRequest};
    use crate::modules::call::repository::{CallParticipantRepository, CallRepository};
    use crate::modules::call::schema::{CallEntity, CallParticipantEntity, CallStatus, CallType};
    use crate::modules::call::service::CallService;
    use crate::modules::message::schema::MessageType;
    use crate::modules::websocket::server::WebSocketServer;
    use crate::tests::mock::database::MockDatabase;

    #[derive(Default)]
    struct MockCallState {
        calls: HashMap<Uuid, CallEntity>,
        conversation_members: HashMap<Uuid, Vec<Uuid>>,
        user_calls: Vec<CallWithDetails>,
        status_updates: Vec<(Uuid, CallStatus)>,
        end_call_durations: Vec<i32>,
        last_history_limit: Option<i64>,
        call_messages: Vec<(Uuid, Uuid, MessageType, Option<String>)>,
    }

    #[derive(Clone)]
    struct MockCallRepo {
        pool: sqlx::PgPool,
        state: Arc<Mutex<MockCallState>>,
    }

    impl MockCallRepo {
        fn new(state: Arc<Mutex<MockCallState>>) -> Self {
            Self {
                pool: MockDatabase::new().pool(),
                state,
            }
        }
    }

    #[async_trait::async_trait]
    impl CallRepository for MockCallRepo {
        fn get_pool(&self) -> &sqlx::PgPool {
            &self.pool
        }

        async fn create_call(
            &self,
            initiator_id: Uuid,
            conversation_id: Uuid,
            call_type: CallType,
        ) -> Result<CallEntity, error::SystemError> {
            let call = CallEntity {
                id: Uuid::now_v7(),
                conversation_id,
                initiator_id,
                call_type,
                status: CallStatus::Initiated,
                started_at: None,
                ended_at: None,
                duration_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut state = self.state.lock().expect("call state mutex poisoned");
            state.calls.insert(call.id, call.clone());
            Ok(call)
        }

        async fn find_by_id(&self, call_id: Uuid) -> Result<Option<CallEntity>, error::SystemError> {
            let state = self.state.lock().expect("call state mutex poisoned");
            Ok(state.calls.get(&call_id).cloned())
        }

        async fn update_call_status(
            &self,
            call_id: Uuid,
            status: CallStatus,
        ) -> Result<Option<CallEntity>, error::SystemError> {
            let mut state = self.state.lock().expect("call state mutex poisoned");
            if let Some(call) = state.calls.get_mut(&call_id) {
                call.status = status.clone();
                if status == CallStatus::Accepted {
                    call.started_at = Some(Utc::now());
                }
                if status == CallStatus::Rejected {
                    call.ended_at = Some(Utc::now());
                }
                call.updated_at = Utc::now();
                let cloned = call.clone();
                let _ = call;
                state.status_updates.push((call_id, status));
                return Ok(Some(cloned));
            }

            Ok(None)
        }

        async fn end_call(
            &self,
            call_id: Uuid,
            duration_seconds: i32,
        ) -> Result<Option<CallEntity>, error::SystemError> {
            let mut state = self.state.lock().expect("call state mutex poisoned");
            if let Some(call) = state.calls.get_mut(&call_id) {
                call.status = CallStatus::Ended;
                call.ended_at = Some(Utc::now());
                call.duration_seconds = Some(duration_seconds);
                let cloned = call.clone();
                let _ = call;
                state.end_call_durations.push(duration_seconds);
                return Ok(Some(cloned));
            }

            Ok(None)
        }

        async fn get_conversation_member_ids(
            &self,
            conversation_id: Uuid,
        ) -> Result<Vec<Uuid>, error::SystemError> {
            let state = self.state.lock().expect("call state mutex poisoned");
            Ok(state
                .conversation_members
                .get(&conversation_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn is_user_in_conversation(
            &self,
            conversation_id: Uuid,
            user_id: Uuid,
        ) -> Result<bool, error::SystemError> {
            let state = self.state.lock().expect("call state mutex poisoned");
            Ok(state
                .conversation_members
                .get(&conversation_id)
                .map(|members| members.contains(&user_id))
                .unwrap_or(false))
        }

        async fn get_user_calls(
            &self,
            _user_id: Uuid,
            limit: i64,
            _cursor: Option<chrono::DateTime<Utc>>,
        ) -> Result<Vec<CallWithDetails>, error::SystemError> {
            let mut state = self.state.lock().expect("call state mutex poisoned");
            state.last_history_limit = Some(limit);
            Ok(state.user_calls.clone())
        }

        async fn create_call_message(
            &self,
            conversation_id: Uuid,
            sender_id: Uuid,
            message_type: MessageType,
            content: Option<String>,
        ) -> Result<(), error::SystemError> {
            let mut state = self.state.lock().expect("call state mutex poisoned");
            state
                .call_messages
                .push((conversation_id, sender_id, message_type, content));
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockParticipantRepo {
        added: Arc<Mutex<Vec<(Uuid, Uuid)>>>,
        left: Arc<Mutex<Vec<(Uuid, Uuid)>>>,
    }

    #[async_trait::async_trait]
    impl CallParticipantRepository for MockParticipantRepo {
        async fn add_participant(
            &self,
            call_id: Uuid,
            user_id: Uuid,
        ) -> Result<CallParticipantEntity, error::SystemError> {
            self.added
                .lock()
                .expect("participant mutex poisoned")
                .push((call_id, user_id));

            Ok(CallParticipantEntity {
                id: Uuid::now_v7(),
                call_id,
                user_id,
                joined_at: Some(Utc::now()),
                left_at: None,
            })
        }

        async fn mark_left(
            &self,
            call_id: Uuid,
            user_id: Uuid,
        ) -> Result<(), error::SystemError> {
            self.left
                .lock()
                .expect("participant mutex poisoned")
                .push((call_id, user_id));
            Ok(())
        }

        async fn is_call_participant(
            &self,
            call_id: Uuid,
            user_id: Uuid,
        ) -> Result<bool, error::SystemError> {
            let added = self.added.lock().expect("participant mutex poisoned");
            Ok(added.contains(&(call_id, user_id)))
        }
    }

    fn connect_ws_user(
        server: &Arc<WebSocketServer>,
        user_id: Uuid,
    ) -> mpsc::UnboundedReceiver<String> {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let session_id = Uuid::now_v7();
        server.connect(session_id, tx);
        server.authenticate(session_id, user_id);
        rx
    }

    async fn recv_json(rx: &mut mpsc::UnboundedReceiver<String>) -> Value {
        let payload = timeout(Duration::from_millis(300), rx.recv())
            .await
            .expect("timeout waiting websocket message")
            .expect("websocket channel closed unexpectedly");

        serde_json::from_str::<Value>(&payload).expect("invalid websocket json payload")
    }

    fn build_service(
        call_repo: MockCallRepo,
        participant_repo: MockParticipantRepo,
        ws_server: Arc<WebSocketServer>,
    ) -> CallService<MockCallRepo, MockParticipantRepo> {
        CallService::with_dependencies(Arc::new(call_repo), Arc::new(participant_repo), ws_server)
    }

    #[tokio::test]
    async fn initiate_call_emits_call_request_to_other_members() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();
        let receiver_id = Uuid::now_v7();

        let state = Arc::new(Mutex::new(MockCallState {
            conversation_members: HashMap::from([(conversation_id, vec![initiator_id, receiver_id])]),
            ..Default::default()
        }));

        let call_repo = MockCallRepo::new(state);
        let participant_repo = MockParticipantRepo::default();
        let ws_server = Arc::new(WebSocketServer::new());
        let mut receiver_rx = connect_ws_user(&ws_server, receiver_id);

        let service = build_service(call_repo, participant_repo.clone(), ws_server);

        let response = service
            .initiate_call(
                initiator_id,
                InitiateCallRequest {
                    conversation_id,
                    call_type: CallType::Audio,
                },
                "Alice".to_string(),
                None,
            )
            .await
            .expect("initiate_call should succeed");

        assert_eq!(response.status, CallStatus::Initiated);
        assert_ne!(response.call_id, Uuid::nil());

        let added = participant_repo
            .added
            .lock()
            .expect("participant mutex poisoned");
        assert!(added.iter().any(|(call_id, user_id)| {
            *call_id == response.call_id && *user_id == initiator_id
        }));

        let payload = recv_json(&mut receiver_rx).await;
        assert_eq!(payload.get("type").and_then(Value::as_str), Some("call-request"));
        assert_eq!(
            payload.get("call_id").and_then(Value::as_str),
            Some(response.call_id.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn initiate_call_rejects_non_member() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();

        let state = Arc::new(Mutex::new(MockCallState {
            conversation_members: HashMap::from([(conversation_id, vec![Uuid::now_v7()])]),
            ..Default::default()
        }));

        let service = build_service(
            MockCallRepo::new(state),
            MockParticipantRepo::default(),
            Arc::new(WebSocketServer::new()),
        );

        let result = service
            .initiate_call(
                initiator_id,
                InitiateCallRequest {
                    conversation_id,
                    call_type: CallType::Video,
                },
                "Alice".to_string(),
                None,
            )
            .await;

        assert!(matches!(result, Err(error::SystemError::Forbidden(_))));
    }

    #[tokio::test]
    async fn respond_call_accept_updates_status_and_emits_event() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();
        let responder_id = Uuid::now_v7();
        let call_id = Uuid::now_v7();

        let initial_call = CallEntity {
            id: call_id,
            conversation_id,
            initiator_id,
            call_type: CallType::Audio,
            status: CallStatus::Initiated,
            started_at: None,
            ended_at: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state = Arc::new(Mutex::new(MockCallState {
            calls: HashMap::from([(call_id, initial_call)]),
            conversation_members: HashMap::from([(conversation_id, vec![initiator_id, responder_id])]),
            ..Default::default()
        }));

        let participant_repo = MockParticipantRepo::default();
        let ws_server = Arc::new(WebSocketServer::new());
        let mut initiator_rx = connect_ws_user(&ws_server, initiator_id);

        let service = build_service(MockCallRepo::new(state.clone()), participant_repo.clone(), ws_server);

        service
            .respond_call(
                responder_id,
                call_id,
                RespondCallRequest {
                    accept: true,
                    reason: None,
                },
            )
            .await
            .expect("respond_call accept should succeed");

        let state_lock = state.lock().expect("call state mutex poisoned");
        assert!(state_lock
            .status_updates
            .iter()
            .any(|(id, status)| *id == call_id && *status == CallStatus::Accepted));
        drop(state_lock);

        let added = participant_repo
            .added
            .lock()
            .expect("participant mutex poisoned");
        assert!(added
            .iter()
            .any(|(saved_call_id, user_id)| *saved_call_id == call_id && *user_id == responder_id));

        let payload = recv_json(&mut initiator_rx).await;
        assert_eq!(payload.get("type").and_then(Value::as_str), Some("call-accept"));
        assert_eq!(
            payload.get("call_id").and_then(Value::as_str),
            Some(call_id.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn cancel_call_rejects_non_initiator() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();
        let other_user = Uuid::now_v7();
        let call_id = Uuid::now_v7();

        let initial_call = CallEntity {
            id: call_id,
            conversation_id,
            initiator_id,
            call_type: CallType::Audio,
            status: CallStatus::Initiated,
            started_at: None,
            ended_at: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state = Arc::new(Mutex::new(MockCallState {
            calls: HashMap::from([(call_id, initial_call)]),
            conversation_members: HashMap::from([(conversation_id, vec![initiator_id, other_user])]),
            ..Default::default()
        }));

        let service = build_service(
            MockCallRepo::new(state),
            MockParticipantRepo::default(),
            Arc::new(WebSocketServer::new()),
        );

        let result = service.cancel_call(other_user, call_id).await;

        assert!(matches!(result, Err(error::SystemError::Forbidden(_))));
    }

    #[tokio::test]
    async fn end_call_marks_participant_left_and_records_duration() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();
        let call_id = Uuid::now_v7();

        let initial_call = CallEntity {
            id: call_id,
            conversation_id,
            initiator_id,
            call_type: CallType::Video,
            status: CallStatus::Accepted,
            started_at: Some(Utc::now() - ChronoDuration::seconds(42)),
            ended_at: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state = Arc::new(Mutex::new(MockCallState {
            calls: HashMap::from([(call_id, initial_call)]),
            conversation_members: HashMap::from([(conversation_id, vec![initiator_id])]),
            ..Default::default()
        }));

        let participant_repo = MockParticipantRepo::default();
        let service = build_service(
            MockCallRepo::new(state.clone()),
            participant_repo.clone(),
            Arc::new(WebSocketServer::new()),
        );

        service
            .end_call(initiator_id, call_id)
            .await
            .expect("end_call should succeed");

        let state_lock = state.lock().expect("call state mutex poisoned");
        let duration = *state_lock
            .end_call_durations
            .last()
            .expect("duration should be recorded");
        assert!(duration >= 40);
        assert!(duration <= 90);
        assert!(state_lock.call_messages.iter().any(
            |(conversation, sender, message_type, content)| {
                *conversation == conversation_id
                    && *sender == initiator_id
                    && *message_type == MessageType::CallEnd
                    && content.as_deref().is_some_and(|v| v.contains("Cuộc gọi video đã kết thúc"))
            }
        ));
        drop(state_lock);

        let left = participant_repo.left.lock().expect("participant mutex poisoned");
        assert!(left
            .iter()
            .any(|(saved_call_id, user_id)| *saved_call_id == call_id && *user_id == initiator_id));
    }

    #[tokio::test]
    async fn get_call_history_clamps_limit_to_max_50() {
        let user_id = Uuid::now_v7();
        let state = Arc::new(Mutex::new(MockCallState::default()));

        let service = build_service(
            MockCallRepo::new(state.clone()),
            MockParticipantRepo::default(),
            Arc::new(WebSocketServer::new()),
        );

        let result = service
            .get_call_history(user_id, 200, None)
            .await
            .expect("get_call_history should succeed");

        assert!(result.calls.is_empty());

        let state_lock = state.lock().expect("call state mutex poisoned");
        assert_eq!(state_lock.last_history_limit, Some(50));
    }

    #[tokio::test]
    async fn respond_call_reject_persists_call_reject_message() {
        let conversation_id = Uuid::now_v7();
        let initiator_id = Uuid::now_v7();
        let responder_id = Uuid::now_v7();
        let call_id = Uuid::now_v7();

        let initial_call = CallEntity {
            id: call_id,
            conversation_id,
            initiator_id,
            call_type: CallType::Audio,
            status: CallStatus::Initiated,
            started_at: None,
            ended_at: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state = Arc::new(Mutex::new(MockCallState {
            calls: HashMap::from([(call_id, initial_call)]),
            conversation_members: HashMap::from([(conversation_id, vec![initiator_id, responder_id])]),
            ..Default::default()
        }));

        let service = build_service(
            MockCallRepo::new(state.clone()),
            MockParticipantRepo::default(),
            Arc::new(WebSocketServer::new()),
        );

        service
            .respond_call(
                responder_id,
                call_id,
                RespondCallRequest {
                    accept: false,
                    reason: Some("Bận".to_string()),
                },
            )
            .await
            .expect("respond_call reject should succeed");

        let state_lock = state.lock().expect("call state mutex poisoned");
        assert!(state_lock.call_messages.iter().any(
            |(conversation, sender, message_type, content)| {
                *conversation == conversation_id
                    && *sender == responder_id
                    && *message_type == MessageType::CallReject
                    && content.as_deref() == Some("Cuộc gọi thoại đã bị từ chối: Bận")
            }
        ));
    }
}
