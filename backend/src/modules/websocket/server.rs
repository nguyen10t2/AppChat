use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::message::ServerMessage;
use crate::METRICS;

const RECONNECT_WINDOW: Duration = Duration::from_secs(120);

/// WebSocket server quản lý tất cả client sessions và conversation rooms
/// Hoạt động như một shared state với DashMap cho truy cập an toàn từ nhiều luồng
#[derive(Default)]
pub struct WebSocketServer {
    /// Map: session_id -> channel sender (gửi message qua mpsc)
    sessions: DashMap<Uuid, mpsc::UnboundedSender<String>>,

    /// Map: user_id -> set of session_ids
    users: DashMap<Uuid, DashSet<Uuid>>,

    /// Map: conversation_id -> set of user_ids
    rooms: DashMap<Uuid, DashSet<Uuid>>,

    /// Map: user_id -> set of conversation_ids
    user_rooms: DashMap<Uuid, DashSet<Uuid>>,

    /// Map: user_id -> last fully-disconnected instant
    last_disconnect_at: DashMap<Uuid, Instant>,
}

impl WebSocketServer {
    /// Tạo WebSocket server mới với state rỗng
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            users: DashMap::new(),
            rooms: DashMap::new(),
            user_rooms: DashMap::new(),
            last_disconnect_at: DashMap::new(),
        }
    }

    /// Lấy danh sách user IDs đang online
    pub fn get_online_users(&self) -> Vec<Uuid> {
        let mut users = Vec::with_capacity(self.users.len());
        for entry in self.users.iter() {
            users.push(*entry.key());
        }
        users
    }

    /// Gửi message tới một session cụ thể
    pub fn send_to_session(&self, session_id: &Uuid, message: &ServerMessage) {
        if let Some(tx) = self.sessions.get(session_id) {
            if let Ok(json) = serde_json::to_string(message) {
                let _ = tx.send(json);
            } else {
                tracing::error!(
                    "Không thể serialize ServerMessage cho session {}",
                    session_id
                );
            }
        }
    }

    /// Gửi message tới tất cả sessions của một user (multi-device)
    pub fn send_to_user(&self, user_id: &Uuid, message: &ServerMessage) {
        if let Some(sessions) = self.users.get(user_id) {
            // Serialize once
            if let Ok(json) = serde_json::to_string(message) {
                for session_id in sessions.iter() {
                    if let Some(tx) = self.sessions.get(&*session_id) {
                        let _ = tx.send(json.clone());
                    }
                }
            }
        }
    }

    /// Gửi message tới nhiều users
    pub fn send_to_users(&self, user_ids: &[Uuid], message: &ServerMessage) {
        if let Ok(json) = serde_json::to_string(message) {
            user_ids.par_iter().for_each(|user_id| {
                if let Some(sessions) = self.users.get(user_id) {
                    for session_id in sessions.iter() {
                        if let Some(tx) = self.sessions.get(&*session_id) {
                            let _ = tx.send(json.clone());
                        }
                    }
                }
            });
        }
    }

    /// Xử lý client connect mới
    pub fn connect(&self, session_id: Uuid, tx: mpsc::UnboundedSender<String>) {
        tracing::debug!("New WebSocket session connected: {}", session_id);
        self.sessions.insert(session_id, tx);
    }

    /// Xử lý client disconnect. Trả về Some(user_id) nếu user không còn kết nối nào.
    pub fn disconnect(&self, session_id: Uuid) -> Option<Uuid> {
        tracing::debug!("WebSocket session disconnected: {}", session_id);
        self.sessions.remove(&session_id);

        let mut user_fully_disconnected = None;

        for user_entry in self.users.iter() {
            let user_id = *user_entry.key();
            let sessions = user_entry.value();

            if sessions.remove(&session_id).is_some() {
                tracing::debug!("Removed session {} from user {}", session_id, user_id);
                if sessions.is_empty() {
                    user_fully_disconnected = Some(user_id);
                }
                break;
            }
        }

        if let Some(user_id) = user_fully_disconnected {
            self.users.remove(&user_id);

            if let Some((_, user_room_ids)) = self.user_rooms.remove(&user_id) {
                let mut empty_rooms: Vec<Uuid> = Vec::new();
                for room_id in user_room_ids.iter() {
                    if let Some(room) = self.rooms.get(&*room_id) {
                        room.remove(&user_id);
                        if room.is_empty() {
                            empty_rooms.push(*room_id);
                        }
                    }
                }
                for room_id in empty_rooms {
                    self.rooms.remove(&room_id);
                }
            }

            tracing::info!(
                "User {} fully disconnected (no more sessions) and removed from all rooms",
                user_id
            );

            self.last_disconnect_at.insert(user_id, Instant::now());
            return Some(user_id);
        }

        None
    }

    /// Xác thực user cho 1 session
    pub fn authenticate(&self, session_id: Uuid, user_id: Uuid) {
        tracing::info!("User {} authenticated on session {}", user_id, session_id);

        if let Some((_, disconnected_at)) = self.last_disconnect_at.remove(&user_id)
            && disconnected_at.elapsed() <= RECONNECT_WINDOW {
                METRICS.inc_ws_reconnect();
        }

        let sessions = self.users.entry(user_id).or_default();
        sessions.insert(session_id);
    }

    /// Join conversation room
    pub fn join_room(&self, user_id: Uuid, conversation_id: Uuid) {
        self.rooms
            .entry(conversation_id)
            .or_default()
            .insert(user_id);
        self.user_rooms
            .entry(user_id)
            .or_default()
            .insert(conversation_id);
        tracing::debug!("User {} joined conversation {}", user_id, conversation_id);
    }

    /// Leave conversation room
    pub fn leave_room(&self, user_id: Uuid, conversation_id: Uuid) {
        if let Some(room) = self.rooms.get(&conversation_id) {
            room.remove(&user_id);
            if room.is_empty() {
                // Drop the reference early to avoid deadlock
                drop(room);
                self.rooms.remove(&conversation_id);
            }
        }

        if let Some(user_room) = self.user_rooms.get(&user_id) {
            user_room.remove(&conversation_id);
        }
    }

    /// Broadcast message tới room, tuỳ chọn skip user
    pub fn broadcast_to_room(
        &self,
        conversation_id: Uuid,
        message: &ServerMessage,
        skip_user_id: Option<Uuid>,
    ) {
        if let Some(room_users) = self.rooms.get(&conversation_id)
            && let Ok(json) = serde_json::to_string(message) {
                let user_ids: Vec<Uuid> = room_users.iter().map(|k| *k).collect();
                user_ids.par_iter().for_each(|user_id| {
                    if Some(*user_id) == skip_user_id {
                        return;
                    }

                    if let Some(sessions) = self.users.get(user_id) {
                        for session_id in sessions.iter() {
                            if let Some(tx) = self.sessions.get(&*session_id) {
                                let _ = tx.send(json.clone());
                            }
                        }
                    }
                });
            }
    }

    /// Broadcast message tới tất cả sessions
    pub fn broadcast_to_all(&self, message: &ServerMessage) {
        if let Ok(json) = serde_json::to_string(message) {
            let endpoints: Vec<mpsc::UnboundedSender<String>> =
                self.sessions.iter().map(|s| s.value().clone()).collect();
            endpoints.par_iter().for_each(|tx| {
                let _ = tx.send(json.clone());
            });
        }
    }

    /// Thông báo bạn bè về sự thay đổi trạng thái
    pub fn user_presence_changed(
        &self,
        user_id: Uuid,
        is_online: bool,
        friend_ids: &[Uuid],
        last_seen: Option<String>,
    ) {
        let event = if is_online {
            ServerMessage::UserOnline { user_id }
        } else {
            ServerMessage::UserOffline { user_id, last_seen }
        };

        if let Ok(json) = serde_json::to_string(&event) {
            friend_ids.par_iter().for_each(|friend_id| {
                if let Some(sessions) = self.users.get(friend_id) {
                    for session_id in sessions.iter() {
                        if let Some(tx) = self.sessions.get(&*session_id) {
                            let _ = tx.send(json.clone());
                        }
                    }
                }
            });
        }
    }

    /// Gửi thông tin trạng thái ban đầu của bạn bè khi user đăng nhập
    pub fn send_initial_presence(&self, user_id: &Uuid, friend_ids: &[Uuid]) {
        let online_friend_ids: Vec<Uuid> = friend_ids
            .iter()
            .filter(|fid| self.users.contains_key(*fid))
            .copied()
            .collect();

        if !online_friend_ids.is_empty() {
            let message = ServerMessage::OnlineUsers {
                user_ids: online_friend_ids,
            };
            self.send_to_user(user_id, &message);
        }
    }
}
