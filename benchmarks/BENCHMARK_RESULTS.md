# Benchmark Results

Checkpoint: `mobius_phase_retrieval_sdk_benchmark_v1`

## Reproducible Commands
Run from `E:\0421\retryix_rs`:

```powershell
cargo test -p retryix_memory mobius_phase_retrieval -- --nocapture
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 5000 40 40
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 20000 200
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 50000 500 150
```

## Notes
- `mobius_phase_retrieval_compare` includes:
  - active fast path
  - hybrid path
  - adaptive policy path
  - full_reptend precision path
  - fast_k sweep
  - adaptive quality breakdown (`v0.4.1`)

## Example Snapshot (5000 / 40 / 40)
- active top1: `0.300`
- hybrid top1: `0.875`
- adaptive top1: `0.825`
- full_reptend top1: `1.000`

Interpretation:
- Fast path minimizes latency
- Hybrid substantially improves top1 with moderate overhead
- Adaptive policy provides decision transparency + breakdown metrics

