# Performance Baseline (Smoke)

- Generated at: `2026-03-23T04:19:15`
- Iterations per case: `1`
- Unit: milliseconds (`ms`)

| Case | Runs | Avg | P50 | P95 | Min | Max |
|---|---:|---:|---:|---:|---:|---:|
| `message_send_guard` | 1 | 1581 | 1581 | 1581 | 1581 | 1581 |
| `call_accept_flow` | 1 | 873 | 873 | 873 | 873 | 873 |
| `ws_fanout_flow` | 1 | 1127 | 1127 | 1127 | 1127 | 1127 |
| `full_test_suite` | 1 | 2764 | 2764 | 2764 | 2764 | 2764 |

## Commands

- `cargo test tests::message_test::tests::test_send_message_to_conversation_rejects_non_member -- --exact`
- `cargo test tests::call_test::tests::respond_call_accept_updates_status_and_emits_event -- --exact`
- `cargo test tests::ws_server_test::test_send_to_user_all_sessions_receive_once -- --exact`
- `cargo test`

## Notes

- Đây là smoke baseline để so sánh tương đối giữa các lần refactor.
- Không dùng làm SLA tuyệt đối vì phụ thuộc môi trường local/CI.
