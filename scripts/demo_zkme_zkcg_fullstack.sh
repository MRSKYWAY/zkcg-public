#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${ZKCG_BASE_URL:-https://zkcg.onrender.com}"
ISSUER_ID="${ISSUER_ID:-zkme-kyc-integration}"
ASSET_ID="${ASSET_ID:-transfer-gate-v1}"
SENDER_WALLET="${SENDER_WALLET:-0x1111111111111111111111111111111111111111}"
USER_WALLET="${USER_WALLET:-0x2222222222222222222222222222222222222222}"
VERIFIER_CONTRACT_ADDRESS="${VERIFIER_CONTRACT_ADDRESS:-0x3333333333333333333333333333333333333333}"
TRANSFER_GATE_ADDRESS="${TRANSFER_GATE_ADDRESS:-0x4444444444444444444444444444444444444444}"
CHAIN_ID="${CHAIN_ID:-1}"
EXPIRES_AT="${EXPIRES_AT:-1900000000}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

zkme_status() {
  local is_grant="$1"
  local account="$2"
  local kyc_status="$3"
  local sanction="$4"

  jq -n \
    --argjson is_grant "$is_grant" \
    --arg account "$account" \
    --arg kyc_status "$kyc_status" \
    --argjson sanction "$sanction" \
    '{
      isGrant: $is_grant,
      associatedAccount: $account,
      kycStatus: $kyc_status,
      verifierValues: {
        sanction: $sanction
      }
    }'
}

zkme_to_zkcg_payload() {
  local status_json="$1"
  local policy_jurisdiction="$2"

  local wallet
  local kyc_passed
  local sanctions_clear
  wallet="$(jq -r '.associatedAccount' <<<"$status_json")"
  kyc_passed="$(jq -r '(.isGrant == true) and (.kycStatus == "KYC Passed")' <<<"$status_json")"
  sanctions_clear="$(jq -r '(.verifierValues.sanction == true)' <<<"$status_json")"

  jq -n \
    --arg issuer_id "$ISSUER_ID" \
    --arg asset_id "$ASSET_ID" \
    --arg sender_wallet "$SENDER_WALLET" \
    --arg receiver_wallet "$wallet" \
    --arg jurisdiction "$policy_jurisdiction" \
    --arg verifier_contract "$VERIFIER_CONTRACT_ADDRESS" \
    --arg transfer_gate "$TRANSFER_GATE_ADDRESS" \
    --argjson chain_id "$CHAIN_ID" \
    --argjson expires_at "$EXPIRES_AT" \
    --argjson kyc_passed "$kyc_passed" \
    --argjson sanctions_clear "$sanctions_clear" \
    '{
      issuer_id: $issuer_id,
      asset_id: $asset_id,
      sender_wallet_address: $sender_wallet,
      receiver_wallet_address: $receiver_wallet,
      receiver_investor_type: "individual",
      receiver_accredited: true,
      receiver_kyc_passed: $kyc_passed,
      receiver_aml_cleared: $sanctions_clear,
      receiver_sanctions_clear: $sanctions_clear,
      receiver_jurisdiction: $jurisdiction,
      allowed_jurisdictions: ["SG", "IN", "DE", "AE"],
      blocked_jurisdictions: ["US", "KP", "IR"],
      receiver_residency_allowed: true,
      holding_period_met: true,
      transfer_amount_units: 100,
      post_transfer_position_units: 500,
      wallet_position_limit_units: 1000,
      post_transfer_concentration_bps: 1500,
      concentration_limit_bps: 2000,
      expires_at: $expires_at,
      chain_id: $chain_id,
      verifier_contract_address: $verifier_contract,
      transfer_gate_address: $transfer_gate
    }'
}

post_json() {
  local endpoint="$1"
  local payload="$2"
  local body_file
  body_file="$(mktemp)"

  local status
  status="$(
    curl -sS \
      -o "$body_file" \
      -w "%{http_code}" \
      -X POST "${BASE_URL%/}${endpoint}" \
      -H 'content-type: application/json' \
      -d "$payload" || true
  )"

  local body
  body="$(cat "$body_file")"
  rm -f "$body_file"

  if [[ "$status" -lt 200 || "$status" -ge 300 ]]; then
    printf '%s\n' "$status"
    printf '%s\n' "$body"
    return 1
  fi

  printf '%s\n' "$status"
  printf '%s\n' "$body"
}

run_case() {
  local label="$1"
  local status_json="$2"
  local policy_jurisdiction="$3"

  local payload
  payload="$(zkme_to_zkcg_payload "$status_json" "$policy_jurisdiction")"

  local endpoint="/v1/rwa/transfer/attest"
  local result
  if ! result="$(post_json "$endpoint" "$payload")"; then
    local http_status
    local error_body
    http_status="$(sed -n '1p' <<<"$result")"
    error_body="$(sed -n '2,$p' <<<"$result")"
    if [[ "$http_status" == "503" && "$error_body" == *"attestation signer unavailable"* ]]; then
      endpoint="/v1/rwa/transfer/evaluate"
      payload="$(jq 'del(.chain_id, .verifier_contract_address, .transfer_gate_address)' <<<"$payload")"
      result="$(post_json "$endpoint" "$payload")"
    else
      echo "case ${label} failed against ${endpoint}: ${error_body}" >&2
      exit 1
    fi
  fi

  local response
  response="$(sed -n '2,$p' <<<"$result")"

  jq -n \
    --arg case "$label" \
    --arg endpoint "$endpoint" \
    --argjson zkme "$status_json" \
    --argjson response "$response" \
    '{
      case: $case,
      endpoint: $endpoint,
      zkme_input: {
        isGrant: $zkme.isGrant,
        associatedAccount: $zkme.associatedAccount,
        kycStatus: $zkme.kycStatus,
        sanction: $zkme.verifierValues.sanction
      },
      zkcg_output: {
        decision: $response.decision,
        policy_passed: $response.policy_passed,
        proof_verified: $response.proof_verified,
        proof_system: $response.proof_system,
        reason_codes: ($response.reason_codes // []),
        claims_hash: $response.claims_hash,
        decision_commitment_hash: $response.decision_commitment_hash,
        payload_hash: $response.payload_hash
      }
    }'
}

require_command curl
require_command jq

echo "[zkMe -> ZKCG] base_url=${BASE_URL}"
echo

eligible_status="$(zkme_status true "$USER_WALLET" "KYC Passed" true)"
blocked_jurisdiction_status="$(zkme_status true "$USER_WALLET" "KYC Passed" true)"
sanctions_failure_status="$(zkme_status true "$USER_WALLET" "KYC Passed" false)"

run_case "eligible_credential" "$eligible_status" "SG"
echo
run_case "blocked_jurisdiction" "$blocked_jurisdiction_status" "US"
echo
run_case "sanctions_failure" "$sanctions_failure_status" "SG"
