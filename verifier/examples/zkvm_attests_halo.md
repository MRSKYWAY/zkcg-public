# zkVM Attests Halo2 — Architecture

This document explains `verifier/examples/zkvm_attests_halo2.rs`
**function by function**, matching the code exactly.

---

## Goal

Use **zkVM** to attest that:

> “Halo2 verified these loan decisions **in the correct protocol order**.”

---

## Phase 1 — Halo2 (Policy Layer)

### Code

```rust
let proof = Proof::new(ProofSystem::Halo2, proof_bytes);
Verifier::verify(&proof, &inputs)
```

ZKCG routes to the Halo2 adapter and verifies:

| Property | Enforced |
|-------|---------|
| `score ≤ threshold` | ✅ |
| Proof validity | ✅ |

Output:

```rust
Vec<u64> approved_scores
```

---

## Phase 2a — zkVM Proof Generation (Parallel)

### Code

```rust
zkvm_prove(score, threshold, dummy_root, dummy_nonce)
```

Each approved loan produces a **zkVM proof** that:

```
(score ≤ threshold) was evaluated
```

⚠️ Stateless  
⚠️ No ordering  
⚠️ No state yet  

---

## Phase 2b — zkVM Attestation (Sequential)

### Code

```rust
engine.process_transition(
    proof,
    inputs,
    commitment,
)
```

This is the **critical step**.

zkVM verifies:

| Check | Purpose |
|---|---|
| Method ID | Correct program |
| Journal hash | Correct public inputs |
| State root | No replay |
| Nonce | Ordered execution |

---

## Why This Is Sequential

Protocol state evolves:

```
state₀ → state₁ → state₂ → ...
```

Each transition depends on the previous one.

Parallel verification would allow:

❌ Double execution  
❌ Replay attacks  
❌ State forks  

---

## Diagram

```
            ┌─────────────┐
            │  Halo2 ZK   │
            │  Proofs     │
            └─────┬───────┘
                  │ approved scores
                  ▼
        ┌───────────────────────┐
        │ zkVM Prover (parallel)│
        │  prove(policy ran)    │
        └─────┬─────────────────┘
              │ zkVM proofs
              ▼
        ┌─────────────────────┐
        │ zkVM VerifierEngine │
        │  (SEQUENTIAL)       │
        └─────────────────────┘
```

---

## What zkVM Verifies (That Halo2 Cannot)

✅ Correct execution order  
✅ Replay protection  
✅ State root continuity  
✅ Program identity  

---

## Combined Security Model

| Layer | Guarantees |
|----|-----------|
| Halo2 | Policy correctness |
| zkVM | Protocol correctness |

Together:

> **“The right rules were enforced, in the right order.”**

---

## Why This Matters

This pattern is used in:

* Rollups
* Compliance engines
* On-chain governance
* Audit logs
* ZK attestations

You are **not replacing Halo2** — you are **anchoring it**.

---
