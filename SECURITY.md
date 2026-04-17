# Security Model

This document outlines the threat model, security assumptions, and non-goals of the ZK-Verified Computation Gateway (ZKCG).

---

## Threat Model

### Adversary Capabilities
An adversary may:
- Submit malformed or adversarial proofs
- Attempt to replay previously valid proofs
- Attempt to submit proofs that violate protocol policy
- Attempt to cause verifier denial-of-service via invalid inputs
- Attempt to infer private inputs from public data

The adversary **cannot**:
- Break standard cryptographic assumptions
- Compromise the verifier host environment (out of scope)

---

## Security Properties

ZKCG is designed to provide the following guarantees:

### 1. Correctness
Only computations that satisfy the protocol-defined constraints and policies can produce valid state transitions.

### 2. Soundness
Invalid computations cannot produce valid proofs under the assumed security of the underlying proof system.

### 3. Replay Protection
Each state transition requires a strictly increasing nonce, preventing replay of previously accepted proofs.

### 4. Privacy
Private inputs used during computation are never revealed to the verifier. Only public inputs and commitments are exposed.

### 5. Deterministic State Transitions
Given the same prior state and inputs, the verifier produces the same next state.

### 6. Public Auditability Of Published Proof Logic
The security story depends on the published proof logic, not just the verifier API.

For the public Halo2 path, the circuits and verifier artifact generation are in the
public repository:

- [circuits/src/score_circuit.rs](/home/skye/ZKCG/circuits/src/score_circuit.rs)
- [circuits/src/rwa_circuit.rs](/home/skye/ZKCG/circuits/src/rwa_circuit.rs)
- [circuits/src/halo2_artifacts.rs](/home/skye/ZKCG/circuits/src/halo2_artifacts.rs)

For the zkVM path, the host/verifier logic and guest binding are also public.

If a proof system or workflow is not backed by published circuit or guest logic,
it should not be described as removing trust in implementation details.

---

## Current Proof Scope

The current public proof scope is narrower than the broadest product narrative.

Today, the public proof programs cover:

- `score <= threshold`
- `rwa.credit.onboarding.v1`
- `rwa.credit.transfer.v1`

The RWA proof programs prove deterministic policy evaluation over normalized
facts and numeric limits. They do **not** prove:

- raw upstream KYC/AML/accreditation evidence truth
- allowlist/blocklist resolution inside the proof
- arbitrary customer policy DSL execution

This distinction matters. Proof-backed infrastructure is only as strong as the
published circuit or guest logic for the exact workflow being claimed.

---

## Assumptions

ZKCG assumes:
- The cryptographic soundness of the underlying zero-knowledge proof system
- The correctness of the published circuit or guest logic for the workflow in use
- Correct implementation of cryptographic primitives
- Correctness of the verifier node implementation
- Secure key management by provers

### Backend-Specific Setup Assumptions

- `zk-vm` depends on the published guest image and receipt verification flow.
- `zk-halo2` depends on the published Halo2 circuit and verifier-artifact generation path.
- `zk-halo2-kzg` is the explicit KZG backend and therefore carries a trusted-setup assumption.

The repo should describe backend assumptions precisely rather than using a blanket
“trustless” label.

### Circuit Risk

Under-constrained circuits are a major ZK risk class.

Publishing the circuit is necessary, but it is not sufficient. Stronger assurance
also requires:

- adversarial tests
- circuit review
- eventual external audit
- clear documentation of exactly what is and is not proven

---

## Out of Scope

The following are explicitly **out of scope**:
- Compromise of the verifier host or operating system
- Side-channel attacks on prover environments
- Economic or incentive-layer attacks
- Network-level censorship or availability guarantees
- A claim that every current hosted product workflow is already captured by a public audited composite circuit

---

## Reporting Vulnerabilities

Security vulnerabilities should be reported **privately**.

Please contact:
- Email: security@zkcg.local (placeholder)

Do not open public issues for security-sensitive bugs.
