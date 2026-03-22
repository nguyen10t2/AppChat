Tôi đã phân tích toàn bộ backend codebase và nhận diện được nhiều code smell và vấn đề kiến trúc cần refactor. Dưới đây là tổng quan và kế hoạch chi tiết theo các phase.

## ✅ Cập nhật tiến độ (2026-03-23)

### Phase 1: Cấu trúc & Configuration — **HOÀN THÀNH**
- [x] Thêm `AppConfig` (load từ env qua `config` crate) + validation startup.
- [x] Thêm `AppState` tập trung (`config`, `metrics`) và inject qua `web::Data`.
- [x] Loại bỏ runtime dependency vào static `ENV` / `METRICS` global.
- [x] Refactor các module chính để dùng DI (`middleware`, `user`, `message`, `file_upload`, `websocket`).
- [x] Hỗ trợ migration path theo config (`MIGRATION_PATH`) thay vì hardcode.
- [x] Xóa module constants legacy không còn sử dụng.

### Phase 2: Error Handling Unification — **HOÀN THÀNH (CORE)**
- [x] Hợp nhất `Error` + `SystemError` về một nguồn `AppError` (giữ compatibility alias).
- [x] Tách error catalog/i18n thành module riêng, tránh hardcoded message rải rác.
   - Đã áp dụng cho `middleware auth`, `user service`, `file_upload/call/message handlers`, `friend/message/call services`, `conversation service`, `file_upload service`.
   - Đã thêm i18n nền tảng (`vi` / `en`) với `Accept-Language` cho các lỗi đi qua request path.
   - Cách dùng hiện tại: gửi header `Accept-Language: en` để nhận lỗi tiếng Anh ở các điểm đã rollout.
   - Đã bổ sung cơ chế localize tập trung tại middleware `request_context`: lỗi `AppError` từ service/handler sẽ được dịch tự động theo locale nếu message map được về key catalog.
   - Đã bắt đầu migration `AppError` sang key-based variants (`BadRequestKey`, `UnauthorizedKey`, ...), giúp giảm phụ thuộc map theo string literal.
   - Đã áp dụng key-based constructors cho middleware auth/authorization và các handler chính (`file_upload`, `call`, `message`).
   - Đã mở rộng key-based vào service/repository quan trọng: `user`, `friend`, `conversation`, `call`, `message`, `file_upload`, `user repository`.
   - Đã chuẩn hóa thêm runtime path ở `utils` + `configs` sang `internal_error_key(...)`.
   - Đã hoàn tất dọn literal còn lại trong test doubles và đồng bộ key-based constructor ở các test chính.
   - Đã sweep toàn bộ runtime constructor lỗi `bad_request/forbidden/not_found/internal_error` để loại bỏ static string literal, chuyển sang key-based i18n.
   - Đã đồng bộ test theo key-based variants để tránh false-fail khi migrate error variant.
   - Đã chạy pass toàn bộ pipeline xác minh:
     - `cargo check`
     - `cargo check --tests`
     - `cargo test`
     - `cargo clippy --tests`
     - `cargo fmt --check`

### Phase 2.1: Metadata/Tracing cho Error Response — **HOÀN THÀNH**
- [x] Bổ sung metadata header ổn định cho error response: `x-error-code`.
- [x] Bổ sung structured tracing cho error path tại middleware `request_context` (request_id, method, path, status, code, duration_ms).
- [x] Mở rộng tracing metadata cho runtime observability: `locale`, `user_agent`, `client_ip` (success + error path).
- [x] Chuẩn hóa mở rộng metadata response body ở chế độ opt-in (`APP_ERROR_BODY_META=true`) để giữ backward compatibility.

### Phase 3: Repository Layer Refactor — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase3-prep.md`.
- [x] Đã chốt entry criteria:
   - Codebase đang xanh toàn pipeline (`check`, `test`, `clippy`, `fmt`).
   - Error model + i18n key-based đã ổn định ở runtime path.
   - Có thể tách Phase 3 thành các batch nhỏ, mỗi batch deploy độc lập.
- [x] Bắt đầu Batch 1 (pilot ở `message repository`: tách SQL constants + helper phân trang).
- [x] Mở rộng Query Builder Pattern sang `conversation/friend` repository (tách SQL constants/helper tái sử dụng).
- [x] Chuẩn hóa transaction entry helper cho các service write-flow chính (`conversation`, `message`, `file_upload`, `friend`).
- [x] Batch 3 bước đầu cho `call`: nâng repository interface tx-ready (`*_with_tx`) + áp dụng transaction boundary ở `CallService` với fallback cho mock tests.
- [x] Interface cleanup: bổ sung docs ngắn cho repository traits chính (`call`, `conversation`, `friend`, `message`) để làm rõ intent read/write/tx.
- [x] Full sweep repository layer cho module còn lại (`user`, `file_upload`): tách SQL constants và đồng bộ docs trait methods.

