# Backend Module Notes (Phase 7)

Ghi chú ngắn cho các module đã refactor nhiều trong Phase 1-6, phục vụ onboarding và bảo trì.

## 1) Message Module
- **Vai trò chính**: xử lý send/edit/delete message và realtime event mới.
- **Service boundary**:
  - orchestration: transaction + persist + broadcast
  - policy helpers: route resolution, membership/ownership checks, input normalization
- **Điểm cần giữ ổn định**:
  - payload/event contract khi gửi qua WebSocket
  - validation `content/file_url/message_type`

## 2) Conversation Module
- **Vai trò chính**: quản lý conversation list/messages/mark-as-seen và group management.
- **Service boundary**:
  - orchestration: load participants, unread counts, room broadcasts
  - policy helpers: group owner/member/type/removal permissions
- **Điểm cần giữ ổn định**:
  - quyền thao tác group (owner/member)
  - behavior `mark_as_seen` và unread count updates

## 3) Call Module
- **Vai trò chính**: initiate/respond/cancel/end call + call history.
- **Service boundary**:
  - orchestration: status transitions + participant lifecycle + call messages
  - policy helpers: membership, initiator-only actions, status gating
- **Điểm cần giữ ổn định**:
  - status transitions (`Initiated -> Accepted/Rejected -> Ended`)
  - message side-effects (`CallReject`, `CallCancel`, `CallEnd`)

## 4) WebSocket Module
- **Vai trò chính**: quản lý session/user/room state và fan-out realtime events.
- **Server boundary**:
  - session lifecycle helpers (connect/auth/disconnect)
  - room membership helpers (join/leave/cleanup)
  - broadcast helpers (user list + optional skip policy)
- **Điểm cần giữ ổn định**:
  - mapping user <-> sessions và cleanup chính xác khi disconnect
  - room delivery semantics và skip-user behavior

## Test Focus Matrix
- `message_test`: message policy + route/normalization regressions
- `conversation_test`: conversation message access control (integration, requires Postgres)
- `call_test`: call status/permission guard branches
- `ws_server_test`: session lifecycle, room membership, broadcast semantics

## Refactor Safety Rules
- Không đổi API contract response (`code`, `message`, data envelope) nếu không có migration plan.
- Không đổi WS event shape nếu chưa phối hợp frontend.
- Khi đổi orchestration/service internals:
  - chạy test module liên quan trước,
  - sau đó chạy full `cargo test` để detect regressions chéo module.
