# AppChat Backend

Backend cho hệ thống chat realtime, xây bằng Rust với Actix Web + WebSocket, Postgres (SQLx) và Redis.

## 1. Tech stack

- Rust 2024
- Actix Web, Actix WS
- SQLx + PostgreSQL
- Redis (cache + presence)
- Tracing + metrics export theo format Prometheus

## 2. Yêu cầu môi trường

- Rust stable
- PostgreSQL đang chạy
- Redis đang chạy
- File `.env` tại thư mục `backend/`

Các biến môi trường chính:

- `SECRET_KEY`
- `DATABASE_URL`
- `REDIS_URL`
- `FRONTEND_URL` (mặc định: `http://localhost:5173`)
- `IP` (mặc định: `127.0.0.1`)
- `PORT` (mặc định: `8080`)
- `ACCESS_TOKEN_EXPIRATION` (mặc định: `900`)
- `REFRESH_TOKEN_EXPIRATION` (mặc định: `604800`)
- `APP_ENV`
- `COOKIE_SECURE` (optional: `1|true|yes|0|false|no`)
- `CLOUDINARY_URL` (optional)

## 3. Chạy dự án

```bash
cargo check
cargo run
```

Mặc định server chạy tại `http://127.0.0.1:8080`.

## 4. Chất lượng mã và test

```bash
cargo clippy -- -D warnings
cargo test --no-fail-fast
```

Test runbook nhanh theo phạm vi thay đổi:

```bash
# Module-focused
cargo test friend_test
cargo test message_test
cargo test call_test
cargo test ws_server_test

# Full sanity
cargo test
```

Trạng thái test hiện tại:

- Unit tests: pass
- Integration tests conversation: `ignored` (cần DB fixture và schema đã migrate)
- `group_management_test`: `ignored` (cần Postgres + seed fixture phù hợp)

Ghi chú module-level sau refactor: xem `docs/backend-module-notes.md`.

## 5. API và realtime

### Endpoint hệ thống

- `GET /` health check
- `GET /metrics` Prometheus text exposition
- `GET /metrics/json` JSON snapshot cho debug nội bộ
- `GET /ws` WebSocket endpoint

### Endpoint business (prefix `/api`)

- Auth/User: đăng ký, đăng nhập, refresh token, profile, search users
- Friend: gửi/duyệt/từ chối request, danh sách bạn bè
- Conversation: tạo conversation, lấy danh sách, lấy messages, mark as seen
- Message: direct/group send, edit, delete
- File upload: upload/get/delete

### Upload

- Local file phục vụ qua `GET /uploads/<filename>`
- Nếu có `CLOUDINARY_URL`, upload sẽ ưu tiên cloud storage

## 6. Observability

### Request context

- Middleware tự sinh hoặc nhận `x-request-id`
- Response trả lại `x-request-id`
- Log HTTP/WS đồng bộ theo request id

### Metrics chính

- `http_requests_total`
- `ws_reconnect_total`
- `ws_disconnect_total`
- `ws_close_total{reason=...}` theo vòng đời close
- `app_message_send_latency_ms_*` (histogram)
- `message_send_p50_ms`, `message_send_p95_ms`, `message_send_p99_ms` (json snapshot)
- `upload_attempt_total`, `upload_failure_total`, `upload_failure_rate`

Lưu ý: metrics hiện lưu in-memory, sẽ reset khi restart process.

Runbook vận hành observability: xem `../docs/observability-runbook.md`.
Guideline hiệu năng: xem `../docs/performance-guidelines.md`.
Baseline benchmark WebSocket map concurrency: `../docs/websocket-benchmark-baseline.md`.

### Performance baseline smoke

Chạy baseline smoke cho các flow cốt lõi (message/call/ws + full suite):

```bash
bash scripts/perf_smoke.sh
```

Tùy chọn số lần lặp mỗi case:

```bash
PERF_ITERATIONS=5 bash scripts/perf_smoke.sh
```

Kết quả mặc định được ghi vào `docs/perf-baseline-latest.md` để so sánh trước/sau refactor.

## 7. Chuẩn hóa error contract

- Error response đã có `code` theo nhóm (`bad_request`, `forbidden`, `not_found`, ...)
- Message dùng catalog tập trung để giảm hardcode phân tán
- Mục tiêu là giữ wording ổn định cho frontend mapping

## 8. Kiến trúc module

- `src/main.rs`: bootstrap app + route wiring
- `src/middlewares/`: auth, authorization, request context
- `src/modules/`: user/friend/conversation/message/file_upload/websocket
- `src/observability/`: metrics + request context
- `src/tests/`: unit test và integration test

Snapshot boundary sau refactor:
- `message`/`conversation`/`call`: service policy helpers tách khỏi orchestration chính.
- `websocket`: session/room/broadcast helpers tách theo lifecycle rõ ràng.
- API handlers: chuẩn hóa flow parse/validate/map trước khi gọi service.

## 9. Chuẩn bị cho frontend

Checklist tích hợp frontend:

- Đồng bộ base URL API: `http://<IP>:<PORT>/api`
- Đăng nhập: lưu `access_token` từ body, `refresh_token` qua cookie httpOnly
- Refresh flow: gọi endpoint refresh khi token hết hạn
- Đồng bộ xử lý lỗi theo `error.code` trước, `message` sau
- Kết nối WebSocket qua `/ws`, gửi event auth ngay sau khi connect
- Theo dõi metrics qua Prometheus scrape từ `/metrics`

Khuyến nghị trước khi tích hợp frontend:

- Chốt contract payload cho các event WS đang dùng ở UI
- Thiết lập env frontend (`VITE_API_URL`, `VITE_WS_URL`) theo backend runtime
- Viết smoke test E2E tối thiểu: sign in -> send message -> realtime receive
