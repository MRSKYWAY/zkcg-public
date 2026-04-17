# Changelog

## v0.2.0 - 2026-03-13

- add a universal `Proof` and `ProofSystem` abstraction across the verifier crate
- introduce adapter-based dispatch for Halo2 and zkVM verification
- add sequential batch verification, parallel batch verification, and per-proof batch results
- expose a public `VerifierRegistry` for custom proof-system registration
- harden zkVM verification by validating full receipts and committed journals
- improve zkVM prover error classification between policy and execution failures
- update the HTTP API to accept `system` and support batch verification endpoints
- refresh tests, benches, examples, README, `SPEC.md`, and `CORE_FREEZE.md` for v0.2.0
