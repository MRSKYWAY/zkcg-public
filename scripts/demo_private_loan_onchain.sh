#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORGE_BIN="${FORGE_BIN:-$HOME/.foundry/bin/forge}"
CAST_BIN="${CAST_BIN:-$HOME/.foundry/bin/cast}"
ANVIL_BIN="${ANVIL_BIN:-$HOME/.foundry/bin/anvil}"
DEPLOYER_PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
ATTESTOR_PRIVATE_KEY="${ATTESTOR_PRIVATE_KEY:-0x59c6995e998f97a5a0044976f7d8f2f6f57f0e8f4b0fba4b2d6538aacbb44c41}"
BORROWER_ADDRESS="${BORROWER_ADDRESS:-0x3333333333333333333333333333333333333333}"
CREDIT_LIMIT_CENTS="${CREDIT_LIMIT_CENTS:-1000000}"
MAX_APR_BPS="${MAX_APR_BPS:-1500}"
ANVIL_URL="${ANVIL_URL:-http://127.0.0.1:8545}"
API_PORT="${API_PORT:-8081}"
API_URL="${API_URL:-http://127.0.0.1:${API_PORT}}"
EXPIRES_AT="${EXPIRES_AT:-1900000000}"
ANVIL_LOG="${ANVIL_LOG:-/tmp/zkcg-anvil.log}"
API_LOG="${API_LOG:-/tmp/zkcg-api-onchain.log}"
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
    ZKCG_STATE_PATH=/tmp/zkcg-demo-onchain.db \
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
LOAN_GATE_ADDRESS="$(echo "$STACK_JSON" | jq -r '.private_loan_gate_address')"

ATTESTATION_JSON="$(curl -sS -X POST "${API_URL}/v1/onchain/private-loan-eligibility/attest" \
  -H 'content-type: application/json' \
  -d "{
    \"applicant_id\":\"demo_borrower_001\",
    \"risk_score\":35,
    \"threshold\":50,
    \"monthly_income_cents\":500000,
    \"monthly_debt_cents\":150000,
    \"requested_credit_cents\":1000000,
    \"chain_id\":31337,
    \"verifier_contract_address\":\"${VERIFIER_ADDRESS}\",
    \"credit_line_address\":\"${LOAN_GATE_ADDRESS}\",
    \"borrower_address\":\"${BORROWER_ADDRESS}\",
    \"credit_limit_cents\":${CREDIT_LIMIT_CENTS},
    \"max_apr_bps\":${MAX_APR_BPS},
    \"expires_at\":${EXPIRES_AT}
  }")"

PROOF_HEX="$(echo "$ATTESTATION_JSON" | jq -r '.proof')"
PUBLIC_INPUTS_HEX="$(echo "$ATTESTATION_JSON" | jq -r '.public_inputs')"
DECISION_ID_HASH="$(echo "$ATTESTATION_JSON" | jq -r '.decision_id_hash')"

"$CAST_BIN" send "$LOAN_GATE_ADDRESS" \
  "approveBorrower(bytes,bytes,address,uint256,uint256)" \
  "$PROOF_HEX" \
  "$PUBLIC_INPUTS_HEX" \
  "$BORROWER_ADDRESS" \
  "$CREDIT_LIMIT_CENTS" \
  "$MAX_APR_BPS" \
  --rpc-url "$ANVIL_URL" \
  --private-key "$DEPLOYER_PRIVATE_KEY" >/dev/null

APPROVAL="$( "$CAST_BIN" call "$LOAN_GATE_ADDRESS" \
  "approvals(address)(uint256,uint256,bytes32)" \
  "$BORROWER_ADDRESS" \
  --rpc-url "$ANVIL_URL")"

jq -n \
  --arg verifier "$VERIFIER_ADDRESS" \
  --arg loan_gate "$LOAN_GATE_ADDRESS" \
  --arg borrower "$BORROWER_ADDRESS" \
  --arg decision_id_hash "$DECISION_ID_HASH" \
  --arg approval "$APPROVAL" \
  '{
    verifier_address: $verifier,
    private_loan_gate_address: $loan_gate,
    borrower_address: $borrower,
    decision_id_hash: $decision_id_hash,
    approval_tuple: $approval,
    note: "private loan decision accepted on-chain"
  }'
