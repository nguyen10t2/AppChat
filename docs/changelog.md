# Changelog

## 2026-03-23

### Backend Refactor (Phase 1 + Phase 2 core)
- Hoàn tất refactor cấu hình theo hướng DI với `AppConfig` + `AppState`.
- Loại bỏ phụ thuộc runtime vào static globals (`ENV`, `METRICS`).
- Hợp nhất lỗi về `AppError` (giữ compatibility alias) và chuyển phần lớn luồng lỗi sang key-based i18n.
- Thêm catalog i18n (`vi`/`en`) và localize error theo `Accept-Language` tại middleware `request_context`.
- Chuẩn hóa constructor lỗi ở runtime path (`bad_request/forbidden/not_found/internal_error`) sang key-based variants.
- Cập nhật service/handler/repository liên quan: `user`, `friend`, `conversation`, `message`, `file_upload`, `call`, `middlewares`, `configs`, `utils`, `websocket`.
- Đồng bộ test để chấp nhận key-based variants và sửa setup test phụ thuộc env (`from_env_lossy`).

### Validation
- Passed `cargo check`.
- Passed `cargo check --tests`.
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).
- Passed `cargo clippy --tests`.
- Passed `cargo fmt --check`.

### Next
- Phase 2.1: Chuẩn hóa metadata/tracing cho error response.

### Documentation
- Chuẩn bị kickoff Phase 3 với checklist triển khai tại `docs/phase3-prep.md`.
- Cập nhật `docs/refactor.md` để đánh dấu trạng thái sẵn sàng khởi động Phase 3.

### Phase 3 - Batch 1 (Kickoff)
- Refactor cơ học `message repository` để tách SQL literals thành constants dùng lại.
- Thêm helper phân trang `pagination_fetch_limit` để thống nhất cách lấy `limit + 1` phục vụ cursor paging.
- Verify: passed `cargo check --tests`.

### Phase 3 - Batch 1 (Continue)
- Mở rộng refactor query constants/helper cho `friend repository`.
- Mở rộng tách SQL constants cho các query lặp ở `conversation repository`.
- Verify tiếp: passed `cargo check --tests`.

### Phase 3 - Batch 2 (Transaction Helpers - Partial)
- Chuẩn hóa transaction boundary bằng `begin_tx()` helper ở các service: `conversation`, `message`, `file_upload`, `friend`.
- Mục tiêu: giảm duplication `pool.begin()` và thống nhất entry point transaction theo service.
- Verify: passed `cargo check --tests` và `cargo clippy --tests`.

### Phase 3 - Batch 2 (Validation + Note)
- Full test suite passed: `cargo test` (39 passed, 0 failed, 3 ignored).
- Ghi nhận giới hạn kiến trúc hiện tại cho `call`: repository methods dùng pool nội bộ, chưa nhận `Executor`/transaction context; cần thay interface nếu muốn transaction-unify end-to-end cho call flow.

### Phase 3 - Batch 3 (Call Interface Tx-Ready)
- Mở rộng `call` repository traits với nhóm method `*_with_tx` để hỗ trợ transaction context rõ ràng.
- Refactor `CallService` write-flows (`initiate_call`, `respond_call`, `cancel_call`, `end_call`) để dùng transaction boundary khi repository hỗ trợ transactions.
- Thêm cơ chế fallback cho unit test mocks: repository mock có thể tắt transaction mode (`supports_transactions = false`) để giữ test độc lập khỏi Postgres runtime.
- Đồng bộ `call_test` mocks theo interface mới.

### Validation (Batch 3)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).
- Passed `cargo clippy --tests`.

### Phase 3 - Batch 3 (Repository Interface Cleanup)
- Bổ sung doc comments ngắn cho repository traits chính (`call`, `conversation`, `friend`, `message`) để chuẩn hóa intent theo read/write/tx context.
- Không đổi business logic hoặc API contract; thay đổi tập trung ở interface readability và maintainability.

### Validation (Batch 3 - Interface Cleanup)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).

### Phase 3 - Batch 3 (Full Repository Sweep)
- Mở rộng cleanup sang `user` + `file_upload` repository layer để đồng bộ pattern với các module trước đó.
- `user/repository_pg`: tách SQL constants khỏi inline queries để tăng readability/maintainability.
- `file_upload/repository_pg`: tách SQL constants cho create/find/delete.
- `user/repository` và `file_upload/repository`: bổ sung doc comments ngắn theo intent method.

### Validation (Batch 3 - Full Sweep)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).

### Phase 2.1 - Error Metadata/Tracing (Start)
- `api/error`: thêm accessor `code()` và bổ sung header `x-error-code` cho mọi error response.
- `middlewares/request_context`: bổ sung structured tracing cho error path, bao gồm `request_id`, `method`, `path`, `status`, `code`, `duration_ms`.
- Giữ nguyên shape error response body hiện tại để không phá API contract.

