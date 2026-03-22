# Phase 3 Prep - Repository Layer Refactor

## Mục tiêu
- Tách biệt rõ data access logic để service layer mỏng hơn.
- Giảm trùng lặp query, thống nhất transaction boundary, giữ nguyên business behavior.
- Không đổi API contract, không đổi shape response.

## Entry Criteria
- Backend pass toàn bộ kiểm tra:
  - `cargo check`
  - `cargo check --tests`
  - `cargo test`
  - `cargo clippy --tests`
  - `cargo fmt --check`
- Error handling Phase 2 core đã ổn định (AppError + key-based i18n).
- Không còn blocker migration từ ENV/METRICS static sang AppState/AppConfig.

## Phạm vi Phase 3
1) Query Builder Pattern
- Extract query fragments dùng lặp nhiều lần (where/search/pagination).
- Chuẩn hóa query paging theo một flow nhất quán.
- Giảm inline SQL điều kiện phức tạp ở repository methods.

2) Transaction Management
- Tạo helper cho begin/commit/rollback theo pattern thống nhất.
- Chuẩn hóa boundary: service orchestrate, repository tập trung data operations.
- Giảm duplicate đoạn commit/rollback guard.

3) Repository Interface
- Rà soát trait signatures để rõ intent (read/write/tx context).
- Bổ sung doc comments ngắn cho method quan trọng.
- Chuẩn hóa naming theo semantics thay vì theo implementation detail.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Query reuse primitives
- Tạo module nhỏ cho query helpers (cursor parse, limit clamp, where builder tối giản).
- Áp dụng trước ở repository có query lặp nhiều: message/conversation/friend.
- Verify: `cargo check --tests`.

### Batch 2 - Transaction helpers
- Tạo helper transaction dùng chung trong repository/service orchestration.
- Refactor các flow có nhiều bước ghi DB (conversation/message/call).
- Verify: `cargo test`.

Tiến độ hiện tại:
- [x] Đã chuẩn hóa `begin_tx()` nội bộ cho các service có write flow nhiều bước: `conversation`, `message`, `file_upload`, `friend`.
- [x] Verify trung gian: `cargo check --tests` + `cargo clippy --tests`.
- [x] Chạy full `cargo test` (39 passed, 0 failed, 3 ignored).
- [x] Nâng `call` repository interface theo hướng tx-ready (`*_with_tx`) và áp dụng transaction boundary cho các write-flow trong `CallService` (`initiate/respond/cancel/end`).
- [x] Đồng bộ test doubles `call_test` cho interface mới; thêm fallback không dùng transaction cho mock repository (`supports_transactions = false`).
- [x] Verify sau cập nhật call flow: `cargo test` (39 passed, 0 failed, 3 ignored).
- [x] Chuẩn hóa docs cho repository traits (`call`, `conversation`, `friend`, `message`) để làm rõ intent read/write/tx context.
- [x] Mở rộng repository cleanup cho các module còn lại: `user` + `file_upload` (doc comments + SQL constants refactor).
- [x] Verify sau interface cleanup: `cargo test` (39 passed, 0 failed, 3 ignored).

### Batch 3 - Interface cleanup
- Chuẩn hóa trait method signatures + docs tối thiểu.
- Đồng bộ test doubles trong `src/tests/**`.
- Verify: `cargo clippy --tests` + `cargo fmt --check`.

## Definition of Done
- Không đổi business logic quan sát được từ API.
- Các thay đổi chia theo batch nhỏ, mỗi batch có thể review/merge độc lập.
- Toàn bộ pipeline xanh sau mỗi batch.
- Tài liệu cập nhật lại trong `docs/refactor.md` + `docs/changelog.md`.

## Ghi chú cross-phase
- Song song Phase 3, đã bắt đầu triển khai Phase 2.1 cho error metadata/tracing ở middleware + error response headers để tăng khả năng truy vết runtime.

## Rủi ro chính và cách giảm thiểu
- Rủi ro: refactor transaction làm thay đổi thứ tự side-effect.
  - Giảm thiểu: giữ test hiện có chạy mỗi batch + ưu tiên refactor cơ học trước.
- Rủi ro: generic hóa quá mức làm code khó đọc.
  - Giảm thiểu: ưu tiên helper nhỏ, tránh abstraction sâu ngay từ đầu.
- Rủi ro: đụng đồng thời nhiều module gây xung đột merge.
  - Giảm thiểu: chia PR theo module và batch.

## Checklist kickoff ngày 1
- [x] Chọn 1 module pilot cho query helper: `message repository`.
- [x] Tách SQL constants + helper phân trang (`pagination_fetch_limit`) trong `message/repository_pg.rs`.
- [x] Mở rộng cùng pattern sang `friend/repository_pg.rs` và `conversation/repository_pg.rs` (SQL constants + helper tái sử dụng).
- [ ] Chốt naming convention cho helper và transaction boundary.
- [ ] Mở batch đầu với phạm vi tối thiểu + test verify đầy đủ.
