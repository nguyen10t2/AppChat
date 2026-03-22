#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/backend"
OUT_FILE="${1:-${ROOT_DIR}/docs/perf-baseline-latest.md}"
ITERATIONS="${PERF_ITERATIONS:-3}"

if ! [[ "${ITERATIONS}" =~ ^[1-9][0-9]*$ ]]; then
  echo "PERF_ITERATIONS must be a positive integer" >&2
  exit 1
fi

TMP_CSV="$(mktemp)"
trap 'rm -f "${TMP_CSV}"' EXIT

declare -a CASE_LABELS=(
  "message_send_guard"
  "call_accept_flow"
  "ws_fanout_flow"
  "full_test_suite"
)

declare -a CASE_COMMANDS=(
  "cargo test tests::message_test::tests::test_send_message_to_conversation_rejects_non_member -- --exact"
  "cargo test tests::call_test::tests::respond_call_accept_updates_status_and_emits_event -- --exact"
  "cargo test tests::ws_server_test::test_send_to_user_all_sessions_receive_once -- --exact"
  "cargo test"
)

printf "case,run,ms\n" > "${TMP_CSV}"

for index in "${!CASE_LABELS[@]}"; do
  label="${CASE_LABELS[${index}]}"
  command="${CASE_COMMANDS[${index}]}"

  for run in $(seq 1 "${ITERATIONS}"); do
    start_ns="$(date +%s%N)"
    (
      cd "${BACKEND_DIR}"
      bash -lc "${command}" >/dev/null
    )
    end_ns="$(date +%s%N)"
    elapsed_ms="$(((end_ns - start_ns) / 1000000))"
    printf "%s,%s,%s\n" "${label}" "${run}" "${elapsed_ms}" >> "${TMP_CSV}"
    echo "${label} run ${run}/${ITERATIONS}: ${elapsed_ms} ms"
  done

done

python3 - "${TMP_CSV}" "${OUT_FILE}" "${ITERATIONS}" <<'PY'
import csv
import datetime
import statistics
import sys
from collections import defaultdict

csv_path, out_path, iterations = sys.argv[1], sys.argv[2], int(sys.argv[3])
rows = defaultdict(list)

with open(csv_path, newline="", encoding="utf-8") as handle:
    reader = csv.DictReader(handle)
    for row in reader:
        rows[row["case"]].append(int(row["ms"]))

case_order = [
    "message_send_guard",
    "call_accept_flow",
    "ws_fanout_flow",
    "full_test_suite",
]

now = datetime.datetime.now().isoformat(timespec="seconds")


def percentile(values, p):
    if not values:
        return 0
    ordered = sorted(values)
    position = int(round((len(ordered) - 1) * p))
    return ordered[position]

with open(out_path, "w", encoding="utf-8") as output:
    output.write("# Performance Baseline (Smoke)\n\n")
    output.write(f"- Generated at: `{now}`\n")
    output.write(f"- Iterations per case: `{iterations}`\n")
    output.write("- Unit: milliseconds (`ms`)\n\n")
    output.write("| Case | Runs | Avg | P50 | P95 | Min | Max |\n")
    output.write("|---|---:|---:|---:|---:|---:|---:|\n")

    for case in case_order:
        values = rows.get(case, [])
        if not values:
            continue

        avg = round(statistics.mean(values), 2)
        p50 = percentile(values, 0.50)
        p95 = percentile(values, 0.95)
        min_value = min(values)
        max_value = max(values)

        output.write(
            f"| `{case}` | {len(values)} | {avg} | {p50} | {p95} | {min_value} | {max_value} |\n"
        )

    output.write("\n## Commands\n\n")
    output.write("- `cargo test tests::message_test::tests::test_send_message_to_conversation_rejects_non_member -- --exact`\n")
    output.write("- `cargo test tests::call_test::tests::respond_call_accept_updates_status_and_emits_event -- --exact`\n")
    output.write("- `cargo test tests::ws_server_test::test_send_to_user_all_sessions_receive_once -- --exact`\n")
    output.write("- `cargo test`\n")
    output.write("\n## Notes\n\n")
    output.write("- Đây là smoke baseline để so sánh tương đối giữa các lần refactor.\n")
    output.write("- Không dùng làm SLA tuyệt đối vì phụ thuộc môi trường local/CI.\n")

print(f"Wrote baseline report to {out_path}")
PY

echo "Done: ${OUT_FILE}"