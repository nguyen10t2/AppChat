# Phase 5 Prep - WebSocket Refactor

## Mục tiêu
- Tách rõ trách nhiệm trong WebSocket layer để giảm coupling giữa session/room/broadcast.
- Giữ nguyên behavior realtime hiện tại (events, payload shape, room semantics).
- Tăng khả năng test bằng cách cô lập logic state management theo từng concern.

## Entry Criteria
- Phase 4 đã hoàn thành (service policy extraction ở `message`, `conversation`, `call`, `friend`).
- Backend test suite đang xanh ổn định.
- Không có thay đổi API contract ở REST/WebSocket từ các phase trước.

## Phạm vi Phase 5
1) Session Management Boundary
- Tách logic connect/auth/disconnect thành unit helpers rõ ràng.
- Chuẩn hóa lifecycle thao tác với session map và user-session index.

2) Room Management Boundary
- Tách join/leave room logic thành helper/module nhỏ theo intent.
- Chuẩn hóa thao tác room membership và cleanup khi disconnect.

3) Broadcast Flow Cleanup
- Tách phần xây message delivery list khỏi send logic.
- Chuẩn hóa các biến thể `send_to_user`, `send_to_users`, `broadcast_to_room` theo policy nhất quán.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Session lifecycle helpers
- Tách helpers cho `connect`, `authenticate`, `disconnect`.
- Giữ nguyên side-effects hiện tại (presence/cleanup).
- Verify: `cargo test ws_server_test` + `cargo test`.

### Batch 2 - Room membership helpers
- Tách helpers cho join/leave/room cleanup.
- Giảm branch lồng và map mutation trực tiếp trong method chính.
- Verify: `cargo test ws_server_test` + `cargo test`.

### Batch 3 - Broadcast policy cleanup
- Tách helper chọn recipients và skip-user policy.
- Đồng nhất flow send đa user/room.
- Verify: `cargo test ws_server_test` + `cargo test`.

## Definition of Done
- Methods WebSocket chính giảm nested branches và rõ intent hơn.
- Có unit/integration coverage hiện hữu cho các behavior cốt lõi (auth, rooms, broadcast).
- Không thay đổi JSON event contract đang dùng ở frontend.
- Tài liệu cập nhật trong `docs/refactor.md` và `docs/changelog.md`.

## Rủi ro và giảm thiểu
- Rủi ro: đổi thứ tự side-effect gây lệch behavior realtime.
  - Giảm thiểu: refactor cơ học từng batch nhỏ + chạy `ws_server_test` sau mỗi batch.
- Rủi ro: race condition khi tách helper thao tác shared state.
  - Giảm thiểu: giữ nguyên chiến lược lock hiện tại, chỉ tách boundary logic.
- Rủi ro: phát sinh khác biệt payload khi tái tổ chức broadcast.
  - Giảm thiểu: không đổi format event; reuse builder/message structs hiện tại.
