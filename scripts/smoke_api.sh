#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:8080}"

echo "[smoke] health"
curl -fsS "${BASE_URL}/healthz"
echo

echo "[smoke] protocol state"
curl -fsS "${BASE_URL}/v1/protocol/state"
echo

echo "[smoke] compliance evaluate"
curl -fsS -X POST "${BASE_URL}/v1/compliance/evaluate" \
  -H 'content-type: application/json' \
  -d '{
    "applicant_id":"smoke_app_001",
    "risk_score":35,
    "threshold":50,
    "monthly_income_cents":500000,
    "monthly_debt_cents":150000,
    "requested_credit_cents":1000000
  }'
echo
