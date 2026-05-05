# Architecture Boundary

Checkpoint: `mobius_phase_retrieval_sdk_boundary_v1`

## Public (Included in This SDK)
- Phase-aware retrieval
- Visibility + importance classification
- Adaptive profile selection
- Retrieval metrics
- Public benchmark/demo examples

## Private / Not Included
- RetryIX MemoryKernel internal routing
- VirtualPIM resident execution path
- TurboQuant-compatible backend internals
- PIM opcode mapping
- SPD/EEPROM physical contract logic
- Hardware-specific scheduling logic
- Physical magnetic / quantum transport mechanisms
- Private execution paths and hardware route internals

## Boundary Statement
This package is intentionally limited to application-layer retrieval and memory-governance APIs.
It is not a hardware runtime package and must not be interpreted as exposing private RetryIX execution infrastructure.
This public crate is standalone and does not require private RetryIX source paths.
