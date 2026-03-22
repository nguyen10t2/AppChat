# Phase 6 Prep - API Layer Cleanup

## Mục tiêu
- Đơn giản hóa handlers bằng cách tách rõ orchestration, validation, và response shaping.
- Giữ nguyên API contract hiện tại (status code, response envelope, error code/message).
- Tăng tính nhất quán giữa các module route/handle để dễ mở rộng và bảo trì.

## Entry Criteria
- Phase 5 đã hoàn thành (WebSocket refactor xong 3 batch).
- Backend test suite đang xanh ổn định.
- Error model key-based + metadata/tracing đã ổn định từ Phase 2/2.1.

## Phạm vi Phase 6
1) Handler Standardization
- Chuẩn hóa flow chung trong handlers: auth claims → validation → service call → success response.
- Tối giản duplicate pattern xử lý input/response giữa các module.

2) Route Organization
- Chuẩn hóa naming và cấu trúc register routes (`configure` pattern nhất quán).
- Gom nhóm endpoint registration theo domain rõ ràng.

3) DTO & Validation Boundary
- Tách rõ request DTO và mapping sang service input (nơi cần).
- Chuẩn hóa validation boundary tại API layer, tránh validation business bị tràn vào handler.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Handler flow normalization (pilot)
- Pilot trên `message` + `friend` handlers để chuẩn hóa flow vào/ra.
- Tách helper nội bộ cho pattern response thành công/lỗi thường gặp.
- Verify: `cargo test friend_test message_test` + `cargo test`.

### Batch 2 - Route organization cleanup
- Chuẩn hóa route registration ở các module còn naming/chia nhóm chưa đồng đều.
- Loại bỏ lặp cấu hình scope/middleware cục bộ không cần thiết (giữ behavior).
- Verify: `cargo test`.

### Batch 3 - DTO/validation boundary cleanup
- Chuẩn hóa mapping request DTO -> service payload ở các handlers dài.
- Giữ validation syntax tại API layer, không đổi business rules service.
- Verify: `cargo test` + kiểm tra các integration test có sẵn.

## Definition of Done
- Handlers ngắn hơn, ít branch lồng và nhất quán flow xử lý.
- Không thay đổi API contract phía frontend.
- Không thay đổi key semantics trong error response.
- Tài liệu cập nhật trong `docs/refactor.md` và `docs/changelog.md`.

## Rủi ro và giảm thiểu
- Rủi ro: đổi cấu trúc handler làm lệch status code/response envelope.
  - Giảm thiểu: refactor cơ học theo batch nhỏ + chạy test sau từng batch.
- Rủi ro: validation boundary đặt sai layer gây đổi behavior.
  - Giảm thiểu: chỉ di chuyển logic trình bày/input, không đổi policy business ở service.
- Rủi ro: route organization làm sai mount path.
  - Giảm thiểu: giữ nguyên path strings và verify bằng test hiện hữu.
