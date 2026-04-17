#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CAST_BIN="${CAST_BIN:-$HOME/.foundry/bin/cast}"
ANVIL_BIN="${ANVIL_BIN:-$HOME/.foundry/bin/anvil}"
DEPLOYER_PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
ATTESTOR_PRIVATE_KEY="${ATTESTOR_PRIVATE_KEY:-0x59c6995e998f97a5a0044976f7d8f2f6f57f0e8f4b0fba4b2d6538aacbb44c41}"
SENDER_ADDRESS="${SENDER_ADDRESS:-0x1111111111111111111111111111111111111111}"
RECEIVER_ADDRESS="${RECEIVER_ADDRESS:-0x2222222222222222222222222222222222222222}"
ISSUER_ID="${ISSUER_ID:-issuer-a}"
ASSET_ID="${ASSET_ID:-credit-fund-a}"
TRANSFER_AMOUNT_UNITS="${TRANSFER_AMOUNT_UNITS:-100}"
ANVIL_URL="${ANVIL_URL:-http://127.0.0.1:8545}"
API_PORT="${API_PORT:-8082}"
API_URL="${API_URL:-http://127.0.0.1:${API_PORT}}"
EXPIRES_AT="${EXPIRES_AT:-1900000000}"
ANVIL_LOG="${ANVIL_LOG:-/tmp/zkcg-anvil-rwa.log}"
API_LOG="${API_LOG:-/tmp/zkcg-api-rwa-onchain.log}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/zkcg-target}"

cleanup() {
  if [[ -n "${API_PID:-}" ]]; then
    kill "$API_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${ANVIL_PID:-}" ]]; then
    kill "$ANVIL_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

wait_for_http() {
  local url="$1"
  local attempts="${2:-30}"
  for _ in $(seq 1 "$attempts"); do
    if curl -sS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

if [[ -z "${ZKCG_SKIP_ANVIL_START:-}" ]]; then
  "$ANVIL_BIN" --host 127.0.0.1 --port 8545 >"$ANVIL_LOG" 2>&1 &
  ANVIL_PID="$!"
  sleep 1
fi

if [[ -z "${ZKCG_SKIP_API_START:-}" ]]; then
  env \
    CARGO_TARGET_DIR="$CARGO_TARGET_DIR" \
    PORT="$API_PORT" \
    ZKCG_ENABLE_PROTOCOL=1 \
    ZKCG_STATE_BACKEND=sqlite \
    ZKCG_STATE_PATH=/tmp/zkcg-rwa-demo.db \
    ZKCG_ATTESTATION_PRIVATE_KEY="$ATTESTOR_PRIVATE_KEY" \
    cargo run -p api --features "zk-vm" >"$API_LOG" 2>&1 &
  API_PID="$!"
  wait_for_http "${API_URL}/healthz" 60
fi

STACK_JSON="$(
  env \
    RPC_URL="$ANVIL_URL" \
    DEPLOYER_PRIVATE_KEY="$DEPLOYER_PRIVATE_KEY" \
    ATTESTOR_PRIVATE_KEY="$ATTESTOR_PRIVATE_KEY" \
    "$ROOT_DIR/scripts/deploy_onchain_stack.sh"
)"

VERIFIER_ADDRESS="$(echo "$STACK_JSON" | jq -r '.verifier_address')"
RWA_TRANSFER_GATE_ADDRESS="$(echo "$STACK_JSON" | jq -r '.rwa_transfer_gate_address')"

