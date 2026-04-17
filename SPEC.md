# Protocol Specification — ZK-Verified Computation Gateway (ZKCG)

This document specifies the core protocol, state machine, proof interfaces, and transition rules of the **ZK-Verified Computation Gateway (ZKCG)**.

It is designed to be:

- **Precise** — deterministic in behavior  
- **Auditable** — comprehensible by other engineers  
- **Robust** — covers edge cases and error conditions  

---

## Table of Contents

1. Protocol Overview  
2. Actors  
3. Core Concepts  
4. State Definition  
5. Message Formats  
6. Valid State Transition Rules  
7. Policy Constraints  
8. Verifier Semantics  
9. Error Codes & Rejections  
10. Universal Verifier Layer  
11. Batch Verification  
12. Extensions  

---

## 1. Protocol Overview

ZKCG is a verifier protocol that enables clients (provers) to submit zero-knowledge proofs attesting that a computation was executed correctly and adheres to specific policy constraints.

A verifier node validates the proof and updates the protocol state when all checks pass.

---

## 2. Actors

- **Prover (Client)** — Executes computation off-chain and produces a ZK proof  
- **Verifier Node** — Validates proofs, enforces policies, updates state  
- **Observer** — Optional read-only entity monitoring public state  

All actors may be real machines in a distributed system.

---

## 3. Core Concepts

### 3.1 Proof

A proof is a portable object that carries both the proof bytes and the proof-system identifier:

```rust
enum ProofSystem {
    Halo2,
    ZkVm,
    Groth16,
    Stark,
    Custom(&'static str),
}

struct Proof {
    system: ProofSystem,
    data: Vec<u8>,
}
```

Applications call a single verifier entrypoint and do not need proof-system-specific verification APIs.

### 3.2 Public Inputs

Public inputs are typed claims, not one fixed struct. The current public claim
families are:

- `Phase1ScoreV1`
- `RwaCreditOnboardingV1`
- `RwaCreditTransferV1`

Each proof binds both:

- workflow facts
- the expected decision commitment

### 3.3 Private Inputs

Data used by the prover but not revealed to the verifier.

### 3.4 Commitment

A cryptographic commitment (e.g., Merkle root) representing the post-computation state.

---

## 4. State Definition

The verifier maintains a deterministic state:

```rust
struct ProtocolState {
    state_root: Hash,
    nonce: u64,
    epoch: u64,
}
```

- `state_root`: Merkle commitment representing current state  
- `nonce`: Strictly increasing counter  
- `epoch`: Version or generation identifier  

---

## 5. Message Formats

### 5.1 Proof Submission

```json
{
  "system": "halo2 | zkvm",
  "proof": "<base64-encoded proof>",
  "public_inputs": "<typed proof claims payload>",
  "new_state_commitment": "<hash>"
}
```

---

## 6. Valid State Transition Rules

A transition is valid if **all** of the following hold:

1. `public_inputs.old_state_root == current.state_root`  
2. `public_inputs.nonce == current.nonce + 1`  
3. The ZK proof is valid  
4. The computed result satisfies all policy constraints  
5. `new_state_commitment` correctly reflects the post-computation state  

If any rule fails, the submission is rejected.

---

## 7. Policy Constraints

### Phase 1 Constraint

A private risk or score check is enforced:

```
computed_score ≤ threshold
```

This constraint **must be embedded in the proof** and cannot be bypassed by the prover.

### RWA Composite Constraints

The public `rwa.credit.onboarding.v1` and `rwa.credit.transfer.v1` proof
programs prove deterministic evaluation over normalized facts, including:

- fixed identifiers and bound wallet addresses
- normalized boolean checks such as KYC/AML/sanctions/jurisdiction status
- expiry and workflow binding
- numeric cap and concentration flags
- deterministic `decision`, `eligibility_class`, and `reason_bits`

These proof programs do **not** prove raw upstream evidence truth or dynamic
list resolution inside the proof.

---

## 8. Verifier Semantics

Upon receiving a proof submission, the verifier performs the following steps:

1. Parse the message  
2. Validate message format  
3. Check that `old_state_root` and `nonce` match current state  
4. Route `Proof.system` to the corresponding verifier adapter  
5. Verify the ZK proof using the provided public inputs  
6. For zkVM proofs, verify the receipt against the expected image ID and committed journal  
7. Enforce policy constraints  
8. Compute and persist the new state  
9. Emit an event or log entry  

All steps are deterministic.

---

## 9. Error Codes & Rejections

| Code | Meaning |
|----|----|
| `ERR_INVALID_FORMAT` | Bad message structure |
| `ERR_STATE_MISMATCH` | Old state does not match current |
| `ERR_NONCE_INVALID` | Invalid nonce |
| `ERR_PROOF_INVALID` | Proof verification failed |
| `ERR_POLICY_VIOLATION` | Policy constraint not satisfied |
| `ERR_COMMITMENT_MISMATCH` | New commitment does not match |

Each error must be returned to the client and logged by the verifier.

---

## 10. Universal Verifier Layer

The verifier exposes a stable application-facing API:

```rust
Verifier::verify(&proof, &public_inputs)
```

The dispatcher resolves the proof system and forwards to a concrete adapter:

```text
Application
    |
    v
Verifier::verify
    |
    +--> Halo2 adapter
    +--> zkVM adapter
    +--> custom registry entry
```

The default registry includes bundled Halo2 and zkVM adapters when their features are enabled.

A public `VerifierRegistry` allows applications to register additional proof-system implementations while preserving the same verifier trait and proof object.

---

## 11. Batch Verification

The verifier supports both sequential and parallel batch execution over `(Proof, PublicInputs)` pairs.

Library entrypoints:

```rust
Verifier::verify_batch(&requests)
Verifier::verify_batch_parallel(&requests)
Verifier::verify_batch_parallel_results(&requests)
```

API entrypoints:

```text
POST /demo/verify-batch
POST /v1/verify-batch
```

Batch API responses include overall batch status, total proof count, and per-proof results in request order.

---

## 12. Extensions

### 12.1 Pluggable Proof Backends

ZKCG supports multiple proof systems:

- Circuit-based proofs (e.g., Halo2)  
- zkVM proofs (e.g., RISC Zero, SP1)  
- SNARK backends (e.g., Groth16)  
- STARK backends  

The verifier interface remains stable; only backend verification logic differs.

---

### 12.2 Versioning

The `epoch` field enables protocol upgrades and routes verification logic to the correct version.

---

## Provenance Statement

This specification is designed to be:

- Unambiguous  
- Machine-verifiable  
- Extensible  

All state transitions and policy checks are deterministic.
