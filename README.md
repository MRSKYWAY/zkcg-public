# ZKCG — Proof-Backed RWA Compliance Gateway

[![zkcg-verifier](https://img.shields.io/crates/v/zkcg-verifier.svg)](https://crates.io/crates/zkcg-verifier)
[![zkcg-common](https://img.shields.io/crates/v/zkcg-common.svg)](https://crates.io/crates/zkcg-common)
[![zkcg-halo2-prover](https://img.shields.io/crates/v/zkcg-halo2-prover.svg)](https://crates.io/crates/zkcg-halo2-prover)
[![zkcg-zkvm-host](https://img.shields.io/crates/v/zkcg-zkvm-host.svg)](https://crates.io/crates/zkcg-zkvm-host)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-brightgreen)](https://github.com/sponsors/MRSKYWAY)

---
# ZK-Verified Computation Gateway (ZKCG)

**ZKCG is the proof-backed compliance gateway for tokenized private credit.**  
Use one hosted API to evaluate wallet onboarding and transfer eligibility off-chain, then enforce the result on-chain with signed attestations and auditable decision outputs.

ZKCG is an **open-core verifier + hosted compliance gateway**.  
The current product surface is optimized for issuer-facing RWA workflows that need replay-safe verification, stable API responses, public proof programs, signed on-chain settlement attestations, and a clean hosted deployment story.

Learn more below 👇

## What ZKCG Ships Today

Primary workflows:

- wallet onboarding eligibility
- transfer approval for tokenized credit assets
- issuer-scoped audit export
- signed on-chain settlement attestation

Primary buyer:

- issuer
- fund admin
- transfer agent
- tokenization platform

Technical integrator:

- backend engineer
- platform engineer
- smart-contract engineer consuming settlement attestations

## Current Trust Model

ZKCG is **not** a blanket “trustless everything” system today.

What is public and proof-backed:

- public proof programs for `phase1.score.v1`
- public proof programs for `rwa.credit.onboarding.v1`
- public proof programs for `rwa.credit.transfer.v1`
- public verifier logic for Halo2 and zkVM
- public replay/state/attestation binding logic

What the proofs cover:

- deterministic policy evaluation over **normalized facts**
- stable decision outputs such as `decision`, `eligibility_class`, and `reason_bits`

What the proofs do **not** cover today:

- raw upstream KYC / AML / accreditation evidence truth
- allowlist / blocklist resolution inside the proof
- native backend-specific on-chain proof verification

Current on-chain settlement is **attestation-backed**:

- ZKCG produces a proof-backed decision off-chain
- the API signs a contract-ready attestation for that decision
- contracts consume `verifyZKCG(proof, publicInputs)` plus the bound payload hash

## Overview

**ZKCG Verifier** is the public verification layer underneath the hosted compliance gateway.

* **Published proof programs**: phase-1 score checks plus RWA onboarding and transfer evaluation over normalized facts
* **Backends**: Halo2 and zkVM (RISC0), with Halo2 also powering self-hosted bulk payout release gating

This repository ships the open-core verifier, the zkVM/Halo2 adapters, the Solidity settlement interface, and a reference API for proof-backed RWA compliance decisions.
Anyone can independently verify proofs, audit the logic, run the API locally, and self-host the protocol-facing endpoints.

---
## High-Level Architecture

Here’s the primary product flow:

```
Issuer / admin policy inputs
        ↓
Normalized facts + policy evaluation
        ↓
Public proof program (Halo2 / zkVM)
        ↓
Hosted API decision + audit record
        ↓
Signed on-chain attestation
        ↓
Settlement contract / service consumer
```

- The proof program checks deterministic policy evaluation over normalized facts.
- The API returns stable machine-readable decision outputs.
- The current settlement path uses signed attestations that bind the on-chain action to the decision.

---

## Why Teams Use ZKCG

Teams evaluating tokenized private credit and permissioned RWA flows usually need:

- reusable onboarding and transfer decisions
- issuer-scoped audit history
- replay-safe settlement approvals
- public proof logic for the decision semantics
- an integration surface simple enough for contracts and backend systems

ZKCG is designed for that exact operating model.

## Secondary Examples

The repo also includes secondary examples that reuse the same verifier and settlement shape:

- private loan eligibility
- risk-gated vaults
- oracle sanity checks
- zkMe credential-to-policy bridge demo

---

## Canonical Product Example

The canonical end-to-end flow in this repo is:

- `POST /v1/rwa/onboarding/evaluate`
- `POST /v1/rwa/transfer/evaluate`
- `POST /v1/rwa/transfer/attest`
- `GET /v1/rwa/audit/export`

See [api/examples/rwa_credit_workflow.md](https://github.com/MRSKYWAY/ZKCG/blob/main/api/examples/rwa_credit_workflow.md) for the full walkthrough.

For the zkMe partner-stack bridge, see [docs/zkme_zkcg_integration.md](/home/skye/ZKCG/docs/zkme_zkcg_integration.md) and run:

```bash
bash scripts/demo_zkme_zkcg_fullstack.sh
```

---


## Repository Structure

```text
ZKCG/
├── common/         # Shared types, errors, and protocol utilities (zkcg-common crate)
├── circuits/       # Public Halo2 circuit definitions and verifier artifacts
├── halo2/prover/   # Public Halo2 proving crate
├── payout/worker/  # Self-hosted payout worker / CLI for bulk release gating
├── verifier/       # Core verifier logic (zkcg-verifier crate)
├── api/            # Hosted/reference API for RWA onboarding, transfer approval, and secondary examples
├── contracts/      # Solidity verifier + consumer examples
├── docs/           # Product, market, and security-adjacent docs
├── SPEC.md         # Full protocol specification
├── CORE_FREEZE.md  # Frozen circuit parameters and commitments
├── SECURITY.md     # Security assumptions and reporting
├── LICENSE         # Apache-2.0
└── README.md       # This file
```

---

## Installation

Add the crates to your project:

```bash
cargo add zkcg-verifier zkcg-common
```

Or manually in `Cargo.toml`:

```toml
[dependencies]
zkcg-verifier = "0.2.0"
zkcg-common   = "0.2.0"
```

---
## 🚀 Prover Crate (Now on crates.io!)

```toml
zkcg-halo2-prover = "0.2.0"
```
Generate Halo2 proofs locally in one line:

```Rust
use zkcg_halo2_prover::{generate_proof, proving_backend_name};

let proof = generate_proof(35, 40)?;   // score <= threshold
println!("backend = {}", proving_backend_name());

println!("✅ Proof generated! Size: {} bytes", proof.len());
``` 
For repeated proofs, cache the proving context instead of rebuilding params and
keys on every call:

```rust
use zkcg_halo2_prover::{DEFAULT_K, Halo2ProverContext, proving_backend_name};

let context = Halo2ProverContext::new(DEFAULT_K)?;

let proof_a = context.prove(35, 40)?;
let proof_b = context.prove(55, 60)?;

println!("backend = {}", proving_backend_name());
println!("proof sizes = {}, {}", proof_a.len(), proof_b.len());
```

Full docs → https://docs.rs/zkcg-halo2-prover

Public prover source → https://github.com/MRSKYWAY/ZKCG/tree/main/halo2/prover
Public circuits → https://github.com/MRSKYWAY/ZKCG/tree/main/circuits
Cached-context example → `halo2/prover/examples/reuse_context.rs`

Want hosted proving, custom circuits, or SLA? → GitHub Sponsors or DM me!

GPU build path:

```bash
cargo +nightly run -p zkcg-halo2-prover --no-default-features --features icicle-gpu
```

This requires a CUDA-capable environment with `nvcc` available during build.
If CUDA is installed in a non-default location, set `CUDAToolkit_ROOT` before
building.

Dead-simple GPU smoke check:

```bash
make gpu-check
```

## Throughput-Focused Payout Backend

ZKCG now ships a specialized Halo2 chunked payout backend for self-hosted bulk payout rounds.

- prover module: [halo2/prover/src/payout.rs](/home/skye/ZKCG/halo2/prover/src/payout.rs)
- worker CLI: [payout/worker/README.md](/home/skye/ZKCG/payout/worker/README.md)
- bench mode: `bash ./scripts/bench.sh halo2-payout`
- demo script: `bash ./scripts/demo_bulk_payout_round.sh`

This backend is intended for frozen payout rounds and off-chain release gating. It runs on the standard Halo2 prover path and shares the same modular proving surface as the rest of the public Halo2 backend.

The current Halo2 payout path uses:

- fixed-size chunk proofs over payout rows
- a serialized payout proof bundle containing the chunk proofs plus the released rows
- verifier-side recomputation of manifest, totals, caps, and release-window policy checks

The default benchmark mode focuses on 128, 256, and 1024 row Halo2 chunk proving. Use the `prove_chunked_round` example for 1k and 10k end-to-end payout round measurements.

## Public Circuits

The proving circuits used by the public Halo2 path are **in this repository**.

Primary circuit paths:

- [circuits/src/score_circuit.rs](/home/skye/ZKCG/circuits/src/score_circuit.rs)
- [circuits/src/rwa_circuit.rs](/home/skye/ZKCG/circuits/src/rwa_circuit.rs)
- [circuits/src/halo2_artifacts.rs](/home/skye/ZKCG/circuits/src/halo2_artifacts.rs)
- [halo2/prover/src/lib.rs](/home/skye/ZKCG/halo2/prover/src/lib.rs)
- [verifier/src/adapters/halo2.rs](/home/skye/ZKCG/verifier/src/adapters/halo2.rs)

If someone concluded the circuits were private, that is a documentation failure, not the actual repo state.

## Proof Scope Today

ZKCG should be evaluated based on what is publicly proven today, not on the broadest product framing.

Current public proof scope:

- The public Halo2 path proves:
  - `score <= threshold`
  - `rwa.credit.onboarding.v1`
  - `rwa.credit.transfer.v1`
- The zkVM path proves the same public claims through the published guest logic plus receipt/journal binding.
- Replay protection, state binding, and attestation binding are public and auditable in the verifier/API/contracts code.

What is **not** true today:

- The system does **not** prove raw upstream KYC/AML/accreditation evidence truth.
- The system does **not** prove allowlist/blocklist resolution or generic policy-DSL execution inside the proof.
- The current RWA proof programs prove deterministic evaluation over normalized booleans, enums, identifiers, and numeric limits.

The correct current framing is:

- public verifier + public circuits/guest logic for the shipped score and RWA workflows
- proof-backed policy evaluation over normalized facts
- hosted attestation plumbing for on-chain settlement

## Features

* `zk-halo2` — Enable Halo2 proof verification backend
* `zk-halo2-kzg` — Enable the nightly-only KZG verifier for ICICLE / zkonduit Halo2 proofs
* `zk-vm` — Enable zkVM (RISC0) verification support

Example:

```toml
zkcg-verifier = { version = "0.2.0", features = ["zk-halo2"] }
```

To accept both the stable CPU Halo2 proofs and the nightly KZG / ICICLE Halo2
proofs through the same `ProofSystem::Halo2` entrypoint:

```toml
zkcg-verifier = { version = "0.2.0", features = ["zk-halo2", "zk-halo2-kzg"] }
```

## Setup Assumptions

The setup story depends on the backend you enable.

- `zk-vm`:
  - Soundness depends on the published guest program and receipt verification path.
- `zk-halo2`:
  - Soundness depends on the published Halo2 circuit and verifier artifact generation in the repo.
  - The repo currently generates bundled verifier artifacts from the public circuit at runtime.
- `zk-halo2-kzg`:
  - This is the explicit KZG path.
  - It carries a trusted-setup assumption and should be described that way.
  - If you market this backend, document the setup provenance explicitly.

So the product should not use “trustless” as a blanket statement across all backends and workflows without qualification.

---

## 🐳 Docker Setup (Optional)

Docker is **optional**.

* Halo2 verification runs natively
* zkVM verification can run natively or in Docker
* Docker is recommended for **reproducible environments and CI**

### Install Docker (Ubuntu / WSL2)

```bash
sudo apt update
sudo apt install docker.io -y
sudo usermod -aG docker $USER
newgrp docker
```

Verify:

```bash
docker --version
```

---

### Build Docker Image

From the repository root:

```bash
docker build -t zkcg-api .
```

### Run The Revenue-First API

The primary product-facing endpoints in `v0.2.0` are:

- `/v1/rwa/onboarding/evaluate`
- `/v1/rwa/transfer/evaluate`

The lower-level `/v1/submit-proof` endpoint remains available for protocol/state transitions, and `/v1/compliance/evaluate` stays available as a secondary compatibility example.

Run locally with persistent state:

```bash
ZKCG_ENABLE_PROTOCOL=1 \
ZKCG_STATE_BACKEND=sqlite \
ZKCG_STATE_PATH=./data/protocol-state.db \
cargo run -p api --features "zk-halo2 zk-vm"
```

Run with Docker and a mounted state volume:

```bash
docker run --rm -p 8080:8080 \
  -v "$(pwd)/data:/data" \
  zkcg-api
```

For hosted reference deployments, the root `Dockerfile` is suitable for Railway-style container deploys and persists protocol state through `ZKCG_STATE_PATH`.

Operational checks:

```bash
curl -sS http://127.0.0.1:8080/healthz
curl -sS http://127.0.0.1:8080/v1/protocol/state
./scripts/smoke_api.sh
```


## 📊 Benchmarks

> **Environment**
>
> * Platform: Windows (WSL2, Ubuntu)
> * CPU: Intel i5 (10th Gen)
> * RAM: 16 GB
> * Build: Release
> * Parallelism: Default (no tuning)

---

### Halo2 (BN254, k = 6)

**Use case:** Interactive / near-real-time ZK policy verification

Current measured paths on this machine:

* **Verify (stable CPU verifier bench):** ~`4.22–4.58 ms`
* **Prove, cached context (stable CPU):** ~`75.98–77.29 ms`
* **Prove, cached context (ICICLE GPU):** ~`68.07–71.24 ms`
* **Prove, one-shot API (stable CPU):** ~`112.87–114.19 ms`
* **Prove, one-shot API (ICICLE GPU):** ~`105.29–106.93 ms`
* **Setup only (CPU / GPU):** ~`36.80–37.23 ms` / `33.40–35.62 ms`

Interpretation:

* The old `~75–80 ms` Halo2 prove number still holds on the cached-context path.
* The higher `~106–114 ms` numbers include one-shot setup (`params`, `vk`, `pk`) on every call.
* On this WSL2 + GTX 1650 Ti setup, ICICLE improves the cached-context proving path by about `9%` versus CPU.

---

### zkVM (RISC0)

**Use case:** Audit-grade execution proofs

* **Prove:** ~13.7 seconds
* **Verify:** ~41–42 ns

---

### Summary

| Backend | Prove Time | Verify Time | Intended Use            |
| ------- | ---------- | ----------- | ----------------------- |
| Halo2 (cached CPU) | ~76.6 ms | ~4.38 ms | Interactive ZK policies |
| Halo2 (cached GPU) | ~69.6 ms | n/a | Interactive ZK policies |
| zkVM    | ~13–17 s   | ~40 ns      | Audit / attestation     |

---

## 🧪 Running Benchmarks

Stable CPU verifier + prover benchmarks:

```bash
bash ./scripts/bench.sh halo2
bash ./scripts/bench.sh halo2-cpu
```

Nightly ICICLE GPU prover benchmark:

```bash
bash ./scripts/bench.sh halo2-gpu
```

zkVM verifier benchmark:

```bash
bash ./scripts/bench.sh zkvm
```

`halo2-gpu` requires a CUDA-capable environment plus the nightly toolchain.

### End-to-End Simulation Results

#### Sequential Halo2 Simulation (1000 proofs)

```
Loans evaluated: 1000
Approvals: 128 (12.8%)

Prove total:   ~482.1 s
Verify total:  ~7.7 s
Throughput:    ~2.0 TPS
```

#### Parallel Halo2 Simulation (8 threads)

```
Loans evaluated: 1000
CPU threads: 8

Approvals: 130 (13.0%)

Prove total:   ~127.4 s
Verify total:  ~5.5 s
Throughput:    ~7.5 TPS
```

---

### Summary

| Backend | Prove Cost | Verify Cost | Throughput | Intended Use |
|------|-----------|------------|-----------|--------------|
| Halo2 (seq) | ~480 ms | ~7 ms | ~2 TPS | Interactive ZK policies |
| Halo2 (8-core) | ~127 ms | ~5 ms | ~7.5 TPS | Batch / off-chain proving |
| zkVM | ~13–17 s | ~40 ns | Prove-bound | Audit & attestation |

---

## Revenue-First Product Surface

The current `v0.2.0` surface is intentionally narrower than a generic “all of ZK” platform.
It is built to help close paid pilots around proof-backed business decisions.

Flagship flow:
- RWA credit onboarding and transfer approval through `/v1/rwa/onboarding/evaluate` and `/v1/rwa/transfer/evaluate`
- Stable decision envelope with `decision_id`, `decision`, `policy_version`, `proof_system`, machine-readable `reason_codes`, human-readable `reasons`, and proof artifacts
- [RWA credit workflow walkthrough](./api/examples/rwa_credit_workflow.md)
- [RWA go-to-market brief](./docs/rwa_go_to_market.md)

Secondary template:
- Legacy lending/compliance compatibility flow through `/v1/compliance/evaluate`
- [Private lending pilot walkthrough](./api/examples/private_lending_pilot.md)
- Risk-gated vault or oracle replacement flows that reuse the same decision + verification pattern
- [Risk-gated vault example](./api/examples/risk_gated_vault.md)
- [On-chain settlement example](./api/examples/on_chain_settlement.md)
- [Contract examples and adoption path](./contracts/README.md)
- [RwaTransferGate.sol](./contracts/examples/RwaTransferGate.sol)
- [PrivateLoanEligibilityGate.sol](./contracts/examples/PrivateLoanEligibilityGate.sol)
- [RiskGatedVault.sol](./contracts/examples/RiskGatedVault.sol)
- [VerifiableOracleWithPolicy.sol](./contracts/examples/VerifiableOracleWithPolicy.sol)

Contract-ready attestation endpoints:
- `POST /v1/rwa/onboarding/attest`
- `POST /v1/rwa/transfer/attest`
- `POST /v1/onchain/private-loan-eligibility/attest`
- `POST /v1/onchain/risk-gated-vault/attest`
- `POST /v1/onchain/verifiable-oracle/attest`
- Requires `ZKCG_ATTESTATION_PRIVATE_KEY` in the API environment

Dead-simple local on-chain flow:

```bash
make onchain-test
make onchain-demo
make onchain-rwa-demo
```

The existing verifier examples remain useful for backend benchmarking and protocol evaluation:
- [Halo2 lending simulation](./verifier/examples/de_fi_lending_sim.rs)
- [Parallel Halo2 lending simulation](./verifier/examples/de_fi_lending_sim_parallel.rs)
- [zkVM lending simulation](./verifier/examples/zkvm_lending_sim.rs)

## Live Demo API (Stateless)

The ZKCG verifier exposes **demo-only, stateless endpoints** that allow anyone to try proof generation and verification without running the stack locally.

> ⚠️ These endpoints are for **testing and demonstration only**.  
> They do **not** persist protocol state and are **rate-limited**.

**Base URL**
```
https://zkcg.onrender.com
```

---

## 1️⃣ Generate a Proof (`/demo/prove`)

Generate a zero-knowledge proof that a `score` satisfies a given `threshold`.

### Request

```bash
curl -X POST https://zkcg.onrender.com/demo/prove \
  -H "Content-Type: application/json" \
  -d '{
    "score": 90,
    "threshold": 100
  }'
```

### Response

```json
{
  "proof": "<PROOF>",
  "proof_size_bytes": 64,
  "note": "Demo-only stateless proof"
}
```

- `proof` is a base64-encoded ZK proof  
- `proof_size_bytes` shows the compact proof size  
- The proof is **not stored server-side**

---

## 2️⃣ Verify a Proof (`/demo/verify`)

Verify a previously generated proof against a threshold.

### Request

```bash
curl -X POST https://zkcg.onrender.com/demo/verify \
  -H "Content-Type: application/json" \
  -d '{
    "system": "halo2",
    "proof": "<YOUR_PROOF>",
    "threshold": 100
  }'
```

### Response

```json
{
  "verified": true
}
```

## 3️⃣ Verify Proofs In Batch (`/demo/verify-batch`)

Verify multiple independent proofs in one request.

### Request

```bash
curl -X POST https://zkcg.onrender.com/demo/verify-batch \
  -H "Content-Type: application/json" \
  -d '{
    "proofs": [
      {
        "system": "halo2",
        "proof": "<PROOF_ONE>",
        "threshold": 100
      },
      {
        "system": "halo2",
        "proof": "<PROOF_TWO>",
        "threshold": 100
      }
    ]
  }'
```

### Response

```json
{
  "verified": false,
  "total": 2,
  "results": [
    {
      "verified": true
    },
    {
      "verified": false,
      "error": "proof verification failed"
    }
  ]
}
```

Response notes:
- the hosted Render deployment currently uses `halo2` for the public stateless demo endpoints
- `system: "zkvm"` remains supported in local or dual-backend deployments, but is not the default for the hosted API


## RWA Credit Compliance API

These endpoints are the primary product-facing API in `v0.2.0`.
They expose a **deterministic, proof-backed onboarding and transfer decision engine** for tokenized credit issuers, fund admins, and transfer agents.  
They evaluate predefined issuer-side policy logic over supplied claims and return:

- The normalized decision envelope
- Stable machine-readable `reason_codes`
- Human-readable reasons for failure (if any)
- The proof system used for the artifact
- A cryptographic proof artifact when one is available

⚠️ **Important**  
This service does **not** source data, perform KYC, or assess fraud.  
It proves *correct execution of policy logic* over caller-supplied claims.

---

### Primary Endpoints

```
POST /v1/rwa/onboarding/evaluate
POST /v1/rwa/onboarding/attest
POST /v1/rwa/transfer/evaluate
POST /v1/rwa/transfer/attest
GET  /v1/rwa/policies
GET  /v1/rwa/decisions/{decision_id}?issuer_id=...
GET  /v1/rwa/audit/export?issuer_id=...&format=json|csv
```

Admin-only controls when `ZKCG_ADMIN_TOKEN` is configured:

```
POST /v1/rwa/policies/{policy_version}/status
POST /v1/rwa/revocations
```

Secondary compatibility endpoint:

```
POST /v1/compliance/evaluate
Content-Type: application/json
```

Operational state endpoint:

```
GET /v1/protocol/state
```

Additional protocol-facing verification endpoint:

```
POST /v1/verify-batch
Content-Type: application/json
```

Lower-level state transition endpoint:

```
POST /v1/submit-proof
Content-Type: application/json
```

---

### RWA Onboarding Request Schema

```json
{
  "issuer_id": "string",
  "asset_id": "string",
  "wallet_address": "0x-address",
  "investor_type": "string",
  "accredited": "boolean",
  "kyc_passed": "boolean",
  "aml_cleared": "boolean",
  "sanctions_clear": "boolean",
  "jurisdiction": "string",
  "allowed_jurisdictions": ["string"],
  "blocked_jurisdictions": ["string"],
  "residency_allowed": "boolean",
  "expires_at": "unix timestamp"
}
```

Hosted example:

```bash
curl -X POST https://zkcg.onrender.com/v1/rwa/onboarding/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "issuer_id":"issuer-a",
    "asset_id":"credit-fund-a",
    "wallet_address":"0x1111111111111111111111111111111111111111",
    "investor_type":"individual",
    "accredited":true,
    "kyc_passed":true,
    "aml_cleared":true,
    "sanctions_clear":true,
    "jurisdiction":"US",
    "allowed_jurisdictions":["US"],
    "blocked_jurisdictions":[],
    "residency_allowed":true,
    "expires_at":1900000000
  }'
```

---

### Example Onboarding Response

```json
{
  "decision_id": "rwa:onboarding:issuer-a:credit-fund-a:0x1111111111111111111111111111111111111111",
  "decision": "eligible",
  "policy_passed": true,
  "policy_version": "rwa.credit.onboarding.v1",
  "proof_verified": true,
  "proof_system": "halo2",
  "reason_bits": 0,
  "reason_codes": [],
  "reasons": [],
  "expires_at": 1900000000,
  "issuer_id": "issuer-a",
  "asset_id": "credit-fund-a",
  "wallet_address": "0x1111111111111111111111111111111111111111",
  "eligibility_class": "accredited",
  "claims_hash": "0x9225753f5f6eee2916816ae8d6da957ff65e0e6c7594fc2d991cca1ef65d23e7",
  "decision_commitment_hash": "0xddaea2fabbe9a3adc9a4d60e039cd097fbb74b3008c661ceaceb09902c96ef81",
  "proof_artifact": "<BASE64_PROOF>"
}
```

Response notes:
- `decision_id` is the stable caller-facing identifier for the evaluated decision
- `policy_version` is fixed to `rwa.credit.onboarding.v1` or `rwa.credit.transfer.v1`
- the hosted API currently uses `halo2` as the default proving backend
- `reason_bits` is the canonical machine-readable bitset for the decision result
- `claims_hash` and `decision_commitment_hash` are the canonical hashes bound into the attestation and contract-consumption flow
- `reason_codes` are the stable machine-readable integration contract
- `proof_artifact` may be empty when a decision is denied before a proof artifact is produced

### Health Check

```json
{
  "status": "ok",
  "policy_version": "rwa.credit.onboarding.v1",
  "state_backend": "sqlite"
}
```

### Protocol State Example

```json
{
  "state_root": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
  "nonce": 0,
  "epoch": 0
}
```

---

### Proof Semantics (Oracle Replacement)

The returned proof attests that:

> The declared policy rules were evaluated correctly over the supplied inputs and resulted in the reported outcome.

The proof guarantees:
- Deterministic policy execution
- Correct rule evaluation
- No reliance on trusted oracles

The proof does **not** guarantee:
- Correctness of inputs
- Identity validity
- Data freshness
- Absence of fraud

These are intentionally out of scope.



## Notes

- Demo endpoints are **stateless**
- No protocol state is mutated
- Intended for:
  - quick testing
  - integration experiments
  - understanding the proof flow
- Production / protocol endpoints are gated separately



## What This Demonstrates

- End-to-end proof generation
- Compact proof size
- Deterministic verification
- Clean HTTP boundary for ZK systems

## Version

Current stable release: **v0.2.0**

Features:
- Halo2 proving and verification
- zkVM proof generation and verification
- Universal verifier and public adapter registry
- Persistent protocol-state API
- Lending/compliance decision API
- Batch verification with per-item results
- Lending simulation examples
- Benchmarks

v0.2.0 includes:
- universal verifier architecture
- hardened zkVM receipt verification
- sequential and parallel batch verification
- per-proof batch API results
- public verifier registry for custom adapters

## Contact

For questions, collaborations, or sponsorships, reach out:
- X (Twitter): [@sujyot]([https://x.com/sujyot](https://x.com/Sujyot10))
- GitHub Issues: Open in this repo for discussions

---


#### Anyone can:

* Audit the verifier
* Run a verifier node
* Independently verify published proofs

Proof generation requires access to private components —
contact [@MRSKYWAY](https://github.com/MRSKYWAY) for collaboration or sponsored access.

---
## License

Apache-2.0

---

## Support the Project

ZKCG is built and maintained by a single developer.

👉 Sponsor: [https://github.com/sponsors/MRSKYWAY](https://github.com/sponsors/MRSKYWAY)


