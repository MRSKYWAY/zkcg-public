# DeFi Lending Simulation (Halo2) — Architecture

This document explains **exactly what happens in the code**, step by step, matching
`verifier/examples/de_fi_lending_sim.rs`.

---

## Goal

Privately evaluate loan eligibility:

> **Prove:** `score ≤ threshold`  
> **Without revealing the score**

---

## Code-Level Flow (Exact)

### 1. Score Generation (off-chain, example only)

```rust
let scores: Vec<u64> = ...
```

Synthetic credit scores for benchmarking.
**Not part of the protocol.**

---

### 2. Halo2 Circuit

```rust
ScoreCircuit {
    score: Value::known(score),
    threshold: Value::known(threshold),
}
```

Constraint enforced:

```
threshold = score + diff
diff ≥ 0
```

Meaning:

```
score ≤ threshold
```

---

### 3. Proof Generation (per loan)

```rust
create_proof(
    params,
    &pk,
    &[circuit],
    &[public_inputs],
)
```

Each loan produces **one Halo2 proof**.

---

### 4. Verification (stateless)

```rust
let proof = Proof::new(ProofSystem::Halo2, proof_bytes);
Verifier::verify(&proof, &inputs)
```

ZKCG routes the proof to the Halo2 adapter and verifies:

| Parameter | Meaning |
|--------|--------|
| `proof` | ZK proof of policy |
| `threshold` | Public policy |
| `score` | Private witness |

❌ No protocol state  
❌ No replay protection  
❌ No ordering guarantees  

---

## Diagram

```
User
 │
 │  score
 ▼
Halo2 Prover
 │
 │  proof(score ≤ threshold)
 ▼
ZKCG Verifier
 (Halo2 adapter)
 │
 │  accept / reject
 ▼
Application Logic
```

---

## What The Universal Verifier Checks For Halo2

✅ Policy correctness  
✅ Cryptographic validity  

❌ Order of execution  
❌ Replay protection  
❌ Aggregation / batching  

---

## Why This Exists

Halo2 is ideal for:

* Fast proving
* High throughput
* Interactive applications

But it **does not model protocol execution**.

That is where zkVM comes in.

---
