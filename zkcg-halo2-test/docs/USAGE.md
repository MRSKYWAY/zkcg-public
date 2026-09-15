# Usage guide

## 1. Add the test dependency

```toml
[dev-dependencies]
zkcg-halo2-test = "0.1.1"
```

Keep the crate in test/dev dependencies where possible; it is intended to provide security-testing harnesses rather than production verification logic.

## 2. Adapt the real verifier

Build one small adapter that calls the production parser and verifier and maps its security-relevant result to `Acceptance`.

```text
security test -> adapter -> real parser/transcript/public-input checks -> real Halo2 verifier
```

Do not duplicate circuit constraints in the adapter. Otherwise the test may only prove that two copies of the same bug agree.

## 3. Build cases from the threat model

Useful negative cases include range violations, forged decision bits, changed commitments, identity changes, domain/context changes, malformed proof bytes, and proof replay under different public inputs.

Give cases stable names so CI failures and audit evidence remain actionable.

## 4. Use independent semantics

For policy-heavy circuits, implement `SecurityOracle` separately from the production evaluator. Then mutate the expected semantic result and require the real verifier to reject it.

## 5. Test binding separately

For each security-relevant context, explicitly test:

```text
original context -> ACCEPT
mutated context  -> REJECT
```

This should be repeated for identity, domain separation, commitments, application/chain selectors, and other security-critical public inputs as applicable.

## 6. Keep malformed-proof tests strict

Treat byte mutation and verification as separate layers:

```text
valid proof -> deterministic mutation -> real parser/verifier -> expected REJECT
```

Include truncation, bit flips, replacements, empty input, and trailing-byte cases where appropriate. Unexpected bytes after an otherwise valid proof are an important parser-boundary test.

## 7. Differential testing

Use two genuinely independent verification paths plus an oracle. Include positive, negative, and boundary cases. A mismatch is evidence to investigate; the harness deliberately does not claim which backend is correct.

## 8. Circuit mutation benchmarks

Create deliberate mutations such as missing equality/copy/boolean constraints or selector omissions and check that the intact circuit rejects the malicious statement while the weakened variant accepts it. This demonstrates sensitivity to the tested mutation class; it is not exhaustive soundness coverage.

## 9. Evidence and CI

Record important cases with `EvidenceRecord` and aggregate them with `EvidenceSummary`.

```bash
cargo test
cargo test --doc
cargo test --features halo2
cargo fmt -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo package --allow-dirty
```

For a release candidate:

```bash
cargo publish --dry-run
cargo publish
```

## 10. Interpreting a green run

A passing suite establishes only the properties represented by the executed corpus and oracle. Continue expanding the corpus from the threat model and record known limitations explicitly.
