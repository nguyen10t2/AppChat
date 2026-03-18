/// WebSocket HTTP Handler (Actorless)
///
/// Xử lý HTTP upgrade thành WebSocket và chia ra 2 tasks đọc/ghi
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse, web};
use actix_ws::Message;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::message::ClientMessage;
use super::presence::PresenceService;
use super::server::WebSocketServer;
use super::session::{MessageSvc, WebSocketSessionImpl};
use crate::modules::friend::repository_pg::FriendRepositoryPg;
use crate::observability::{RequestContext, WsCloseReason};
use crate::METRICS;
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP handler để upgrade connection thành WebSocket (không dùng Actor)
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<WebSocketServer>>,
    message_service: web::Data<MessageSvc>,
    presence_service: web::Data<PresenceService>,
    friend_repo: web::Data<FriendRepositoryPg>,
) -> Result<HttpResponse, Error> {
    tracing::debug!("WebSocket upgrade request từ {:?}", req.peer_addr());

    let correlation_id = req
        .extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.clone())
        .or_else(|| {
            req.headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let mut ws_session = WebSocketSessionImpl::new(
        server.get_ref().clone(),
        tx.clone(),
        correlation_id.clone(),
        Some(Arc::new(message_service.into_inner().as_ref().clone())),
        Some(Arc::new(presence_service.into_inner().as_ref().clone())),
        Some(Arc::new(friend_repo.into_inner().as_ref().clone())),
    );

    let session_id = ws_session.id;

    // Đăng ký session với Server
    server.connect(session_id, tx);

    // Spawn task gửi (Writer Task)
    let mut session_writer = session.clone();
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session_writer.text(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn task nhận và xử lý (Reader + State Task)
    actix_web::rt::spawn(async move {
        let mut heartbeat = actix_web::rt::time::interval(HEARTBEAT_INTERVAL);
        let close_reason = loop {
            tokio::select! {
                // Heartbeat / ping mechanism
                _ = heartbeat.tick() => {
                    // Cập nhật TTL presence trên Redis
                    if let (Some(user_id), Some(presence)) = (ws_session.user_id, ws_session.presence_service.clone()) {
                        actix_web::rt::spawn(async move {
                            if let Err(e) = presence.refresh_presence(user_id).await {
                                tracing::warn!("Lỗi refresh Redis presence cho user {}: {}", user_id, e);
                            }
                        });
                    }

                    // Ping cho Client
                    if session.ping(b"").await.is_err() {
                        tracing::warn!("Ping failed. Disconnecting {}", session_id);
                        break WsCloseReason::PingFailure;
                    }
                }

                // Nhận thông điệp mới từ Client -> timeout nếu lâu không có tin nhắn (bao gồm pong)
                result = timeout(CLIENT_TIMEOUT, msg_stream.recv()) => {
                    let msg_opt = match result {
                        Ok(m) => m,
                        Err(_) => {
                            tracing::warn!("Client WebSocket timeout: {}", session_id);
                            break WsCloseReason::Timeout;
                        }
                    };

                    match msg_opt {
                        Some(Ok(Message::Text(text))) => {
                            let text_str = text.to_string();
                            match serde_json::from_str::<ClientMessage>(&text_str) {
                                Ok(client_msg) => {
                                    ws_session.handle_client_message(client_msg).await;
                                }
                                Err(e) => {
                                    tracing::warn!("Parse ClientMessage lỗi: {} (raw: {})", e, &text_str[..100.min(text_str.len())]);
                                }
                            }
                        }
                        Some(Ok(Message::Ping(bytes))) => {
                            let _ = session.pong(&bytes).await;
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Cập nhật trạng thái connection OK - do loop chờ tiếp
                        }
                        Some(Ok(Message::Close(reason))) => {
                            tracing::info!("Client gửi Close frame: {:?}", reason);
                            break WsCloseReason::ClientClose;
                        }
                        Some(Ok(Message::Binary(_))) | Some(Ok(Message::Continuation(_))) | Some(Ok(Message::Nop)) => {
                            // Bỏ qua
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket protocol error: {}", e);
                            break WsCloseReason::ProtocolError;
                        }
                        None => {
                            // Mất kết nối client tự nhiên
                            break WsCloseReason::StreamEnded;
                        }
                    }
                }
            }
        };

        METRICS.record_ws_close_reason(close_reason);
        METRICS.inc_ws_disconnect();

        // Cleanup: Xóa trạng thái của session và đóng kết nối
        let _ = session.close(None).await;

        let fully_disconnected_user = server.disconnect(session_id);

        if let (Some(user_id), Some(presence_service)) =
            (fully_disconnected_user, ws_session.presence_service.clone())
        {
            let friend_ids = ws_session.friend_ids.clone();
            let server_ref = ws_session.server.clone();

            actix_web::rt::spawn(async move {
                // Set offline trong db redis
                if let Err(e) = presence_service.set_offline(user_id).await {
                    tracing::error!("Lỗi set Redis offline cho user {}: {}", user_id, e);
                }

                if !friend_ids.is_empty() {
                    let last_seen = Some(chrono::Utc::now().to_rfc3339());
                    server_ref.user_presence_changed(user_id, false, &friend_ids, last_seen);
                }
            });
        }

        tracing::debug!("WebSocket message loop kết thúc: {}", session_id);
    });

    tracing::info!(correlation_id = %correlation_id, "WebSocket connection established");
    Ok(response)
}
