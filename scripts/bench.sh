#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-}"

if [ -z "$MODE" ]; then
  echo "Usage: bench.sh [halo2|zkvm|halo2-cpu|halo2-gpu|halo2-payout]"
  exit 1
fi

echo "=== ZKCG Benchmarks ==="
echo "Mode: $MODE"
echo "CPU info:"
lscpu | head -n 20
if [ "$MODE" = "halo2-gpu" ] && command -v nvidia-smi >/dev/null 2>&1; then
  echo "GPU info:"
  nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
fi
echo "======================"

if [ "$MODE" = "halo2" ]; then
  cd /home/skye/ZKCG/verifier
  cargo bench --features zk-halo2 --bench halo2_verify
elif [ "$MODE" = "halo2-cpu" ]; then
  cd /home/skye/ZKCG
  cargo bench -p zkcg-halo2-prover --bench halo2_prove
elif [ "$MODE" = "zkvm" ]; then
  cd /home/skye/ZKCG/verifier
  cargo bench --features zk-vm --bench zkvm_verify
elif [ "$MODE" = "halo2-gpu" ]; then
  cd /home/skye/ZKCG
  cargo +nightly bench -p zkcg-halo2-prover --no-default-features --features icicle-gpu --bench halo2_prove
elif [ "$MODE" = "halo2-payout" ]; then
  cd /home/skye/ZKCG
  cargo bench -p zkcg-halo2-prover --bench halo2_payout_chunk -- --noplot
else
  echo "Unknown mode: $MODE"
  exit 1
fi
