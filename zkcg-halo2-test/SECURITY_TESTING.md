# Security testing guidance

The reference ZKCG evaluation uses the following workflow:

1. Establish an independent expected result before testing the verifier.
2. Exercise adversarial values against the real proving and verification path.
3. Mutate security-relevant public claims and require rejection.
4. Maintain a deterministic malformed-proof corpus including truncation, bit flips, replacements, and trailing-byte cases.
5. Check replay and domain/identity binding with changed public inputs.
6. Compare independent backends against an independent oracle where multiple proving systems exist.
7. Run controlled circuit mutations to benchmark whether missing constraints are observable.
8. Record limitations and evidence explicitly; passing tests are not a proof of complete security.
