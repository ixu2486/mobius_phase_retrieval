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
cargo run --example json_retrieval_demo -- examples/json_retrieval_demo_input.json
```

The private RetryIX runtime, MemoryKernel scheduling logic, hardware-routing logic, and PIM-related execution paths are not included.

## What This SDK Does

- Provides application-layer phase-aware retrieval
- Provides visibility + importance classification language
- Provides hybrid retrieval: fast candidate recall + precision rerank
- Provides adaptive profile selection and retrieval metrics
- Provides reproducible build, test, and demo commands

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

Run from this repository root:

```bash
cargo build
cargo test
cargo run --example basic_usage
cargo run --example json_retrieval_demo -- examples/json_retrieval_demo_input.json
```

Expected behavior:

- `cargo build` compiles the standalone public crate.
- `cargo test` runs the SDK unit tests.
- `basic_usage` runs a deterministic retrieval demo.
- `json_retrieval_demo` loads the sample JSON input and produces retrieval output.

## Files

- `Cargo.toml`
- `src/lib.rs`
- `examples/basic_usage.rs`
- `examples/json_retrieval_demo.rs`
- `examples/json_retrieval_demo_input.json`
- `examples/json_retrieval_demo_output.example.json`
- `docs/API_OVERVIEW.md`
- `docs/ARCHITECTURE_BOUNDARY.md`
- `benchmarks/BENCHMARK_RESULTS.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `LICENSE`
- `LICENSE_SUMMARY.md`

## Author

- Ice Xu
- Contact: `ice____@msn.com`
