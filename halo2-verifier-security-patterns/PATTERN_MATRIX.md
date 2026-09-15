# Security Pattern Matrix

| Pattern | Security question | Minimum negative control | Existing ZKCG evidence |
|---|---|---|---|
| Public-input binding | Can an attacker change a security-relevant expected value without changing the proof statement? | Wrong commitment / changed public input | Phase 2 mutation; Phase 5 canary |
| Constraint completeness | Are semantic limits enforced by constraints rather than only by precomputed booleans? | Violating numeric facts with independently controlled flags | Phase 1 position/concentration tests |
| Independent oracle | Is expected behavior derived independently from the production evaluator? | Production-vs-oracle disagreement | Phase 5 100-vector oracle |
| Mutation testing | Do security-property mutations become rejected proofs? | Flip decision/class/reason bits | Phase 2 mutation matrix |
| Circuit-soundness mutation | Does weakening a Halo2 constraint/binding create an observable accepting proof for a malicious statement? | Missing copy/equality/boolean binding or selector enforcement | Phase 8 five-class end-to-end mutation benchmark |
| Malformed proofs | Does the verifier reject corrupted and non-canonical proof encodings? | Truncation, bit flips, trailing bytes | Phase 3 corpus; trailing-byte finding/remediation |
| Binding / replay | Can a valid proof be replayed under different public inputs or identities? | Alter identity/workflow/public-input fields | Phase 3 |
| Fuzzing | Does malformed input cause crashes, hangs, or parser/backend instability? | Random/mutated proof bytes and inputs | Phase 4 |
| Differential testing | Do independent backends implement the same security semantics? | Wrong-commitment canary + shared oracle | Phase 5 |

## Review rule

A verifier integration should have at least one positive test, one negative control, and one independently derived expected result for every security-critical policy family. Where a pattern is not applicable, document why rather than silently omitting it.

For circuit-soundness testing, use a deliberately weakened circuit as a mutation benchmark and require the correct circuit to reject the same malicious statement. The benchmark is evidence of sensitivity to tested mutation classes, not a complete soundness proof.

## Evidence rule

Record exact test commands, corpus sizes, detected failures, remediations, and known limitations. A green run demonstrates only what the executed corpus and checks cover.
