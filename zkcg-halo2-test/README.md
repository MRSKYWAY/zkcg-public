# zkcg-halo2-test

Reusable security-testing infrastructure for Halo2-based proof systems.

This crate grew out of the ZKCG security evaluation and extracts the reusable parts of Phases 1–8 into generic APIs. ZKCG-specific facts, policy objects, circuits, and decisions remain in the consuming project.

## What it covers

| Evaluation phase | Reusable crate capability |
|---|---|
| Phase 1 | adversarial constraint/invariant harness |
| Phase 2 | semantic mutation harness + independent oracle interface |
| Phase 3 | malformed-proof mutation helpers + binding/replay harness |
| Phase 4 | fuzzing templates and proof-byte mutation primitives |
| Phase 5 | differential backend harness with an independent oracle |
| Phase 6 | reusable security-pattern APIs and matrices |
| Phase 7 | evidence records and documentation model |
| Phase 8 | circuit-soundness mutation runner for controlled Halo2 benchmarks |

## Security model

The crate is a testing framework, not a proof of security. A green run establishes only the properties exercised by the supplied cases and oracles.

In particular, it does not claim:

- complete Halo2 circuit soundness;
- automatic discovery of arbitrary circuit underconstraints;
- correctness of an external policy oracle merely because two implementations agree;
- protection against every malformed proof encoding;
- reproduction or detection of any particular real-world vulnerability.

## Core patterns

### Adversarial testing

Construct a security-relevant input with an explicit expected verifier outcome and run it against the real verifier path.

### Semantic mutation

Deliberately alter expected decisions, classifications, reason bits, commitments, or related public values and require rejection. The consumer supplies an independent `SecurityOracle` implementation.

### Malformed proofs

Use deterministic mutations such as truncation, bit flips, replacement, and trailing-byte append operations to build a reproducible negative corpus.

### Binding and replay

Verify an original proof/context succeeds and then require the same proof or context with an altered security-relevant input to reject.

### Differential verification

Run identical inputs through two verification backends and an independent oracle. Any disagreement is a test failure.

### Circuit-soundness mutation

For each controlled circuit mutation, require:

```text
intact circuit + malicious statement  -> REJECT
mutated circuit + same statement      -> ACCEPT
```

The actual Halo2 circuit and proving/verifying code remain owned by the consuming project; this crate provides the reusable assertion harness.

## Example

```rust
use zkcg_halo2_test::{Acceptance, CircuitMutationCase, CircuitMutationRunner};

fn main() -> Result<(), ()> {
    let runner = CircuitMutationRunner::new(|input: &u64, mutated: bool| {
        let accepted = if mutated { *input == 42 } else { *input == 7 };
        Ok::<_, ()>(if accepted { Acceptance::Accepted } else { Acceptance::Rejected })
    });

    let result = runner.run(CircuitMutationCase {
        name: "missing output binding".into(),
        malicious_input: 42,
    })?;

    assert!(result.intact_rejected);
    assert!(result.mutated_accepted);
    Ok(())
}
```

## Reference implementation

ZKCG is the first reference consumer. Its security evaluation retains application-specific test vectors while using this crate for reusable harness logic.

## Development

```bash
cargo test -p zkcg-halo2-test
cargo test -p zkcg-halo2-test --features halo2
cargo fmt --all -- --check
cargo clippy -p zkcg-halo2-test --all-features --all-targets -- -D warnings
cargo package -p zkcg-halo2-test --allow-dirty
```

Before the first crates.io release, run `cargo publish --dry-run -p zkcg-halo2-test` from a clean tree.
