#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use crate::api::error;
    use crate::configs::RedisCache;
    use crate::modules::conversation::model::{
        ConversationDetail, ConversationRow, NewLastMessage, NewParticipant,
        ParticipantDetailWithConversation,
    };
    use crate::modules::conversation::repository::{
        ConversationRepository, LastMessageRepository, ParticipantRepository,
    };
    use crate::modules::conversation::schema::{
        ConversationEntity, ConversationType, LastMessageEntity, ParticipantEntity,
    };
    use crate::modules::message::model::{InsertMessage, MessageQuery};
    use crate::modules::message::repository::MessageRepository;
    use crate::modules::message::schema::{MessageEntity, MessageType};
    use crate::modules::message::service::{MessageRoute, MessageService};
    use crate::modules::websocket::server::WebSocketServer;

    fn dummy_pool() -> sqlx::PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://postgres:postgres@localhost/postgres")
            .expect("failed to create lazy pool")
    }

    #[derive(Clone)]
    struct MockConversationRepo {
        pool: sqlx::PgPool,
        conversation_type: ConversationType,
        is_member: bool,
    }

    #[async_trait::async_trait]
    impl ConversationRepository for MockConversationRepo {
        fn get_pool(&self) -> &sqlx::Pool<sqlx::Postgres> {
            &self.pool
        }

        async fn find_by_id<'e, E>(
            &self,
            conversation_id: &Uuid,
            _tx: E,
        ) -> Result<Option<ConversationEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(Some(ConversationEntity {
                id: *conversation_id,
                _type: self.conversation_type.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }

        async fn find_one_conversation_detail(
            &self,
            _conversation_id: &Uuid,
        ) -> Result<Option<ConversationDetail>, error::SystemError> {
            Ok(None)
        }

        async fn create<'e, E>(
            &self,
            _type: &ConversationType,
            _tx: E,
        ) -> Result<ConversationEntity, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Err(error::SystemError::internal_error("not used"))
        }

        async fn create_direct_conversation<'e>(
            &self,
            _user_a: &Uuid,
            _user_b: &Uuid,
            _tx: &mut sqlx::Transaction<'e, sqlx::Postgres>,
        ) -> Result<ConversationEntity, error::SystemError> {
            Err(error::SystemError::internal_error("not used"))
        }

        async fn create_group_conversation<'e>(
            &self,
            _name: &str,
            _unique_member_ids: &[Uuid],
            _user_id: &Uuid,
            _tx: &mut sqlx::Transaction<'e, sqlx::Postgres>,
        ) -> Result<ConversationEntity, error::SystemError> {
            Err(error::SystemError::internal_error("not used"))
        }

        async fn find_direct_between_users<'e, E>(
            &self,
            _user_a: &Uuid,
            _user_b: &Uuid,
            _tx: E,
        ) -> Result<Option<ConversationEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(None)
        }

        async fn find_all_conversation_with_details_by_user<'e, E>(
            &self,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<Vec<ConversationRow>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(vec![])
        }

        async fn get_conversation_and_check_membership<'e, E>(
            &self,
            conversation_id: &Uuid,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<(Option<ConversationEntity>, bool), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok((
                Some(ConversationEntity {
                    id: *conversation_id,
                    _type: self.conversation_type.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }),
                self.is_member,
            ))
        }

        async fn update_timestamp<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockParticipantRepo {
        direct_unread_calls: Arc<Mutex<u32>>,
        group_unread_calls: Arc<Mutex<u32>>,
        participants: Vec<ParticipantDetailWithConversation>,
    }

    #[async_trait::async_trait]
    impl ParticipantRepository for MockParticipantRepo {
        async fn create_participant<'e, E>(
            &self,
            participant: &NewParticipant,
            _tx: E,
        ) -> Result<ParticipantEntity, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(ParticipantEntity {
                conversation_id: participant.conversation_id,
                user_id: participant.user_id,
                unread_count: participant.unread_count,
                joined_at: Utc::now(),
                deleted_at: None,
            })
        }

        async fn increment_unread_count<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            let mut lock = self
                .direct_unread_calls
                .lock()
                .expect("direct unread mutex poisoned");
            *lock += 1;
            Ok(())
        }

        async fn increment_unread_count_for_others<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _sender_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            let mut lock = self
                .group_unread_calls
                .lock()
                .expect("group unread mutex poisoned");
            *lock += 1;
            Ok(())
        }

        async fn reset_unread_count<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(())
        }

        async fn mark_as_seen<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _user_id: &Uuid,
            _last_seen_message_id: &Uuid,
            _tx: E,
        ) -> Result<(), error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(())
        }

        async fn find_participants_by_conversation_id<'e, E>(
            &self,
            _conversation_ids: &[Uuid],
            _tx: E,
        ) -> Result<Vec<ParticipantDetailWithConversation>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(self.participants.clone())
        }

        async fn get_unread_counts<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _tx: E,
        ) -> Result<HashMap<Uuid, i32>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(HashMap::new())
        }
    }

    #[derive(Clone)]
    struct MockLastMessageRepo;

    #[async_trait::async_trait]
    impl LastMessageRepository for MockLastMessageRepo {
        async fn upsert_last_message<'e, E>(
            &self,
            last_message: &NewLastMessage,
            _tx: E,
        ) -> Result<LastMessageEntity, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(LastMessageEntity {
                id: Uuid::now_v7(),
                content: last_message.content.clone(),
                conversation_id: last_message.conversation_id,
                created_at: last_message.created_at,
            })
        }
    }

    #[derive(Clone)]
    struct MockMessageRepo {
        pool: sqlx::PgPool,
    }

    #[async_trait::async_trait]
    impl MessageRepository for MockMessageRepo {
        fn get_pool(&self) -> &sqlx::PgPool {
            &self.pool
        }

        async fn find_by_id<'e, E>(
            &self,
            _message_id: &Uuid,
            _tx: E,
        ) -> Result<Option<MessageEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(None)
        }

        async fn create<'e, E>(
            &self,
            message: &InsertMessage,
            _tx: E,
        ) -> Result<MessageEntity, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(MessageEntity {
                id: Uuid::now_v7(),
                conversation_id: message.conversation_id,
                sender_id: message.sender_id,
                reply_to_id: message.reply_to_id,
                _type: message._type.clone(),
                content: message.content.clone(),
                file_url: message.file_url.clone(),
                is_edited: false,
                deleted_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn find_by_query<'e, E>(
            &self,
            _query: &MessageQuery,
            _limit: i32,
            _tx: E,
        ) -> Result<Vec<MessageEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(vec![])
        }

        async fn delete_message<'e, E>(
            &self,
            _message_id: &Uuid,
            _user_id: &Uuid,
            _tx: E,
        ) -> Result<bool, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(false)
        }

        async fn edit_message<'e, E>(
            &self,
            _message_id: &Uuid,
            _user_id: &Uuid,
            _new_content: &str,
            _tx: E,
        ) -> Result<Option<MessageEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(None)
        }

        async fn get_last_message_by_conversation<'e, E>(
            &self,
            _conversation_id: &Uuid,
            _tx: E,
        ) -> Result<Option<MessageEntity>, error::SystemError>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            Ok(None)
        }
    }

    async fn build_service(
        conversation_type: ConversationType,
        is_member: bool,
    ) -> (
        MessageService<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >,
        Arc<Mutex<u32>>,
        Arc<Mutex<u32>>,
        Uuid,
        Uuid,
    ) {
        let pool = dummy_pool();
        let direct_unread_calls = Arc::new(Mutex::new(0));
        let group_unread_calls = Arc::new(Mutex::new(0));

        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let conversation_id = Uuid::now_v7();

        let participant_repo = MockParticipantRepo {
            direct_unread_calls: direct_unread_calls.clone(),
            group_unread_calls: group_unread_calls.clone(),
            participants: vec![
                ParticipantDetailWithConversation {
                    user_id: sender_id,
                    display_name: "Sender".to_string(),
                    avatar_url: None,
                    unread_count: 0,
                    joined_at: Utc::now(),
                    conversation_id,
                },
                ParticipantDetailWithConversation {
                    user_id: recipient_id,
                    display_name: "Recipient".to_string(),
                    avatar_url: None,
                    unread_count: 0,
                    joined_at: Utc::now(),
                    conversation_id,
                },
            ],
        };

        let service = MessageService::with_dependencies(
            Arc::new(MockConversationRepo {
                pool: pool.clone(),
                conversation_type,
                is_member,
            }),
            Arc::new(MockMessageRepo { pool: pool.clone() }),
            Arc::new(participant_repo),
            Arc::new(MockLastMessageRepo),
            Arc::new(
                RedisCache::new()
                    .await
                    .expect("failed to initialize redis cache pool"),
            ),
            Arc::new(WebSocketServer::new()),
        );

        (
            service,
            direct_unread_calls,
            group_unread_calls,
            sender_id,
            conversation_id,
        )
    }

    #[tokio::test]
    async fn test_send_message_to_conversation_rejects_non_member() {
        let (service, _direct_calls, _group_calls, _sender_id, _conversation_id) =
            build_service(ConversationType::Group, false).await;

        let result = service
            .send_message_to_conversation(Uuid::now_v7(), Uuid::now_v7(), "hello".to_string())
            .await;

        assert!(matches!(result, Err(error::SystemError::Forbidden(_))));
    }

    #[test]
    fn test_resolve_message_route_group() {
        let sender_id = Uuid::now_v7();

        let route = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::resolve_message_route(&ConversationType::Group, sender_id, [sender_id]);

        assert!(matches!(route, Ok(MessageRoute::Group)));
    }

    #[test]
    fn test_resolve_message_route_direct_success() {
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();

        let route = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::resolve_message_route(&ConversationType::Direct, sender_id, [sender_id, recipient_id]);

        assert!(matches!(
            route,
            Ok(MessageRoute::Direct {
                recipient_id: id
            }) if id == recipient_id
        ));
    }

    #[test]
    fn test_resolve_message_route_direct_missing_recipient() {
        let sender_id = Uuid::now_v7();

        let route = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::resolve_message_route(&ConversationType::Direct, sender_id, [sender_id]);

        assert!(matches!(route, Err(error::SystemError::BadRequest(_))));
    }

    #[test]
    fn test_normalize_message_input_defaults_to_text() {
        let result = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::normalize_message_input(Some(" hello ".to_string()), None, None)
        .expect("expected valid text payload");

        assert!(matches!(result.0, MessageType::Text));
        assert_eq!(result.1, Some("hello".to_string()));
        assert_eq!(result.2, None);
    }

    #[test]
    fn test_normalize_message_input_requires_file_url_for_file_type() {
        let result = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::normalize_message_input(None, Some(MessageType::File), None);

        assert!(matches!(result, Err(error::SystemError::BadRequest(_))));
    }

    #[test]
    fn test_normalize_message_input_rejects_empty_payload() {
        let result = MessageService::<
            MockMessageRepo,
            MockConversationRepo,
            MockParticipantRepo,
            MockLastMessageRepo,
        >::normalize_message_input(Some("   ".to_string()), None, Some(" ".to_string()));

        assert!(matches!(result, Err(error::SystemError::BadRequest(_))));
    }
}