### Phase 2.1 - Error Metadata/Tracing (Continue)
- Mở rộng structured tracing cho cả success + error path với metadata runtime bổ sung: `locale`, `user_agent`, `client_ip`.
- Giữ nguyên API contract, chỉ tăng observability cho vận hành/debug.

### Phase 2.1 - Error Metadata/Tracing (Body Meta)
- Bổ sung metadata tùy chọn trong error body dưới field `meta` (ví dụ `retryable`) theo cơ chế opt-in.
- Mặc định **không bật** để giữ tương thích ngược; bật qua env `APP_ERROR_BODY_META=true`.
- Vẫn giữ nguyên hai field chính `code` và `message`.

### Validation (Phase 2.1 - Start)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).

### Validation (Phase 2.1 - Continue)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).

### Validation (Phase 2.1 - Body Meta)
- Passed `cargo test` (39 passed, 0 failed, 3 ignored).

### Next Step Prepared - Phase 4 Kickoff
- Tạo tài liệu chuẩn bị triển khai Phase 4 tại `docs/phase4-prep.md`.
- Chốt phạm vi/batch cho service-layer decoupling theo hướng incremental, giữ nguyên API contract.

### Phase 4 - Batch 1 (Message Policy Extraction Pilot)
- Refactor `message/service` để tách policy/validation khỏi orchestration send flow.
- Tách các policy helpers theo intent domain:
	- `get_member_conversation_or_err`
	- `resolve_route_for_conversation`
	- `resolve_direct_conversation`
	- `ensure_conversation_member`
	- `ensure_message_owner`
- Áp dụng helper vào các luồng chính: `send_message_to_conversation`, `send_direct_message_payload`, `delete_message`, `edit_message`.
- Bổ sung unit tests policy trong `message_test` cho membership/ownership constraints.

### Validation (Phase 4 - Batch 1)
- Passed `cargo test message_test`.
- Passed `cargo test` (42 passed, 0 failed, 3 ignored).

### Phase 4 - Batch 2 (Conversation Policy Cleanup)
- Refactor `conversation/service` để tách group permission rules khỏi orchestration methods.
- Thêm các policy helpers: `ensure_conversation_member`, `ensure_group_conversation`, `ensure_group_owner`, `ensure_member_removal_permission`.
- Áp dụng các helper vào flow: `mark_as_seen`, `update_group_info`, `add_member`, `remove_member`.
- Bổ sung unit tests cho helper policy ngay trong module `conversation/service`.

### Validation (Phase 4 - Batch 2)
- Passed `cargo test conversation::service::tests`.
- Passed `cargo test` (47 passed, 0 failed, 3 ignored).

### Phase 4 - Batch 3 (Call/Friend Orchestration Simplification)
- Refactor `call/service` để tách guard policies khỏi orchestration flow:
	- `ensure_call_member`
	- `ensure_call_status`
	- `ensure_call_initiator`
- Áp dụng helper policy vào `initiate_call`, `respond_call`, `cancel_call`, `end_call`.
- Refactor `friend/service` để tách policy helpers:
	- `ensure_not_self_friend_request`
	- `normalize_friend_pair`
	- `ensure_friend_request_receiver`
- Áp dụng helper policy vào `send_friend_request`, `accept_friend_request`, `decline_friend_request`.
- Bổ sung unit tests helper policy trong `call/service` và `friend/service`.

### Validation (Phase 4 - Batch 3)
- Passed `cargo test call::service::tests`.
- Passed `cargo test friend::service::tests`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Next Step Prepared - Phase 5 Kickoff
- Tạo tài liệu chuẩn bị triển khai Phase 5 tại `docs/phase5-prep.md`.
- Chốt phạm vi/batch cho WebSocket refactor theo hướng incremental, giữ nguyên event contract.

### Phase 5 - Batch 1 (Session Lifecycle Helpers)
- Refactor `websocket/server` để tách lifecycle policies khỏi method chính:
	- `remove_session_from_user`
	- `on_user_fully_disconnected`
	- `remove_user_from_all_rooms`
	- `record_recent_reconnect`
	- `attach_session_to_user`
- Áp dụng helper vào `disconnect` và `authenticate` để giảm branch lồng, giữ nguyên behavior realtime.

### Validation (Phase 5 - Batch 1)
- Passed `cargo test ws_server_test`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Phase 5 - Batch 2 (Room Membership Helpers)
- Refactor `websocket/server` để tách helper quản lý room membership:
	- `add_user_to_room`
	- `add_room_to_user`
	- `remove_user_from_room`
	- `remove_room_from_user`
