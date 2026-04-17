# ZKCG Contract Examples

These Solidity examples support the current **tokenized private credit**
settlement path for ZKCG while keeping the trust model explicit.

Use this contract layer when you want:

- proof-backed onboarding or transfer decisions off-chain
- signed on-chain attestations for settlement
- a stable `verifyZKCG(proof, publicInputs)` interface for consumers

Primary example:

- [RwaTransferGate.sol](./examples/RwaTransferGate.sol)

Secondary examples:

- [PrivateLoanEligibilityGate.sol](./examples/PrivateLoanEligibilityGate.sol)
- [RiskGatedVault.sol](./examples/RiskGatedVault.sol)
- [VerifiableOracleWithPolicy.sol](./examples/VerifiableOracleWithPolicy.sol)

## Recommended Adoption Path

### 1. Standard verifier interface with a real implementation

Use [ZKCGVerifier.sol](./ZKCGVerifier.sol) first.

Why this is the current production default:

- It already implements [IZKCGVerifier.sol](./interfaces/IZKCGVerifier.sol)
- Apps can integrate against `verifyZKCG(proof, publicInputs)` immediately
- The current implementation is attestation-backed, so the interface stays stable
  even before backend-specific proof verifiers land
- Consuming contracts stay dead simple while still binding the verified payload
  to the actual on-chain action

### 2. One-file quickstart

Use [ZKCGSignedDecisionGate.sol](./examples/ZKCGSignedDecisionGate.sol) if you
want a single-purpose settlement gate without deploying a separate verifier
contract.

## Shipped Contracts

### Core verifier

- [IZKCGVerifier.sol](./interfaces/IZKCGVerifier.sol)
- [ZKCGVerifier.sol](./ZKCGVerifier.sol)
- [ZKCGVerifierConsumer.sol](./examples/ZKCGVerifierConsumer.sol)

### Production-grade examples

- [RwaTransferGate.sol](./examples/RwaTransferGate.sol)
- [BaseZKCGDecisionConsumer.sol](./examples/BaseZKCGDecisionConsumer.sol)
- [PrivateLoanEligibilityGate.sol](./examples/PrivateLoanEligibilityGate.sol)
- [RiskGatedVault.sol](./examples/RiskGatedVault.sol)
- [VerifiableOracleWithPolicy.sol](./examples/VerifiableOracleWithPolicy.sol)

### Direct attestation shortcut

- [ZKCGSignedDecisionGate.sol](./examples/ZKCGSignedDecisionGate.sol)

## Local Contract Testing

This repo now includes a local Foundry config in [foundry.toml](../foundry.toml) and
an end-to-end test suite in [ZKCGVerifier.t.sol](./test/ZKCGVerifier.t.sol).

Run it with:

```bash
forge test
```

Or from the repo root:

```bash
make onchain-test
make onchain-demo
```

The suite covers:

- direct verifier acceptance
- private loan eligibility gating
- risk-gated vault deposits
- verifiable oracle updates
- RWA transfer approval
- replay rejection

## Trust Models

### `ZKCGVerifier`

- Trust model: attestation-backed settlement today
- The verifier contract checks a signed ZKCG decision under the standard
  `verifyZKCG(proof, publicInputs)` interface
- The current proof-backed decision layer sits off-chain; contracts consume the
  signed settlement artifact
- Integrators can code to the interface now and keep that integration stable if
  backend-specific on-chain proof verifiers are added later

### Signed decision gate

- Trust model: semitrusted
- Contract verifies a signature from a configured ZKCG attestor
- Best for fastest integrations and partner pilots

## Standard Encoding

`publicInputs` is encoded as:

```solidity
abi.encode(
    decisionId,
    policyVersionHash,
    proofSystemHash,
    payloadHash,
    expiresAt
)
```

`proof` is the 65-byte Ethereum signature over:

```solidity
keccak256(
    abi.encode(
        DECISION_TYPEHASH,
        chainId,
        verifierAddress,
        decisionId,
        policyVersionHash,
        proofSystemHash,
        payloadHash,
        expiresAt
    )
)
```

`payloadHash` must bind the actual on-chain action. The shipped examples show
the recommended pattern:

- RWA transfer gating binds the gate contract plus the canonical `claims_hash` and
  `decision_commitment_hash` returned by the API
- loan gating binds the borrower, contract address, credit limit, and APR
- vault deposits bind the depositor, contract address, and deposit amount
- oracle updates bind the consumer contract, feed id, price, and expiry

## What This Repo Does Not Yet Ship

- Backend-specific Halo2 or zkVM verifier contracts
- A Solidity package or audit for the on-chain path
