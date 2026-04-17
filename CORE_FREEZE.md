# ZKCG Core Freeze — v0.2.0

This document defines the frozen invariants of the ZKCG protocol core.

## Frozen Components

- `PublicInputs` / `ProofClaims` typed-claims schema
- Universal `Proof` + `ProofSystem` abstraction
- `ProofVerifier` trait and `Verifier` entrypoint
- State transition semantics
- Halo2 circuit constraints
- zkVM guest logic and commit order
- zkVM receipt + journal binding
- Default verifier registry behavior

## Invariants

1. A proof is valid iff:
   - the verifier adapter selected by `Proof.system` accepts the proof
   - the workflow-specific decision commitment is reproduced from the claimed facts
   - public inputs match the committed values
   - backend cryptography verifies
   - zkVM receipts verify against the expected image ID and committed journal

Current note on scope:
- the frozen public phase-1 Halo2 scope includes `score <= threshold`
- the frozen public RWA v1 scope includes `rwa.credit.onboarding.v1` and `rwa.credit.transfer.v1`
- the frozen RWA v1 proof boundary is deterministic evaluation over normalized facts, not raw upstream evidence truth or in-proof list resolution

2. Backends must be observationally equivalent:
   - Halo2
   - zkVM

3. Batch verification preserves request order in its reported results.

4. State transitions are deterministic:
   - nonce monotonic
   - state_root binding enforced outside proof

## Non-Goals

- Generic proving SDK
- Production-ready CLI
- Performance guarantees
- Bundled Groth16/STARK/WASM implementations

## Change Policy

Breaking changes are not allowed without:
- Version bump
- New freeze document

Status: FROZEN
Version: v0.2.0
