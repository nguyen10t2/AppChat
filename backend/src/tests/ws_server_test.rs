use crate::modules::websocket::{message::ServerMessage, server::WebSocketServer};
use tokio::sync::mpsc;
use uuid::Uuid;

// ... Để viết DB integration test cho WS thì cần Mock hoặc in-memory DB.
// Vì vậy ta viết unit test cho WebSocketServer State trước để kiểm tra logic in-memory hoạt động an toàn.

#[tokio::test]
async fn test_server_connect_disconnect() {
    let server = WebSocketServer::new();
    let session_id = Uuid::now_v7();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Test connect
    server.connect(session_id, tx);
    assert_eq!(server.get_online_users().len(), 0); // Chưa authenticate

    // Test send message
    server.broadcast_to_all(&ServerMessage::Pong);
    let msg: String = rx.recv().await.unwrap();
    assert!(msg.contains("pong"));

    // Test disconnect
    let fully_disconnected = server.disconnect(session_id);
    assert!(fully_disconnected.is_none());
}

#[tokio::test]
async fn test_server_authentication_and_presence() {
    let server = WebSocketServer::new();

    let session_1 = Uuid::now_v7();
    let session_2 = Uuid::now_v7();
    let user_id = Uuid::now_v7();

    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();

    server.connect(session_1, tx1.clone());
    server.connect(session_2, tx2.clone());

    server.authenticate(session_1, user_id);
    assert_eq!(server.get_online_users().len(), 1);

    server.authenticate(session_2, user_id);
    assert_eq!(server.get_online_users().len(), 1); // Cùng 1 user

    // Gửi message tới user - cả 2 session phải nhận được
    server.send_to_user(&user_id, &ServerMessage::Pong);

    let msg1: String = rx1.recv().await.unwrap();
    assert!(msg1.contains("pong"));

    // Disconnect session 1, user vẫn online trên session 2
    let fully_disconnected = server.disconnect(session_1);
    assert!(fully_disconnected.is_none());
    assert_eq!(server.get_online_users().len(), 1);

    // Disconnect nốt session 2, user offline
    let fully_disconnected_2 = server.disconnect(session_2);
    assert_eq!(fully_disconnected_2, Some(user_id));
    assert_eq!(server.get_online_users().len(), 0);
}

#[tokio::test]
async fn test_server_rooms() {
    let server = WebSocketServer::new();

    let user_1 = Uuid::now_v7();
    let user_2 = Uuid::now_v7();
    let room_id = Uuid::now_v7();

    let session_1 = Uuid::now_v7();
    let session_2 = Uuid::now_v7();

    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    server.connect(session_1, tx1);
    server.connect(session_2, tx2);

    server.authenticate(session_1, user_1);
    server.authenticate(session_2, user_2);

    server.join_room(user_1, room_id);
    server.join_room(user_2, room_id);

    // Broadcast
    server.broadcast_to_room(room_id, &ServerMessage::Pong, None);

    let msg1: String = rx1.recv().await.unwrap();
    let msg2: String = rx2.recv().await.unwrap();

    assert!(msg1.contains("pong"));
    assert!(msg2.contains("pong"));

    // Leave room
    server.leave_room(user_1, room_id);

    server.broadcast_to_room(room_id, &ServerMessage::Pong, None);
    let msg2_2: String = rx2.recv().await.unwrap();
    assert!(msg2_2.contains("pong"));

    // rx1 không nhận được vì đã leave
    let timeout: Result<Option<String>, tokio::time::error::Elapsed> =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv()).await;
    assert!(timeout.is_err());
}

#[tokio::test]
async fn test_broadcast_to_room_skip_user() {
    let server = WebSocketServer::new();

    let user_1 = Uuid::now_v7();
    let user_2 = Uuid::now_v7();
    let user_3 = Uuid::now_v7();
    let room_id = Uuid::now_v7();

    let session_1 = Uuid::now_v7();
    let session_2 = Uuid::now_v7();
    let session_3 = Uuid::now_v7();

    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();
    let (tx3, mut rx3) = mpsc::unbounded_channel();

    server.connect(session_1, tx1);
    server.connect(session_2, tx2);
    server.connect(session_3, tx3);

    server.authenticate(session_1, user_1);
    server.authenticate(session_2, user_2);
    server.authenticate(session_3, user_3);

    server.join_room(user_1, room_id);
    server.join_room(user_2, room_id);
    server.join_room(user_3, room_id);

    server.broadcast_to_room(room_id, &ServerMessage::Pong, Some(user_2));

    let msg1: String = rx1.recv().await.unwrap();
    let msg3: String = rx3.recv().await.unwrap();
    assert!(msg1.contains("pong"));
    assert!(msg3.contains("pong"));

    let skipped: Result<Option<String>, tokio::time::error::Elapsed> =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx2.recv()).await;
    assert!(skipped.is_err());
}

#[tokio::test]
async fn test_send_to_user_all_sessions_receive_once() {
    let server = WebSocketServer::new();

    let user_id = Uuid::now_v7();
    let session_1 = Uuid::now_v7();
    let session_2 = Uuid::now_v7();

    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    server.connect(session_1, tx1);
    server.connect(session_2, tx2);
    server.authenticate(session_1, user_id);
    server.authenticate(session_2, user_id);

    server.send_to_user(&user_id, &ServerMessage::Pong);

    let msg1: String = rx1.recv().await.unwrap();
    let msg2: String = rx2.recv().await.unwrap();
    assert!(msg1.contains("pong"));
    assert!(msg2.contains("pong"));

    let extra1: Result<Option<String>, tokio::time::error::Elapsed> =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv()).await;
    let extra2: Result<Option<String>, tokio::time::error::Elapsed> =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx2.recv()).await;
    assert!(extra1.is_err());
    assert!(extra2.is_err());
}
