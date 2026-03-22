# Phase 4 Prep - Service Layer Decoupling

## Mục tiêu
- Giảm trách nhiệm chồng chéo trong service layer, tách rõ orchestration và domain policies.
- Giữ nguyên API contract và behavior quan sát được từ client.
- Tăng testability bằng cách thu hẹp phạm vi mỗi service và giảm coupling xuyên module.

## Entry Criteria
- Phase 2.1 metadata/tracing đã triển khai và verify xanh.
- Phase 3 repository refactor đã hoàn tất phạm vi backend modules.
- Pipeline backend xanh:
  - `cargo check --tests`
  - `cargo test`
  - `cargo clippy --tests`

## Phạm vi Phase 4
1) Service Boundary Cleanup
- Tách orchestration flow khỏi validation/policy logic cài cắm trong service methods dài.
- Chuẩn hóa helper naming theo intent domain thay vì implementation detail.

2) Domain Policy Extraction
- Tách rule checks thành hàm/policy units độc lập, dễ test.
- Ưu tiên các flow nhiều nhánh hiện tại: `message`, `conversation`, `call`, `friend`.

3) Dependency Narrowing
- Giảm số dependency trực tiếp trong service constructors khi có thể.
- Giữ DI pattern đã có, tránh thêm global/static mới.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Pilot tách policy cho message flow
- Tách nhóm rule validation (route/message type/reply constraints) khỏi flow gửi tin nhắn.
- Mục tiêu: giảm độ dài method và tách unit-test theo policy path.
- Verify: `cargo test message_test` + `cargo test`.

### Batch 2 - Conversation policy cleanup
- Tách rule quyền nhóm (owner/member/add/remove/update) thành helper/policy units rõ ràng.
- Giữ nguyên transaction boundaries đã chuẩn hóa ở Phase 3.
- Verify: `cargo test conversation_test` (nếu khả dụng) + `cargo test`.

### Batch 3 - Call/Friend orchestration simplification
- Tối giản branch logic trong call/friend service methods bằng helper domain-level.
- Giữ tx-aware repository usage hiện tại, không thay đổi API routes.
- Verify: `cargo test call_test friend_test` + `cargo test`.

## Definition of Done
- Service methods chính giảm độ phức tạp (ít nested branch hơn, intent rõ hơn).
- Policy checks có unit tests hoặc coverage qua test hiện hữu.
- Không thay đổi shape response API (`code`, `message`, dữ liệu business).
- Tài liệu cập nhật trong `docs/refactor.md` và `docs/changelog.md`.

## Rủi ro và giảm thiểu
- Rủi ro: tách helper sai boundary làm đổi behavior.
  - Giảm thiểu: refactor cơ học từng batch nhỏ + chạy full test mỗi batch.
- Rủi ro: over-abstraction gây khó đọc.
  - Giảm thiểu: ưu tiên hàm nhỏ gần domain, không tạo layer mới không cần thiết.
- Rủi ro: đụng nhiều module cùng lúc.
  - Giảm thiểu: pilot theo module, đóng batch độc lập.
