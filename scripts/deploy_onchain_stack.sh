#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORGE_BIN="${FORGE_BIN:-$HOME/.foundry/bin/forge}"
CAST_BIN="${CAST_BIN:-$HOME/.foundry/bin/cast}"
RPC_URL="${RPC_URL:-http://127.0.0.1:8545}"
DEPLOYER_PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
ATTESTOR_PRIVATE_KEY="${ATTESTOR_PRIVATE_KEY:-0x59c6995e998f97a5a0044976f7d8f2f6f57f0e8f4b0fba4b2d6538aacbb44c41}"

if [[ ! -x "$FORGE_BIN" ]]; then
  echo "forge not found at $FORGE_BIN" >&2
  exit 1
fi

if [[ ! -x "$CAST_BIN" ]]; then
  echo "cast not found at $CAST_BIN" >&2
  exit 1
fi

extract_address() {
  awk '/Deployed to:/ {print $3}'
}

ATTESTOR_ADDRESS="$("$CAST_BIN" wallet address --private-key "$ATTESTOR_PRIVATE_KEY")"

deploy_contract() {
  local contract_path="$1"
  shift
  "$FORGE_BIN" create "$contract_path" \
    --root "$ROOT_DIR" \
    --rpc-url "$RPC_URL" \
    --private-key "$DEPLOYER_PRIVATE_KEY" \
    --broadcast \
    "$@" | extract_address
}

VERIFIER_ADDRESS="$(deploy_contract contracts/ZKCGVerifier.sol:ZKCGVerifier --constructor-args "$ATTESTOR_ADDRESS")"
LOAN_GATE_ADDRESS="$(deploy_contract contracts/examples/PrivateLoanEligibilityGate.sol:PrivateLoanEligibilityGate --constructor-args "$VERIFIER_ADDRESS")"
VAULT_ADDRESS="$(deploy_contract contracts/examples/RiskGatedVault.sol:RiskGatedVault --constructor-args "$VERIFIER_ADDRESS")"
ORACLE_ADDRESS="$(deploy_contract contracts/examples/VerifiableOracleWithPolicy.sol:VerifiableOracleWithPolicy --constructor-args "$VERIFIER_ADDRESS")"
RWA_TRANSFER_GATE_ADDRESS="$(deploy_contract contracts/examples/RwaTransferGate.sol:RwaTransferGate --constructor-args "$VERIFIER_ADDRESS")"

jq -n \
  --arg rpc_url "$RPC_URL" \
  --arg verifier "$VERIFIER_ADDRESS" \
  --arg loan_gate "$LOAN_GATE_ADDRESS" \
  --arg vault "$VAULT_ADDRESS" \
  --arg oracle "$ORACLE_ADDRESS" \
  --arg rwa_transfer_gate "$RWA_TRANSFER_GATE_ADDRESS" \
  --arg attestor "$ATTESTOR_ADDRESS" \
  '{
    rpc_url: $rpc_url,
    verifier_address: $verifier,
    private_loan_gate_address: $loan_gate,
    risk_gated_vault_address: $vault,
    oracle_address: $oracle,
    rwa_transfer_gate_address: $rwa_transfer_gate,
    attestor_address: $attestor
  }'
