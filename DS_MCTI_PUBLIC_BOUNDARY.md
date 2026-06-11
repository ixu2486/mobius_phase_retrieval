# DS-MCTI Public Boundary Claim

This repository exposes a limited public demonstration boundary for **DS-MCTI v0**.

The public communication demo is provided for reproducibility, research discussion, and boundary clarification. It does **not** disclose, license, or waive rights to the full DS-MCTI closure-generation theory or RetryIX patent-sensitive core technology.

---

## Publicly Demonstrated Seven-Layer Closure Route

The following seven-layer path-dependent closure route was originally identified, implemented, and publicly demonstrated by **Ice Xu / RetryIX**:

```text
1/7 -> 1/17 -> 1/19 -> 1/23 -> 1/29 -> 1/47 -> 1/58 => 1/7
```

This route is used in the public communication demo as a reproducible closure behavior.

---

## Public Demo Boundary

The public demo exposes only:

- The observable communication behavior
- The seven-layer ordered route used by Device A
- The normalized return behavior used by Device B
- The `CHAIN_CLOSURE_PASS` verification output
- A minimal reproducible Python implementation for research discussion

The public demo does **not** disclose or grant rights to the full DS-MCTI closure-generation theory.

---

## A/B Communication Boundary

In the public communication model:

- Device A carries the full seven-layer route.
- Device B exposes only the return-gate behavior:

```text
1/58 => 1/7
```

- The seventh layer `1/58` acts as a return-normalization gate / Möbius topological threshold.
- The returned `1/7` is accepted only as a verified normalized return from the peer.

This is not a simple echo, replay, or two-point mapping.

---

## Reserved Rights

All rights are reserved by **Ice Xu / RetryIX** for:

- DS-MCTI closure-generation theory
- Seven-layer route derivation methods
- Alternative closure-chain generation methods
- Möbius complement phase-space construction
- Context-coordinate acquisition methods derived from DS-MCTI
- Semantic primitive correctness verification derived from DS-MCTI
- Root verification methods for semantic reconstruction systems
- Low-frequency, magnetic-field, quantum, physical-transport, PIM, and hardware execution mappings
- Commercial deployment or integration of DS-MCTI-derived closure verification
- Patent-sensitive RetryIX core technology

---

## Research Permission

Personal, academic, educational, and non-commercial research use of the public demo is allowed under the repository license terms.

Commercial use, platform integration, productization, hosted services, enterprise deployment, derivative closure-verification systems, or integration into proprietary systems requires a separate written commercial license from RetryIX.

---

## Boundary Statement

This demo exposes the closure behavior and reproducible communication boundary of DS-MCTI v0.

It does not disclose, license, or waive rights to the full closure-generation theory, private RetryIX runtime internals, patent-sensitive core technology, or future DS-MCTI-derived applications.

For DS-MCTI public boundary and seven-layer closure-route reservation, see:

`../DS_MCTI_PUBLIC_BOUNDARY.md`
