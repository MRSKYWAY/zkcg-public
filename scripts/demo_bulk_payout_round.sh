#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

MANIFEST="$TMP_DIR/manifest.ndjson"
POLICY="$TMP_DIR/policy.json"
SNAPSHOT="$TMP_DIR/recipient-snapshot.json"
STATE_DB="$TMP_DIR/payout.sqlite"
OUT_DIR="$TMP_DIR/out"

cat > "$MANIFEST" <<'EOF'
{"recipient_address":"0x1111111111111111111111111111111111111111","amount_units":10}
{"recipient_address":"0x2222222222222222222222222222222222222222","amount_units":25}
{"recipient_address":"0x3333333333333333333333333333333333333333","amount_units":40}
EOF

cat > "$POLICY" <<'EOF'
{
  "operator_id": "miner-a",
  "program_id": "pool-main",
  "asset_id": "btc",
  "round_id": "round-42",
  "round_cap_units": 1000,
  "per_recipient_cap_units": 100,
  "max_rows_per_round": 10,
  "max_chunks_per_round": 4,
  "round_nonce": 42,
  "release_window_ends_at": 4000000000
}
EOF

cat > "$SNAPSHOT" <<'EOF'
{
  "expires_at": 4000000000,
  "recipients": [
    {
      "recipient_address": "0x1111111111111111111111111111111111111111",
      "approved": true,
      "kyc_passed": true,
      "aml_cleared": true,
      "sanctions_clear": true
    },
    {
      "recipient_address": "0x2222222222222222222222222222222222222222",
      "approved": true,
      "kyc_passed": true,
      "aml_cleared": true,
      "sanctions_clear": true
    },
    {
      "recipient_address": "0x3333333333333333333333333333333333333333",
      "approved": true,
      "kyc_passed": true,
      "aml_cleared": true,
      "sanctions_clear": true
    }
  ]
}
EOF

cd "$ROOT_DIR"

echo "[halo2-payout-demo] proving payout round"
cargo run --release -p zkcg-payout-worker -- \
  prove-round --manifest "$MANIFEST" --policy "$POLICY" --recipient-snapshot "$SNAPSHOT" --state-db "$STATE_DB" --out "$OUT_DIR"

echo "[halo2-payout-demo] verifying proof"
cargo run --release -p zkcg-payout-worker -- \
  verify-round --proof "$OUT_DIR/proof.bin" --claims "$OUT_DIR/claims.json"

echo "[halo2-payout-demo] authorizing release"
cargo run --release -p zkcg-payout-worker -- \
  authorize-release --proof "$OUT_DIR/proof.bin" --claims "$OUT_DIR/claims.json" --state-db "$STATE_DB"