### Phase 4: Service Layer Decoupling — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase4-prep.md`.
- [x] Batch 1 pilot: tách policy/validation khỏi message service flow.
   - `message/service`: tách policy helpers theo intent domain:
      - `get_member_conversation_or_err`
      - `resolve_route_for_conversation`
      - `resolve_direct_conversation`
      - `ensure_conversation_member`
      - `ensure_message_owner`
   - Giảm branch lồng trong `send_message_to_conversation`, `send_direct_message_payload`, `delete_message`, `edit_message`.
   - Bổ sung unit tests policy ở `message_test` cho membership/ownership checks.
- [x] Batch 2: conversation policy cleanup.
   - `conversation/service`: tách group policy helpers:
      - `ensure_conversation_member`
      - `ensure_group_conversation`
      - `ensure_group_owner`
      - `ensure_member_removal_permission`
   - Áp dụng helper vào các flow: `mark_as_seen`, `update_group_info`, `add_member`, `remove_member`.
   - Bổ sung unit tests helper policy trong `conversation/service`.
- [x] Batch 3: call/friend orchestration simplification.
   - `call/service`: tách policy guards `ensure_call_member`, `ensure_call_status`, `ensure_call_initiator` và áp dụng cho các flow `initiate_call`, `respond_call`, `cancel_call`, `end_call`.
   - `friend/service`: tách policy helpers `ensure_not_self_friend_request`, `normalize_friend_pair`, `ensure_friend_request_receiver` và áp dụng cho `send_friend_request`, `accept_friend_request`, `decline_friend_request`.
   - Bổ sung unit tests helper policy trong cả `call/service` và `friend/service`.

### Phase 5: WebSocket Refactor — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase5-prep.md`.
- [x] Batch 1: session lifecycle helpers.
   - `websocket/server`: tách helper cho lifecycle session/auth:
      - `remove_session_from_user`
      - `on_user_fully_disconnected`
      - `remove_user_from_all_rooms`
      - `record_recent_reconnect`
      - `attach_session_to_user`
   - Áp dụng helper vào flow `disconnect` và `authenticate` để giảm branch lồng và giữ nguyên side-effects.
- [x] Batch 2: room membership helpers.
   - `websocket/server`: tách helper quản lý membership:
      - `add_user_to_room`
      - `add_room_to_user`
      - `remove_user_from_room`
      - `remove_room_from_user`
   - Áp dụng helper vào `join_room`, `leave_room`, và cleanup room khi user disconnect.
- [x] Batch 3: broadcast policy cleanup.
   - `websocket/server`: tách helper delivery policy:
      - `send_json_to_user`
      - `broadcast_json_to_users`
   - Áp dụng helper vào `send_to_user`, `send_to_users`, `broadcast_to_room`, `user_presence_changed`.
   - Chuẩn hóa skip-user policy và user-list delivery flow, giữ nguyên event contract.

### Phase 6: API Layer Cleanup — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase6-prep.md`.
- [x] Batch 1: handler flow normalization (pilot `message` + `friend`).
   - `friend/handle`: chuẩn hóa flow claims/path parsing bằng helper dùng lại (`current_user_id`, `path_id`).
   - `message/handle`: chuẩn hóa helper cho recipient parsing, friendship check, membership check và payload mapping (`require_recipient_id`, `ensure_friendship`, `ensure_conversation_membership`, `build_direct_payload`).
   - Giảm duplicate logic handler, giữ nguyên endpoint behavior/response contract.
- [x] Batch 2: route organization cleanup.
   - `friend/route`, `message/route`, `conversation/route`: chuẩn hóa pattern `configure -> *_scope()`.
   - Loại bỏ cấu trúc route dư thừa (`scope("")`) và import không dùng.
   - Giữ nguyên mount paths và endpoint behavior.
- [x] Batch 3: DTO/validation boundary cleanup.
   - `call/handler`: tách helper boundary cho claims/path/cursor parse và user profile mapping (`current_user_id`, `path_call_id`, `parse_history_cursor`, `load_call_user_profile`).
   - `conversation/handle`: tách helper boundary cho claims/path parse và validation checks ở API layer (`current_user_id`, `path_uuid`, `ensure_conversation_member`, `ensure_can_create_conversation_membership`).
   - Chuẩn hóa mapping input DTO -> service calls, giữ nguyên business rules ở service layer.

