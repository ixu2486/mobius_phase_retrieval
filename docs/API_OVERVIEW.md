# API Overview

Target module:
`crates/retryix_memory/src/mobius_phase_retrieval/mod.rs`

## Core Public Types
- `MobiusPhaseRetrieval`
- `HybridPhaseRetrieval`
- `RetrievalQuery`
- `RetrievalHit`
- `VisibilityClass`
- `PhaseImportanceClass`
- `HybridRetrievalProfile`
- `AdaptiveRetrievalDecision`
- `AdaptiveRetrievalReason`
- `PhaseRetrievalMetrics`

## Typical Flow
1. Add records (`global_t`, `semantic_anchor`, `visibility`, `importance`, `payload_ref`, `corruption_score`)
2. Build query (`top_k`, `phase_period_mode`, `attention_points`, `semantic_anchor`)
3. Run `adaptive_retrieve_with_decision(...)` or `adaptive_retrieve(...)`
4. Consume ranked hits and scores

## Profile Modes
- `LowLatency`  => `(fast_k=32, final_k=8)`
- `Balanced`    => `(fast_k=128, final_k=16)`
- `HighAccuracy`=> `(fast_k=256, final_k=16)`

## Adaptive Decision Rules (v0.4)
- Small `top_k` + low attention fan-out => `LowLatency`
- Important query hints / multi-point / corruption-sensitive => `HighAccuracy`
- Otherwise => `Balanced`

## Output Signal Examples
- `payload_ref`
- `visibility`
- `importance`
- `latent_recoverable`
- `total_score`

## Boundary Reminder
This API decides recall ranking and memory-governance classes at application layer only.

