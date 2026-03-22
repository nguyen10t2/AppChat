# Performance Guidelines (Backend)

## Mục tiêu
- Chọn đúng primitive đồng thời cho đúng loại workload.
- Giảm risk regression hiệu năng khi refactor các path realtime và auth.

## DashMap vs Mutex<HashMap>

### Ưu tiên DashMap khi
- Shared state có read concurrency cao (session/user/room presence map).
- Cần giảm contention cho nhiều request/connection đồng thời.
- Dữ liệu được truy cập theo key độc lập.

### Ưu tiên Mutex<HashMap> khi
- Contention thấp, truy cập ít cạnh tranh.
- Cần cấu trúc đơn giản và tiết kiệm memory hơn.
- Cần lock toàn map cho thao tác bulk ngắn.

## Tokio vs Rayon

### Dùng tokio::task::spawn_blocking cho
- CPU-bound work nằm trong async request path (ví dụ hash/verify password).
- Blocking libraries cần offload khỏi async executor threads.

### Dùng rayon cho
- Batch CPU-bound thuần compute, không phụ thuộc async runtime.
- Data-parallel operations chạy độc lập, yêu cầu throughput cao.

### Tránh
- Dùng thread spawn thủ công + channel trong async path nếu `spawn_blocking` đã đủ.

## WebSocket broadcast strategy
- Tập recipients nhỏ: ưu tiên sequential fan-out để giảm scheduling overhead.
- Tập recipients lớn: dùng parallel fan-out để giữ throughput.
- Tối ưu dựa trên benchmark thay vì giả định.

## Baseline & Validation
- Smoke baseline: `PERF_ITERATIONS=3 bash backend/scripts/perf_smoke.sh`
- Concurrency benchmark: `cargo bench --bench websocket_benchmark -- --sample-size 20 --measurement-time 2`
- Baseline WebSocket hiện tại: `docs/websocket-benchmark-baseline.md`

## PR checklist (performance-sensitive)
- Có số liệu trước/sau ở cùng môi trường chạy.
- Có đánh giá tác động p95 (ít nhất smoke baseline).
- Không tăng cardinality metrics không cần thiết.
- Cập nhật runbook/changelog khi thêm hoặc đổi strategy.