- Áp dụng helper vào `join_room`, `leave_room`, và luồng cleanup room trong disconnect path.

### Validation (Phase 5 - Batch 2)
- Passed `cargo test ws_server_test`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Phase 5 - Batch 3 (Broadcast Policy Cleanup)
- Refactor `websocket/server` để tách helper policy cho user-delivery:
	- `send_json_to_user`
	- `broadcast_json_to_users`
- Áp dụng helper vào `send_to_user`, `send_to_users`, `broadcast_to_room`, `user_presence_changed`.
- Chuẩn hóa skip-user policy và giảm duplicate logic broadcast.

### Validation (Phase 5 - Batch 3)
- Passed `cargo test ws_server_test`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Next Step Prepared - Phase 6 Kickoff
- Tạo tài liệu chuẩn bị triển khai Phase 6 tại `docs/phase6-prep.md`.
- Chốt phạm vi/batch cho API Layer Cleanup theo hướng incremental, giữ nguyên API contract.

### Phase 6 - Batch 1 (Handler Flow Normalization)
- Refactor `friend/handle` để chuẩn hóa flow parse claims/path input bằng helper dùng lại.
- Refactor `message/handle` để chuẩn hóa các bước parse/guard/payload mapping bằng helper nội bộ.
- Giảm duplication ở handlers, giữ nguyên response envelope và semantics của endpoint hiện tại.

### Validation (Phase 6 - Batch 1)
- Passed `cargo test friend_test`.
- Passed `cargo test message_test`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Phase 6 - Batch 2 (Route Organization Cleanup)
- Chuẩn hóa route registration ở `friend/route`, `message/route`, `conversation/route` theo pattern nhất quán `configure -> *_scope()`.
- Loại bỏ wrapper dư thừa (`scope("")`) và import không dùng, giữ nguyên route paths.
- Không thay đổi mount structure và hành vi endpoint.

### Validation (Phase 6 - Batch 2)
- Passed `cargo test friend_test`.
- Passed `cargo test message_test`.
- Passed `cargo test conversation_test` (0 passed, 2 ignored do integration requirement).
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Phase 6 - Batch 3 (DTO/Validation Boundary Cleanup)
- Refactor `call/handler` để tách helper parse boundary (claims/path/cursor) và mapping user profile cho initiation flow.
- Refactor `conversation/handle` để tách helper claims/path parsing và validation checks ở API layer trước khi gọi service.
- Chuẩn hóa mapping request DTO -> service input, không thay đổi business policy ở service layer.

### Validation (Phase 6 - Batch 3)
- Passed `cargo test call_test`.
- Passed `cargo test conversation_test` (0 passed, 2 ignored do integration requirement).
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Next Step Prepared - Phase 7 Kickoff
- Tạo tài liệu chuẩn bị triển khai Phase 7 tại `docs/phase7-prep.md`.
- Chốt phạm vi/batch cho Testing & Documentation theo hướng incremental, không đổi API contract.

### Phase 7 - Batch 1 (Test Utility Consolidation)
- `tests/mock`: thêm utility dùng chung cho test setup (`lazy_mock_pool`, `test_redis_cache`).
- Cập nhật `call_test`, `friend_test`, `message_test` để tái sử dụng utility và giảm duplicate setup code.
- Dọn import thừa sau khi chuyển sang helper mới.

### Validation (Phase 7 - Batch 1)
- Passed `cargo test call_test`.
- Passed `cargo test friend_test`.
- Passed `cargo test message_test`.
- Passed `cargo test` (53 passed, 0 failed, 3 ignored).

### Phase 7 - Batch 2 (Targeted Coverage Uplift)
- `friend_test`: thêm test cho nhánh `send_friend_request` khi đã có pending request và nhánh hợp nhất danh sách trong `get_friend_requests`.
- `call_test`: thêm test cho các nhánh guard thiếu coverage (`respond_call` invalid status, `end_call` non-member, `cancel_call` invalid status).
- Tăng coverage cho các branch nhiều rủi ro regression sau refactor service/API.

### Validation (Phase 7 - Batch 2)
- Passed `cargo test friend_test`.
- Passed `cargo test call_test`.
- Passed `cargo test` (58 passed, 0 failed, 3 ignored).

### Phase 7 - Batch 3 (Documentation Pass)
- Thêm tài liệu module-level notes tại `docs/backend-module-notes.md` cho các module refactor chính (`message`, `conversation`, `call`, `websocket`).
- Cập nhật `backend/README.md` với test runbook theo module + full sanity và snapshot boundary sau refactor.
- Đồng bộ tài liệu vận hành/test theo trạng thái codebase hiện tại.

### Validation (Phase 7 - Batch 3)
- Passed `cargo test` (58 passed, 0 failed, 3 ignored).

