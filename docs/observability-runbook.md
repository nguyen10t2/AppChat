# Observability Runbook (Backend)

## Mục tiêu
- Theo dõi sức khỏe runtime của backend theo 3 lớp: HTTP, WebSocket, domain flows.
- Phát hiện sớm regression sau refactor bằng baseline so sánh tương đối.
- Hướng dẫn triage nhanh khi phát hiện signal bất thường.

## Endpoint quan sát
- Prometheus text: `GET /metrics`
- JSON snapshot nội bộ: `GET /metrics/json`

## Metrics ưu tiên theo dõi

### 1) HTTP / nền tảng
- `app_http_requests_total`

### 2) WebSocket lifecycle
- `app_ws_reconnect_total`
- `app_ws_disconnect_total`
- `app_ws_close_total{reason="client_close|timeout|protocol_error|ping_failure|stream_end"}`

### 3) Message flow
- `app_message_send_total`
- `app_message_send_latency_ms_bucket`
- `app_message_send_latency_ms_sum`
- `app_message_send_latency_ms_count`

### 4) Call flow
- `app_call_initiate_total`
- `app_call_accept_total`
- `app_call_reject_total`
- `app_call_cancel_total`
- `app_call_end_total`

### 5) Conversation flow
- `app_conversation_create_total`
- `app_conversation_mark_seen_total`
- `app_conversation_group_update_total`
- `app_conversation_member_add_total`
- `app_conversation_member_remove_total`

### 6) Upload flow
- `app_upload_attempt_total`
- `app_upload_failure_total`

## Baseline workflow
1. Tạo baseline:
   - `PERF_ITERATIONS=3 bash backend/scripts/perf_smoke.sh`
2. So sánh report mới với report trước ở `docs/perf-baseline-latest.md`.
3. Chạy benchmark map-concurrency khi có thay đổi WebSocket state strategy:
   - `cargo bench --bench websocket_benchmark -- --sample-size 20 --measurement-time 2`
   - So sánh với `docs/websocket-benchmark-baseline.md`.
4. Nếu chênh lệch lớn, chạy lại cùng môi trường để loại nhiễu local.

## Guardrail đề xuất (smoke)

Các guardrail dưới đây dùng cho cảnh báo sớm sau refactor, không phải SLA tuyệt đối.

- Performance smoke:
  - Cảnh báo mức 1 nếu `p95` của một case tăng trên `25%` so với baseline gần nhất.
  - Cảnh báo mức 2 nếu `p95` tăng trên `40%` trong 2 lần đo liên tiếp.
- Message latency runtime:
  - Cảnh báo nếu tỷ lệ bucket `le="250"` giảm rõ rệt liên tục trong các lần scrape.
- Upload reliability:
  - Cảnh báo nếu `app_upload_failure_total / app_upload_attempt_total > 0.10` trong một cửa sổ quan sát ổn định.
- WS quality:
  - Cảnh báo nếu `timeout + ping_failure` close reason tăng đột biến tương đối so với `client_close`.

## Triage flow khi có cảnh báo
1. Xác minh phạm vi:
   - Chỉ một flow (`message/call/ws/upload`) hay toàn hệ thống.
2. Kiểm tra deploy delta:
   - So sánh thay đổi mới nhất trong service/repository/websocket path liên quan.
3. Thu thập snapshot:
   - Lấy `GET /metrics/json` trước/sau khi tái hiện issue.
4. Tái hiện có kiểm soát:
   - Chạy lại `backend/scripts/perf_smoke.sh` với cùng `PERF_ITERATIONS`.
5. Khoanh vùng root-cause:
   - Nếu chỉ tăng ở flow cụ thể: kiểm tra transaction/broadcast/path branch mới.
   - Nếu tăng toàn cục: kiểm tra DB/Redis connectivity và contention tài nguyên máy.
6. Quyết định xử lý:
   - Rollback nếu mức 2 + ảnh hưởng rõ user path.
   - Nếu mức 1, mở issue tối ưu và theo dõi thêm 1-2 chu kỳ đo.

## Cardinality & naming guardrails
- Chỉ dùng labels ổn định theo enum/domain key.
- Không thêm label chứa `user_id`, `conversation_id`, `call_id`, request path raw.
- Khi thêm metric mới, ưu tiên counter/histogram đơn giản, tránh nhiều chiều không cần thiết.

Guideline chọn primitive hiệu năng: `docs/performance-guidelines.md`.

## Gợi ý checklist PR (observability)
- Có thêm hoặc cập nhật metric ở flow thay đổi.
- Không làm vỡ endpoint `/metrics` và `/metrics/json`.
- Có baseline smoke sau thay đổi có ảnh hưởng performance.
- Cập nhật docs nếu thêm signal mới.