### Phase 7: Testing & Documentation — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase7-prep.md`.
- [x] Batch 1: test utility consolidation.
   - `tests/mock`: bổ sung shared helpers `lazy_mock_pool` và `test_redis_cache` cho setup dùng chung.
   - Cập nhật `call_test`, `friend_test`, `message_test` để dùng utility mới, giảm lặp setup.
   - Dọn import thừa sau refactor để giữ test compile sạch.
- [x] Batch 2: targeted coverage uplift.
   - `friend_test`: bổ sung coverage cho nhánh pending request conflict và merge kết quả `get_friend_requests` (to + from).
   - `call_test`: bổ sung coverage cho các nhánh guard quan trọng (`respond_call` sai status, `end_call` non-member, `cancel_call` sai status).
   - Tăng coverage branch rủi ro regression sau các phase service/API cleanup.
- [x] Batch 3: documentation pass.
   - Bổ sung tài liệu module-level notes tại `docs/backend-module-notes.md` cho `message`, `conversation`, `call`, `websocket`.
   - Cập nhật `backend/README.md` với test runbook theo module + full sanity.
   - Đồng bộ snapshot boundary sau refactor để onboarding/bảo trì thuận tiện.

### Phase 8: Performance & Observability — **HOÀN THÀNH**
- [x] Đã chuẩn bị tài liệu kickoff và checklist triển khai tại `docs/phase8-prep.md`.
- [x] Batch 1: metrics coverage completion.
- [x] Batch 2: baseline measurement setup.
- [x] Batch 3: runbook & guardrail docs.

### Ghi chú migration
- Không đổi business logic.
- Không đổi API contract response shape (`code`, `message`).
- Refactor theo incremental steps, có thể deploy độc lập từng phase.

## 📊 Tổng quan các vấn đề chính

### 1. **SOLID Principles Violations**
- **Single Responsibility**: Các service class có quá nhiều trách nhiệm (DB + business logic + WebSocket + cache management)
- **Open/Closed Principle**: Hardcoded dependencies, khó mở rộng cho implement khác
- **Dependency Inversion**: Repository implementations được trực tiếp vào service thay vì injection qua trait

### 2. **Code Duplication & Complexity**
- Logic password hashing/verification dùng blocking thread (rayon) thay vì async
- Message routing logic trộn lẫn giữa direct/group messages
- Complex SQL CASE WHEN statements cho conditional updates
- JWT encode/decode logic trộn với data structures

### 3. **Error Handling Issues**
- Hai enum Error (`Error` và `SystemError`) gây nhầm lẫn
- Hardcoded error messages tiếng Việt trộn lẫn với catalog constants
- Conflict message logic phức tạp với constraint parsing thủ công

### 4. **Tight Coupling**
- Services phụ thuộc trực tiếp vào concrete repository implementations (UserRepositoryPg, etc.)
- Configuration scattered khắp nơi (ENV static, hardcode values)
- CORS origin logic lẫn lẫn

### 5. **Lack of Abstraction**
- Cache keys format thủ công, trộn lẫn logic
- WebSocket server làm quá nhiều việc (session, room, broadcasting, presence)
- No clear separation giữa data access và business logic

### 6. **Testing Challenges**
- Hardcoded database migration path
- Static ENV và METRICS global khó test
- Complex transaction management khó mock

### 7. **Code Smell**
- God Objects (main.rs với 20+ dependency injection)
- Magic strings và numbers (CACHE_TTL = 300, RECONNECT_WINDOW = 120s)
- Long methods với nhiều nested conditions
- Inconsistent naming conventions (handle.rs vs handler.rs, route vs configure)

## 🎯 Kế hoạch Refactor theo Phases

### **Phase 1: Cấu trúc & Configuration** ⏱️ 2-3 ngày
**Mục tiêu**: Tách biệt concerns và tạo abstraction layer

**Công việc**:
1. **Config System Refactor**
   - Tạo `AppConfig` struct với builder pattern
   - Dùng `config` crate thay vì `std::env::var` trực tiếp
   - Tạo environment-specific configs (dev, test, prod)
   - Validate configuration at startup với clear error messages

2. **Dependency Injection**
   - Tạo `AppState` struct centralized
   - Dùng generic trait injection thay vì concrete types
   - Xóa static `ENV` và `METRICS` global
   - Implement Service container hoặc manual DI

3. **Database & Cache Abstraction**
   - Tạo `DbPool` trait wrapper
   - Centralize migration path configuration
   - Cache key generation logic tách biệt
   - Consider Redis serialization format optimization

### **Phase 2: Error Handling Unification** ⏱️ 2-3 ngày
**Mục tiêu**: Đơn giản hóa và thống nhất error handling

