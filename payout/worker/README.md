# zkcg-payout-worker

`zkcg-payout-worker` is the self-hosted bulk payout worker for ZKCG's Halo2 payout backend.

It is designed for frozen payout rounds where you want to:

- read an NDJSON payout manifest from disk
- bind the release decision to a recipient screening snapshot
- prove chunked payout-round summaries with Halo2 and package them into one release bundle
- verify the resulting proof through the generic verifier registry
- authorize release once and record replay protection in SQLite

## Commands

```bash
cargo run --release -p zkcg-payout-worker -- \
  prove-round \
  --manifest ./manifest.ndjson \
  --policy ./policy.json \
  --recipient-snapshot ./recipient-snapshot.json \
  --state-db ./payout.sqlite \
  --out ./out

cargo run --release -p zkcg-payout-worker -- \
  verify-round --proof ./out/proof.bin --claims ./out/claims.json

cargo run --release -p zkcg-payout-worker -- \
  authorize-release --proof ./out/proof.bin --claims ./out/claims.json --state-db ./payout.sqlite
```
