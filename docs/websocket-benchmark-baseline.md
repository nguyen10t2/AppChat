# WebSocket Benchmark Baseline

Generated: 2026-03-23
Command:

```bash
cargo bench --bench websocket_benchmark -- --sample-size 20 --measurement-time 2
```

## Parallel Reads

| Users | DashMap (median) | Mutex<HashMap> (median) | Faster |
|---:|---:|---:|---:|
| 10 | 256.02 µs | 778.04 µs | ~3.04x |
| 100 | 275.94 µs | 967.14 µs | ~3.50x |
| 500 | 253.32 µs | 1.2138 ms | ~4.79x |

## Parallel Writes

| Users | DashMap (median) | Mutex<HashMap> (median) | Faster |
|---:|---:|---:|---:|
| 10 | 278.32 µs | 670.97 µs | ~2.41x |
| 100 | 272.95 µs | 890.91 µs | ~3.26x |
| 500 | 335.61 µs | 921.00 µs | ~2.74x |

## Summary
- DashMap consistently outperforms Mutex<HashMap> under concurrent read/write workloads.
- The gap grows on higher-read scenarios and larger key spaces.
- Current decision remains valid: keep DashMap for WebSocket shared state.

## Notes
- This is a local-machine baseline for relative comparison.
- Use the same benchmark command and machine profile for before/after comparisons.