**Công việc**:
1. **Merge Error Enums**
   - Gộp `Error` và `SystemError` thành single enum
   - Tạo `AppError` enum với clear variants
   - Implement `From` conversions cho tất cả external errors

2. **Error Catalog & i18n**
   - Tách error messages vào file riêng
   - Dùng error codes thay vì raw messages
   - Implement message template system

3. **Error Middleware Enhancement**
   - Standardize error response format
   - Add request tracing cho errors
   - Implement error context/metadata

### **Phase 3: Repository Layer Refactor** ⏱️ 3-4 ngày
**Mục tiêu**: Tách biệt data access logic

**Công việc**:
1. **Query Builder Pattern**
   - Extract common query patterns
   - Tạo reusable query components
   - Simplify complex conditional updates

2. **Transaction Management**
   - Tạo transaction helper abstraction
   - Centralize commit/rollback logic
   - Implement transaction boundaries

3. **Repository Interface**
   - Tất cả repository traits
   - Add method-level documentation
   - Consider pagination abstraction

### **Phase 4: Service Layer Decoupling** ⏱️ 4-5 ngày
**Mục tiêu**: Giảm responsibilities của service classes

**Công việc**:
1. **Split Service Responsibilities**
   - UserService → UserService + AuthService + CacheService
   - MessageService → MessageService + RoutingService
   - ConversationService → ConversationService + PermissionService

2. **Business Logic Extraction**
   - Tạo validators riêng cho business rules
   - Extract policy logic (friendship rules, call rules)
   - Implement domain events

3. **Async Password Operations**
   - Dùng async password hashing library
   - Xóa rayon blocking threads
   - Implement rate limiting

### **Phase 5: WebSocket Refactor** ⏱️ 3-4 ngày
**Mục tiêu**: Tách biệt WebSocket state management

**Công việc**:
1. **Session Management**
   - Tạo `SessionManager` trait
   - Implement lifecycle management
   - Add session metrics

2. **Room Management**
   - Tạo `RoomManager` trait
   - Implement room lifecycle
   - Add room-level metrics

3. **Message Broadcasting**
   - Tạo `BroadcastService` riêng
   - Implement message serialization optimization
   - Add broadcast queue với backpressure

### **Phase 6: API Layer Cleanup** ⏱️ 2-3 ngày
**Mục tiêu**: Đơn giản hóa handlers

**Công việc**:
1. **Handler Standardization**
   - Tạo `HandlerResponse` trait
   - Standardize request validation flow
   - Implement common authentication pattern

2. **Route Organization**
   - Consistent naming: `configure()` vs `setup()`
   - Extract common middleware setup
   - Group related endpoints

3. **Request/Response DTOs**
   - Create dedicated request/response models
   - Implement validation rules
   - Add API versioning consideration

### **Phase 7: Testing & Documentation** ⏱️ 2-3 ngày
**Mục tiêu**: Tăng test coverage và documentation

**Công việc**:
1. **Test Infrastructure**
   - Create test utilities for mocking
   - Implement integration test fixtures
   - Add property-based tests

2. **Documentation**
   - Module-level documentation
   - API documentation generation
   - Architecture decision records

### **Phase 8: Performance & Observability** ⏱️ 2-3 ngày
**Mục tiêu**: Tăng performance và monitoring

**Công việc**:
1. **Metrics Enhancement**
   - Structured metrics collection
   - Add performance benchmarks
   - Implement alerting

2. **Caching Strategy**
   - Review cache hit rates
   - Implement cache invalidation
   - Add multi-level caching

3. **Connection Pooling**
   - Optimize DB pool sizes
   - Implement connection health checks
   - Add circuit breakers

## 📋 Chi tiết triển khai

Tôi sẽ tạo document chi tiết cho từng phase trong thư mục `docs/` với:
- Refactor Plan Overview
- Architecture Decisions (ADR - Architecture Decision Records)
- Implementation Guides cho từng phase
- Testing Strategy
- Migration Path (rollback plan)

## ⚠️ Lưu ý quan trọng

1. **Không rewrite logic**: Chỉ refactor structure, không thay đổi business logic
2. **Incremental Migration**: Mỗi phase phải có thể deploy độc lập
3. **Backward Compatibility**: Giữ API contract unchanged trong giai đoạn refactor
4. **Testing Priority**: Mỗi phase phải có test coverage > 80%
5. **Documentation**: Tất cả thay đổi phải được document

## ✅ Trạng thái hiện tại

- Phase 1 đã hoàn thành.
- Phase 2 (core) đã hoàn thành và đã qua pipeline kiểm chứng.
- Hạng mục kế tiếp gần nhất: Phase 2.1 (metadata/tracing cho error response).