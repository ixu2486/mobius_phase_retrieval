# Benchmark Results

Checkpoint: `mobius_phase_retrieval_sdk_benchmark_v1`

## Reproducible Commands
Run from `E:\0421\retryix_rs\sdk\mobius_phase_retrieval`:

```powershell
cargo build
cargo test
cargo run --example basic_usage
cargo run --example json_retrieval_demo -- examples/json_retrieval_demo_input.json
```

## Notes
- The standalone SDK intentionally focuses on application-layer retrieval flow.
- Extended compare benchmarks remain available in the main RetryIX workspace (`retryix_memory` examples).

## Example Snapshot (basic_usage)
- selected_profile: `Balanced`
- adaptive_reason: `DefaultBalanced`
- sample ranked hits emitted to stdout
Interpretation:
- Standalone crate is independently buildable/runnable
- Public boundary remains application-layer only
