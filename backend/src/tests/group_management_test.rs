#[cfg(test)]
mod tests {
    use crate::configs::AppConfig;
    use crate::configs::connect_database;
    use crate::middlewares::{authentication, authorization};
    use crate::modules::conversation::handle::{self, ConversationSvc};
    use crate::modules::conversation::model::{AddMemberRequest, UpdateGroupRequest};
    use crate::modules::conversation::repository_pg::{
        ConversationPgRepository, ParticipantPgRepository,
    };
    use crate::modules::conversation::service::ConversationService;
    use crate::modules::friend::handle::FriendSvc;
    use crate::modules::friend::repository_pg::FriendRepositoryPg;
    use crate::modules::friend::service::FriendService;
    use crate::modules::message::repository_pg::MessageRepositoryPg;
    use crate::modules::user::repository_pg::UserRepositoryPg;
    use crate::modules::user::schema::UserRole;
    use crate::modules::websocket::server::WebSocketServer;
    use crate::utils::{Claims, TypeClaims};
    use actix_web::{App, http::StatusCode, middleware::from_fn, test, web};
    use std::sync::Arc;
    use uuid::Uuid;

    async fn seed_user(pool: &sqlx::PgPool, id: Uuid, username: &str) {
        sqlx::query("INSERT INTO users (id, username, hash_password, email, role, display_name) VALUES ($1, $2, 'hash', $3, 'USER', $2)")
            .bind(id)
            .bind(username)
            .bind(format!("{}@test.local", username))
            .execute(pool)
            .await
            .unwrap();
    }

    fn build_token(user_id: Uuid) -> String {
        let cfg = AppConfig::from_env_lossy();
        Claims::new(&user_id, &UserRole::User, 3600)
            .with_type(TypeClaims::AccessToken)
            .encode(cfg.jwt_secret.as_ref())
            .unwrap()
    }

    fn build_services(pool: sqlx::PgPool) -> (ConversationSvc, FriendSvc) {
        let participant_repo = ParticipantPgRepository::default();
        let conversation_repo =
            ConversationPgRepository::new(pool.clone(), participant_repo.clone());
        let message_repo = MessageRepositoryPg::new(pool.clone());
        let ws_server = Arc::new(WebSocketServer::new());

        let conversation_svc = ConversationService::with_dependencies(
            Arc::new(conversation_repo),
            Arc::new(participant_repo),
            Arc::new(message_repo),
            ws_server,
        );

        let friend_repo = Arc::new(FriendRepositoryPg::new(pool.clone()));
        let user_repo = Arc::new(UserRepositoryPg::new(pool));
        let friend_svc = FriendService::with_dependencies(friend_repo, user_repo);

        (conversation_svc, friend_svc)
    }

    #[tokio::test]
    #[ignore = "requires postgres"]
    async fn test_group_management_flow() {
        let pool = connect_database().await.unwrap();
        let creator_id = Uuid::now_v7();
        let member_id = Uuid::now_v7();
        let outsider_id = Uuid::now_v7();
        let group_id = Uuid::now_v7();

        seed_user(&pool, creator_id, "creator").await;
        seed_user(&pool, member_id, "member").await;
        seed_user(&pool, outsider_id, "outsider").await;

        // Seed group
        sqlx::query("INSERT INTO conversations (id, type) VALUES ($1, 'group')")
            .bind(group_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO group_conversations (conversation_id, name, created_by) VALUES ($1, 'Old Name', $2)").bind(group_id).bind(creator_id).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO participants (conversation_id, user_id, unread_count) VALUES ($1, $2, 0)",
        )
        .bind(group_id)
        .bind(creator_id)
        .execute(&pool)
        .await
        .unwrap();

        let (conv_svc, friend_svc) = build_services(pool.clone());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(conv_svc))
                .app_data(web::Data::new(friend_svc))
                .service(
                    web::scope("/api/conversations")
                        .wrap(from_fn(authorization(vec![UserRole::User])))
                        .wrap(from_fn(authentication))
                        .service(handle::update_group)
                        .service(handle::add_member)
                        .service(handle::remove_member),
                ),
        )
        .await;

        let token = build_token(creator_id);

        // 1. Test Rename Group
        let req = test::TestRequest::patch()
            .uri(&format!("/api/conversations/{}/group", group_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(UpdateGroupRequest {
                name: Some("New Name".to_string()),
                avatar_url: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify in DB
        let name: String =
            sqlx::query_scalar("SELECT name FROM group_conversations WHERE conversation_id = $1")
                .bind(group_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(name, "New Name");

        // 2. Test Add Member (Non-friend should fail if enforced, but here we check creator permission first)
        // Note: For simplicity in this test, we don't seed friendship but check the add_member logic
        let req = test::TestRequest::post()
            .uri(&format!("/api/conversations/{}/members", group_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(AddMemberRequest { user_id: member_id })
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Should fail because they are not friends
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // Seed friendship
        sqlx::query("INSERT INTO friends (user_id1, user_id2, status) VALUES ($1, $2, 'ACCEPTED')")
            .bind(creator_id)
            .bind(member_id)
            .execute(&pool)
            .await
            .unwrap();

        let req = test::TestRequest::post()
            .uri(&format!("/api/conversations/{}/members", group_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(AddMemberRequest { user_id: member_id })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 3. Test Remove Member (Kick)
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/api/conversations/{}/members/{}",
                group_id, member_id
            ))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify soft delete
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM participants WHERE conversation_id = $1 AND user_id = $2 AND deleted_at IS NOT NULL").bind(group_id).bind(member_id).fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1);

        // Cleanup
        let _ = sqlx::query("DELETE FROM group_conversations WHERE conversation_id = $1")
            .bind(group_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM participants WHERE conversation_id = $1")
            .bind(group_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(group_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM friends WHERE user_id1 = $1")
            .bind(creator_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id IN ($1, $2, $3)")
            .bind(creator_id)
            .bind(member_id)
            .bind(outsider_id)
            .execute(&pool)
            .await;
    }
}
