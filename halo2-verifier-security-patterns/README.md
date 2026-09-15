# Halo2 Verifier Security Patterns

Reusable security-test patterns extracted from the ZKCG security evaluation phases.

This directory is a test methodology library, not a claim that a verifier is secure merely because these patterns exist. Each pattern is intended to be adapted to the concrete proof system, public-input encoding, transcript, and backend under test.

## Patterns

1. **Public-input binding** — prove and verify that every security-relevant expected value is cryptographically bound to the proof's public inputs.
2. **Constraint-completeness adversarial tests** — construct facts that violate a numeric or semantic invariant while independently controlling any normalized flags used by the circuit.
3. **Independent semantic oracle** — derive expected decisions and reason bits outside the production evaluator and compare production behavior against that oracle.
4. **Mutation testing** — deliberately corrupt security-relevant expected values and require the verifier to reject them.
5. **Circuit-soundness mutation** — deliberately remove copy/equality/boolean bindings or selector enforcement from a controlled Halo2 circuit and require the intact circuit to reject the same malicious statement. Phase 8 validates five representative mutation classes end-to-end.
6. **Malformed-proof corpus** — mutate, truncate, append to, and otherwise corrupt serialized proof bytes; require rejection, including strict end-of-input handling where the format requires it.
7. **Binding / replay tests** — reuse proofs with altered public inputs, identities, workflow selectors, or commitments and require rejection.
8. **Fuzzing** — continuously exercise proof bytes, public inputs, and proof dispatch with malformed/randomized inputs. Fuzzing is crash/robustness evidence, not a proof of semantic completeness.
9. **Cross-backend differential testing** — compare independent proving/verification backends against the same independent oracle, including a deliberate divergence canary.

## How to use this library

For a new verifier integration, start with the checklist in `PATTERN_MATRIX.md`, then instantiate the templates under `templates/`. Record any discovered issue separately from the pattern that detected it.

For circuit-soundness work, use the mutation-benchmark reference in the consuming project. The benchmark validates sensitivity to representative underconstraint classes; it does not replace formal verification, static analysis, or expert review.

The patterns should be used together: no single pattern establishes complete circuit correctness.
