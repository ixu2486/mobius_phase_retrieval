# Möbius Phase Retrieval SDK

Möbius Phase Retrieval SDK helps AI and retrieval systems decide what to remember, what to hide, what to recall, what to archive, and what can be safely forgotten.

中文：
Möbius Phase Retrieval SDK 協助 AI 與檢索系統判斷什麼該記住、什麼可暫藏、什麼需要召回、什麼可歸檔、什麼可以安全遺忘。

## Standalone Build

This public SDK is now a standalone Rust crate and does not depend on private RetryIX path crates.

```bash
cargo build
cargo test
cargo run --example basic_usage
```

## What This SDK Does
- Provides application-layer phase-aware retrieval
- Provides visibility + importance classification language
- Provides hybrid retrieval (fast candidate recall + precision rerank)
- Provides adaptive profile selection and retrieval metrics
- Provides reproducible benchmark/demo commands

## What This SDK Does Not Do

This public SDK is limited to the application-layer retrieval and semantic indexing interface.

It does not expose:

- RetryIX private runtime internals
- MemoryKernel internal scheduling details
- TurboQuant-compatible backend internals
- private hardware-interface contracts
- device-level routing or execution paths
- physical transport, specialized hardware execution, or hardware PIM logic

## Main Use Cases
1. AI long-context memory recall
2. Data-center semantic retrieval classification
3. Latent / archived / cold memory governance

## Core Concepts
- Full-reptend phase coordinate
- Möbius half-turn latent pairing
- Visibility class
- Importance class
- Hybrid retrieval
- Adaptive profile
- Metrics

## Novelty Boundary
This SDK does not claim to invent retrieval, reranking, semantic indexing, or memory tiering from scratch.

Its contribution is the application of full-reptend prime phase coordinates and Möbius half-turn latent pairing as a practical indexing schema for memory governance, latent-state recovery, and phase-aware retrieval.

## Public Boundary Claim
This SDK is an application-layer retrieval and memory-governance SDK.
It does not implement physical transport, quantum transport, hardware PIM execution, or private RetryIX runtime internals.

## Minimal Reproducible Commands
Run from repo root `E:\0421\retryix_rs`:

```powershell
cargo check -p retryix_memory
cargo test -p retryix_memory mobius_phase_retrieval -- --nocapture
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 5000 40 40
```

Additional benchmark commands:

```powershell
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 20000 200
cargo run -p retryix_memory --release --example mobius_phase_retrieval_compare -- 50000 500 150
```

SDK usage demos:

```powershell
cargo run -p retryix_memory --release --example mobius_phase_sdk_basic_usage
cargo run -p retryix_memory --release --example mobius_phase_sdk_json_retrieval_demo -- sdk/mobius_phase_retrieval/examples/json_retrieval_demo_input.json
```

## Files
- `examples/basic_usage.rs`
- `examples/json_retrieval_demo.rs`
- `examples/json_retrieval_demo_input.json`
- `examples/json_retrieval_demo_output.example.json`
- `docs/API_OVERVIEW.md`
- `docs/ARCHITECTURE_BOUNDARY.md`
- `benchmarks/BENCHMARK_RESULTS.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `LICENSE_SUMMARY.md`

## Author
- Ice Xu
- Contact: `ice____@msn.com`
