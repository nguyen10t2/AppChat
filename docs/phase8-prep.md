# Phase 8 Prep - Performance & Observability

## Mục tiêu
- Tăng khả năng quan sát hệ thống runtime bằng metrics/tracing có ngữ cảnh rõ ràng theo domain.
- Thiết lập baseline performance thực tế cho các luồng chính (message/call/websocket) để phát hiện regression sớm.
- Tối ưu ở mức an toàn, không thay đổi API contract và behavior business.

## Entry Criteria
- Phase 7 hoàn thành (test/docs đã đồng bộ trạng thái codebase).
- Full backend test suite đang xanh ổn định.
- Error metadata/tracing foundation từ Phase 2.1 đã sẵn sàng để mở rộng.

## Phạm vi Phase 8
1) Metrics Enhancement
- Bổ sung metrics domain-level cho call/conversation/message flows còn thiếu.
- Chuẩn hóa naming labels để tránh cardinality cao không cần thiết.

2) Performance Baseline
- Thiết lập benchmark/smoke performance nhẹ cho các luồng cốt lõi.
- Định nghĩa ngưỡng theo percentile cơ bản (p50/p95) để theo dõi xu hướng.

3) Runtime Guardrails
- Rà soát cấu hình pool/cache/timeouts để giảm risk bottleneck phổ biến.
- Cập nhật hướng dẫn observability runbook cho đội vận hành.

## Kế hoạch triển khai (đề xuất)
### Batch 1 - Metrics coverage completion
- Mở rộng metrics ở các điểm write-flow chính chưa có signal rõ.
- Đồng bộ metadata tags tối thiểu cần cho debug runtime.
- Verify: `cargo test` + check endpoint metrics.

### Batch 2 - Baseline measurement setup
- Thêm script/checklist benchmark nhẹ phục vụ so sánh trước/sau thay đổi.
- Ưu tiên các luồng: send message, ws fan-out, call status transitions.
- Verify: benchmark smoke + `cargo test`.

### Batch 3 - Runbook & guardrail docs
- Cập nhật docs vận hành cho metrics/tracing/alerts cơ bản.
- Ghi rõ ngưỡng theo dõi và triage flow khi detect degradation.
- Verify: docs review + `cargo test` sanity.

## Definition of Done
- Có thêm tín hiệu metrics/tracing ở các luồng quan trọng chưa được bao phủ đủ.
- Có baseline performance đơn giản, lặp lại được.
- Có tài liệu vận hành observability cập nhật theo codebase hiện tại.
- Không thay đổi API contract hoặc semantics nghiệp vụ.

## Rủi ro và giảm thiểu
- Rủi ro: metrics labels quá chi tiết gây tăng cardinality.
  - Giảm thiểu: giới hạn labels theo enum/domain key ổn định.
- Rủi ro: benchmark thiếu tính ổn định do môi trường local dao động.
  - Giảm thiểu: dùng smoke benchmark cho xu hướng tương đối, không coi là SLA tuyệt đối.
- Rủi ro: tối ưu sớm gây phức tạp code path.
  - Giảm thiểu: ưu tiên observability-first, tối ưu có đo lường và rollout nhỏ.
