# Phase 7 Prep - Testing & Documentation

## Mục tiêu
- Tăng độ tin cậy của refactor bằng test coverage có trọng tâm cho các luồng đã chuẩn hóa.
- Chuẩn hóa tài liệu kỹ thuật theo module để onboarding và bảo trì dễ hơn.
- Giữ nguyên behavior/API contract trong quá trình bổ sung test + docs.

## Entry Criteria
- Phase 6 đã hoàn thành (API Layer Cleanup xong 3 batch).
- Full backend test suite đang xanh ổn định.
- Các thay đổi kiến trúc chính từ Phase 1-6 đã được cập nhật trong `docs/refactor.md` và `docs/changelog.md`.

## Phạm vi Phase 7
1) Test Infrastructure Hardening
- Củng cố test utilities dùng chung cho mocks/setup reusable.
- Bổ sung coverage có mục tiêu cho các helper/policy tách ra ở các phase trước.

2) Integration Test Hygiene
- Chuẩn hóa cách đánh dấu/ghi chú test phụ thuộc Postgres (`ignored` rationale rõ ràng).
- Rà soát naming và grouping test modules để phản ánh domain boundaries hiện tại.

3) Documentation Consolidation
- Bổ sung module-level notes cho các module refactor nhiều (`message`, `conversation`, `call`, `websocket`).
- Cập nhật hướng dẫn test/runbook backend ngắn gọn, bám trạng thái hiện tại của dự án.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Test utility consolidation
- Chuẩn hóa helper dùng chung trong `backend/src/tests/mock` và các test module liên quan.
- Giảm duplicate setup code không cần thiết.
- Verify: `cargo test`.

### Batch 2 - Targeted coverage uplift
- Thêm/điều chỉnh test cho các policy/helper mới ở service/handler khi có khoảng trống rõ ràng.
- Không thêm test “trùng ý nghĩa”; ưu tiên branch có rủi ro regression.
- Verify: test module liên quan + `cargo test`.

### Batch 3 - Documentation pass
- Cập nhật README/docs cho quy trình test backend và snapshot kiến trúc sau refactor.
- Đồng bộ trạng thái phase/checklist trong docs chính.
- Verify: review docs + `cargo test` sanity.

## Definition of Done
- Test utilities được chuẩn hóa, giảm code setup lặp lại.
- Coverage được cải thiện ở các điểm policy/branch quan trọng sau refactor.
- Tài liệu module và hướng dẫn test phản ánh đúng trạng thái hiện tại.
- Không thay đổi API contract hoặc business behavior.

## Rủi ro và giảm thiểu
- Rủi ro: thêm test phụ thuộc hạ tầng gây flaky.
  - Giảm thiểu: tách rõ unit vs integration; giữ `ignored` có chú thích với test cần Postgres.
- Rủi ro: test mới gắn chặt implementation details.
  - Giảm thiểu: ưu tiên assert hành vi quan sát được thay vì assert nội bộ quá sâu.
- Rủi ro: docs lệch code sau khi cập nhật.
  - Giảm thiểu: cập nhật docs theo batch nhỏ và chạy test sanity sau mỗi batch.
