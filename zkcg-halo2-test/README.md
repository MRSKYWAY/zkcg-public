# zkcg-halo2-test

Reusable security-testing infrastructure for Halo2-based proof systems.

`zkcg-halo2-test` gives you small, composable harnesses for testing the *security properties around* a Halo2 verifier: adversarial statements, semantic mutations, malformed proof bytes, replay/context binding, differential verification, and controlled circuit-soundness mutations.

> **Important:** this is a testing framework, not a proof of security. A passing suite establishes only the properties represented by the cases and independent oracles that you actually exercised.

## Install

```toml
[dev-dependencies]
zkcg-halo2-test = "0.1.1"
```

For the optional Halo2 feature:

```toml
[dev-dependencies]
zkcg-halo2-test = { version = "0.1.1", features = ["halo2"] }
```

## Basic pattern

Adapt your real verifier to the small `Acceptance` abstraction:

```rust
use zkcg_halo2_test::{Acceptance, AdversarialCase, AdversarialHarness};

fn verify_real(input: &u64) -> Result<Acceptance, String> {
    // Call the real proof parser + Halo2 verification path here.
    Ok(if *input <= 100 { Acceptance::Accepted } else { Acceptance::Rejected })
}

#[test]
fn rejects_invalid_statement() {
    let harness = AdversarialHarness::new(verify_real);
    let result = harness.run(AdversarialCase {
        name: "position limit exceeded".into(),
        input: 101,
        expected: Acceptance::Rejected,
    }).unwrap();
    assert_eq!(result.observed, Acceptance::Rejected);
}
```

The callback should reach the real production parser/verifier path rather than reproduce the same constraints in the test code.

## Which API should I use?

| Question | API |
| --- | --- |
| Should this malicious statement reject? | `AdversarialHarness` |
| Does a changed semantic result reject? | `MutationHarness` |
| Can a proof/context be replayed after a security-relevant change? | `ReplayHarness` |
| Do two backends agree with an independent oracle? | `DifferentialHarness` |
| Does a weakened circuit expose the missing constraint? | `CircuitMutationRunner` |
| Can I generate deterministic malformed proof bytes? | `mutate_proof` + `MalformedProofMutation` |
| Can I keep audit-friendly test evidence? | `EvidenceSummary` |

## Adversarial testing

Use `AdversarialHarness` for explicit security invariants:

```rust
harness.run(AdversarialCase {
    name: "amount exceeds policy bound".into(),
    input: malicious_statement,
    expected: Acceptance::Rejected,
})?;
```

A mismatch produces `SecurityTestError::UnexpectedAcceptance` or `UnexpectedRejection`.

## Semantic mutation

Use `MutationHarness` when the proof is structurally valid but a security-relevant semantic result is deliberately changed. The consumer can also implement `SecurityOracle` to derive expected semantics independently from the production evaluator.

## Replay and context binding

`ReplayHarness` asserts:

```text
original proof/context + original statement -> ACCEPT
same proof/context + mutated security input -> REJECT
```

Use this for identity, domain separation, commitments, workflow selectors, chain/context identifiers, and other public-input binding properties.

## Malformed proofs

`mutate_proof` generates deterministic byte mutations. The mutation helper does not claim the result must be rejected; always route mutated bytes through the real parser/verifier.

```rust
let malformed = mutate_proof(
    &valid_proof,
    MalformedProofMutation::Append(vec![0x00]),
);
assert!(real_verify_bytes(&malformed).is_err());
```

Built-in mutations include empty, truncation, bit flips, byte replacement, and appended bytes. Include trailing-byte cases when your proof format requires strict end-of-input handling.

## Differential verification

`DifferentialHarness` compares backend A, backend B, and an independent oracle. It is especially useful when two proving systems or verification implementations exist.

```text
backend A -> decision
backend B -> decision
oracle     -> decision
       all three must agree
```

Agreement is evidence for the tested vector; it does not establish that the oracle itself is correct.

## Circuit-soundness mutation

`CircuitMutationRunner` is for controlled mutation benchmarks:

```text
intact circuit + malicious statement -> REJECT
mutated circuit + same statement     -> ACCEPT
```

The consumer owns the actual Halo2 circuit mutations. This is a sensitivity benchmark for representative missing constraints, not exhaustive circuit-soundness coverage.

## Evidence reporting

`EvidenceSummary` records normalized results without imposing a particular report or CI format:

```rust
let mut evidence = EvidenceSummary::new();
evidence.record(EvidenceRecord {
    phase: "phase-3",
    category: "malformed-proof",
    test_name: "trailing-byte".into(),
    outcome: "pass",
});
assert!(evidence.is_clean());
```

## Recommended repository layout

```text
crate/tests/
├── adversarial.rs
├── semantic.rs
├── malformed.rs
├── replay.rs
├── differential.rs
└── circuit_mutation.rs
```

Keep application-specific policy, circuits, and proof adapters in the consuming repository; use this crate as the reusable harness/assertion layer.

## Security boundary

A green suite is not a global security certificate. In particular, this crate does not automatically establish complete Halo2 circuit soundness, discover arbitrary underconstraints, validate an independent oracle, or guarantee rejection of every malformed proof encoding.

## Development

```bash
cargo test
cargo test --doc
cargo test --features halo2
cargo fmt -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo package --allow-dirty
cargo publish --dry-run
```

See `docs/USAGE.md` for the longer integration guide.