### Next Step Prepared - Phase 8 Kickoff
- Tạo tài liệu chuẩn bị triển khai Phase 8 tại `docs/phase8-prep.md`.
- Chốt phạm vi/batch cho Performance & Observability theo hướng incremental, giữ nguyên API contract.

### Phase 8 - Batch 1 (Metrics Coverage Completion)
- Mở rộng `AppMetrics` với domain counters cho call flows:
	- `call_initiate_total`, `call_accept_total`, `call_reject_total`, `call_cancel_total`, `call_end_total`.
- Mở rộng `AppMetrics` với domain counters cho conversation write-flows:
	- `conversation_create_total`, `conversation_mark_seen_total`, `conversation_group_update_total`, `conversation_member_add_total`, `conversation_member_remove_total`.
- Bổ sung API increment methods, snapshot fields và Prometheus exposition cho toàn bộ counters mới.
- `CallService`: thêm constructor `with_dependencies_and_metrics(...)` và ghi nhận metrics ở các luồng thành công (`initiate`, `accept/reject`, `cancel`, `end`).
- `ConversationService`: thêm constructor `with_dependencies_and_metrics(...)` và ghi nhận metrics ở các luồng thành công (`create`, `mark_as_seen`, `update_group_info`, `add_member`, `remove_member`).
- `main.rs`: wire shared `app_state.metrics` vào `CallService` và `ConversationService` để dùng runtime metrics chung.

### Validation (Phase 8 - Batch 1)
- Passed `cargo test` (58 passed, 0 failed, 3 ignored).

### Phase 8 - Batch 2 (Baseline Measurement Setup)
- Thêm script smoke baseline tại `backend/scripts/perf_smoke.sh` để đo thời gian chạy cho các flow mục tiêu:
	- `message_send_guard`
	- `call_accept_flow`
	- `ws_fanout_flow`
	- `full_test_suite`
- Script hỗ trợ lặp theo env `PERF_ITERATIONS` (mặc định `3`) và sinh report markdown với `avg/p50/p95/min/max`.
- Thêm runbook ngắn trong `backend/README.md` để chạy baseline và lưu kết quả chuẩn tại `docs/perf-baseline-latest.md`.
- Sinh baseline đầu tiên bằng `PERF_ITERATIONS=1` để kiểm tra workflow end-to-end.

### Validation (Phase 8 - Batch 2)
- Passed `PERF_ITERATIONS=1 bash scripts/perf_smoke.sh`.

### Phase 8 - Batch 3 (Runbook & Guardrail Docs)
- Thêm runbook vận hành observability tại `docs/observability-runbook.md`.
- Bổ sung checklist theo dõi metrics theo domain (`HTTP`, `WS`, `message`, `call`, `conversation`, `upload`).
- Định nghĩa guardrail smoke cho baseline (`p95` tăng theo ngưỡng tương đối), upload failure ratio và WS close-quality signals.
- Bổ sung triage flow chuẩn khi có cảnh báo để khoanh vùng nhanh root-cause.
- Liên kết runbook vào `backend/README.md` để onboarding và vận hành thuận tiện.

### Validation (Phase 8 - Batch 3)
- Docs review completed (không đổi runtime behavior/API contract).

### Performance Follow-up (Post Phase 8)
- `utils/mod.rs`: refactor password hash/verify từ `rayon::spawn + oneshot` sang `tokio::task::spawn_blocking` để giảm overhead orchestration trong async path.
- `websocket/server.rs`: áp dụng hybrid fan-out strategy:
	- tập nhỏ chạy tuần tự để giảm scheduling overhead,
	- tập lớn dùng `rayon::par_iter()` để giữ throughput broadcast.
- Thêm benchmark harness cho WebSocket map contention tại `backend/benches/websocket_benchmark.rs` (DashMap vs Mutex<HashMap>) + cấu hình bench trong `backend/Cargo.toml`.
- Bổ sung guideline chọn công nghệ hiệu năng tại `docs/performance-guidelines.md` và liên kết từ runbook/README.

### Validation (Performance Follow-up)
- Passed `cargo test user_test`.
- Passed `cargo test` (58 passed, 0 failed, 3 ignored).
- Passed `cargo check --benches`.
- Passed `cargo bench --bench websocket_benchmark -- --sample-size 20 --measurement-time 2`.

### Benchmark Result Snapshot (WebSocket Map Concurrency)
- DashMap nhanh hơn Mutex<HashMap> ở tất cả case parallel reads/writes đã đo.
- Mức cải thiện quan sát được:
	- reads: khoảng `3.0x` đến `4.8x`
	- writes: khoảng `2.4x` đến `3.3x`
- Lưu baseline chi tiết tại `docs/websocket-benchmark-baseline.md`.
