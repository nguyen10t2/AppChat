#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{
        App,
        http::StatusCode,
        middleware::from_fn,
        test,
        web,
    };
    use uuid::Uuid;

    use crate::configs::connect_database;
    use crate::middlewares::{authentication, authorization};
    use crate::modules::conversation::handle::ConversationSvc;
    use crate::modules::conversation::repository_pg::{
        ConversationPgRepository, ParticipantPgRepository,
    };
    use crate::modules::conversation::route as conversation_route;
    use crate::modules::conversation::service::ConversationService;
    use crate::modules::message::repository_pg::MessageRepositoryPg;
    use crate::modules::user::schema::UserRole;
    use crate::modules::websocket::server::WebSocketServer;
    use crate::utils::{Claims, TypeClaims};
    use crate::ENV;

    async fn seed_user(
        pool: &sqlx::PgPool,
        id: Uuid,
        username: &str,
        email: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, hash_password, email, role, display_name)
            VALUES ($1, $2, $3, $4, 'USER', $5)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind("test_hash")
        .bind(email)
        .bind(username)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn cleanup_users(pool: &sqlx::PgPool, user_ids: &[Uuid]) {
        let _ = sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(user_ids)
            .execute(pool)
            .await;
    }

    fn build_claims(user_id: Uuid) -> Claims {
        Claims::new(&user_id, &UserRole::User, 3600).with_type(TypeClaims::AccessToken)
    }

    fn build_access_token(user_id: Uuid) -> String {
        build_claims(user_id)
            .encode(ENV.jwt_secret.as_ref())
            .expect("should encode access token")
    }

    fn build_conversation_service(pool: sqlx::PgPool) -> ConversationSvc {
        let participant_repo = ParticipantPgRepository::default();
        let conversation_repo = ConversationPgRepository::new(pool.clone(), participant_repo.clone());
        let message_repo = MessageRepositoryPg::new(pool);

        ConversationService::with_dependencies(
            Arc::new(conversation_repo),
            Arc::new(participant_repo),
            Arc::new(message_repo),
            Arc::new(WebSocketServer::new()),
        )
    }

    #[tokio::test]
    #[ignore = "requires postgres running with migrated schema"]
    async fn test_get_messages_forbidden_when_user_not_member() {
        let pool = connect_database()
            .await
            .expect("database must be available for integration test");

        let owner_id = Uuid::now_v7();
        let outsider_id = Uuid::now_v7();
        let conversation_id = Uuid::now_v7();

        seed_user(&pool, owner_id, "owner_conv_test", "owner_conv_test@appchat.local")
            .await
            .expect("seed owner user should succeed");
        seed_user(
            &pool,
            outsider_id,
            "outsider_conv_test",
            "outsider_conv_test@appchat.local",
        )
        .await
        .expect("seed outsider user should succeed");

        sqlx::query("INSERT INTO conversations (id, type) VALUES ($1, 'group')")
            .bind(conversation_id)
            .execute(&pool)
            .await
            .expect("seed conversation should succeed");

        sqlx::query("INSERT INTO participants (conversation_id, user_id, unread_count) VALUES ($1, $2, 0)")
            .bind(conversation_id)
            .bind(owner_id)
            .execute(&pool)
            .await
            .expect("seed participant should succeed");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_conversation_service(pool.clone())))
                .service(
                    web::scope("/api").service(
                        web::scope("")
                            .wrap(from_fn(authorization(vec![UserRole::User])))
                            .wrap(from_fn(authentication))
                            .configure(conversation_route::configure),
                    ),
                ),
        )
        .await;

        let token = build_access_token(outsider_id);
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/conversations/{conversation_id}/messages?limit=10"
            ))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        cleanup_users(&pool, &[owner_id, outsider_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires postgres running with migrated schema"]
    async fn test_get_messages_success_when_user_is_member() {
        let pool = connect_database()
            .await
            .expect("database must be available for integration test");

        let owner_id = Uuid::now_v7();
        let conversation_id = Uuid::now_v7();

        seed_user(&pool, owner_id, "member_conv_test", "member_conv_test@appchat.local")
            .await
            .expect("seed member user should succeed");

        sqlx::query("INSERT INTO conversations (id, type) VALUES ($1, 'group')")
            .bind(conversation_id)
            .execute(&pool)
            .await
            .expect("seed conversation should succeed");

        sqlx::query("INSERT INTO participants (conversation_id, user_id, unread_count) VALUES ($1, $2, 0)")
            .bind(conversation_id)
            .bind(owner_id)
            .execute(&pool)
            .await
            .expect("seed participant should succeed");

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, sender_id, type, content) VALUES ($1, $2, $3, 'text', $4)",
        )
        .bind(Uuid::now_v7())
        .bind(conversation_id)
        .bind(owner_id)
        .bind("hello integration")
        .execute(&pool)
        .await
        .expect("seed message should succeed");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_conversation_service(pool.clone())))
                .service(
                    web::scope("/api").service(
                        web::scope("")
                            .wrap(from_fn(authorization(vec![UserRole::User])))
                            .wrap(from_fn(authentication))
                            .configure(conversation_route::configure),
                    ),
                ),
        )
        .await;

        let token = build_access_token(owner_id);
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/conversations/{conversation_id}/messages?limit=10"
            ))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(response).await;
        let message_count = body
            .get("data")
            .and_then(|v| v.get("messages"))
            .and_then(|v| v.as_array())
            .map_or(0, std::vec::Vec::len);
        assert!(message_count >= 1);

        cleanup_users(&pool, &[owner_id]).await;
    }
}