ATTESTATION_JSON="$(curl -sS -X POST "${API_URL}/v1/rwa/transfer/attest" \
  -H 'content-type: application/json' \
  -d "{
    \"issuer_id\":\"${ISSUER_ID}\",
    \"asset_id\":\"${ASSET_ID}\",
    \"sender_wallet_address\":\"${SENDER_ADDRESS}\",
    \"receiver_wallet_address\":\"${RECEIVER_ADDRESS}\",
    \"receiver_investor_type\":\"individual\",
    \"receiver_accredited\":true,
    \"receiver_kyc_passed\":true,
    \"receiver_aml_cleared\":true,
    \"receiver_sanctions_clear\":true,
    \"receiver_jurisdiction\":\"US\",
    \"allowed_jurisdictions\":[\"US\"],
    \"blocked_jurisdictions\":[],
    \"receiver_residency_allowed\":true,
    \"holding_period_met\":true,
    \"transfer_amount_units\":${TRANSFER_AMOUNT_UNITS},
    \"post_transfer_position_units\":500,
    \"wallet_position_limit_units\":1000,
    \"post_transfer_concentration_bps\":1500,
    \"concentration_limit_bps\":2000,
    \"expires_at\":${EXPIRES_AT},
    \"chain_id\":31337,
    \"verifier_contract_address\":\"${VERIFIER_ADDRESS}\",
    \"transfer_gate_address\":\"${RWA_TRANSFER_GATE_ADDRESS}\"
  }")"

PROOF_HEX="$(echo "$ATTESTATION_JSON" | jq -r '.proof')"
PUBLIC_INPUTS_HEX="$(echo "$ATTESTATION_JSON" | jq -r '.public_inputs')"
DECISION_ID_HASH="$(echo "$ATTESTATION_JSON" | jq -r '.decision_id_hash')"
CLAIMS_HASH="$(echo "$ATTESTATION_JSON" | jq -r '.claims_hash')"
DECISION_COMMITMENT_HASH="$(echo "$ATTESTATION_JSON" | jq -r '.decision_commitment_hash')"

"$CAST_BIN" send "$RWA_TRANSFER_GATE_ADDRESS" \
  "approveTransfer(bytes,bytes,bytes32,bytes32)" \
  "$PROOF_HEX" \
  "$PUBLIC_INPUTS_HEX" \
  "$CLAIMS_HASH" \
  "$DECISION_COMMITMENT_HASH" \
  --rpc-url "$ANVIL_URL" \
  --private-key "$DEPLOYER_PRIVATE_KEY" >/dev/null

TRANSFER_KEY="$("$CAST_BIN" call "$RWA_TRANSFER_GATE_ADDRESS" \
  "transferKeyFor(bytes32,bytes32)(bytes32)" \
  "$CLAIMS_HASH" \
  "$DECISION_COMMITMENT_HASH" \
  --rpc-url "$ANVIL_URL")"

APPROVED_DECISION_ID="$("$CAST_BIN" call "$RWA_TRANSFER_GATE_ADDRESS" \
  "approvedDecisionId(bytes32)(bytes32)" \
  "$TRANSFER_KEY" \
  --rpc-url "$ANVIL_URL")"

jq -n \
  --arg verifier "$VERIFIER_ADDRESS" \
  --arg rwa_transfer_gate "$RWA_TRANSFER_GATE_ADDRESS" \
  --arg issuer_id "$ISSUER_ID" \
  --arg asset_id "$ASSET_ID" \
  --arg transfer_key "$TRANSFER_KEY" \
  --arg approved_decision_id "$APPROVED_DECISION_ID" \
  --arg decision_id_hash "$DECISION_ID_HASH" \
  --arg claims_hash "$CLAIMS_HASH" \
  --arg decision_commitment_hash "$DECISION_COMMITMENT_HASH" \
  '{
    verifier_address: $verifier,
    rwa_transfer_gate_address: $rwa_transfer_gate,
    issuer_id: $issuer_id,
    asset_id: $asset_id,
    transfer_key: $transfer_key,
    approved_decision_id: $approved_decision_id,
    decision_id_hash: $decision_id_hash,
    claims_hash: $claims_hash,
    decision_commitment_hash: $decision_commitment_hash,
    note: "rwa transfer decision accepted on-chain"
  }'
