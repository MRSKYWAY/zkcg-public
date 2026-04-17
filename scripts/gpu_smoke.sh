#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/zkcg-gpu-target}"
ZKCG_GPU_TOOLCHAIN="${ZKCG_GPU_TOOLCHAIN:-nightly}"

if ! rustup toolchain list | grep -q "^${ZKCG_GPU_TOOLCHAIN}"; then
  echo "[gpu] Rust toolchain '${ZKCG_GPU_TOOLCHAIN}' is not installed"
  echo "[gpu] install it with: rustup toolchain install ${ZKCG_GPU_TOOLCHAIN}"
  exit 1
fi

echo "[gpu] checking local CUDA visibility"
if ! nvidia-smi >/dev/null 2>&1; then
  echo "[gpu] nvidia-smi is unavailable or GPU access is blocked"
  echo "[gpu] the ICICLE backend is wired in, but this machine cannot validate runtime GPU access"
fi

if ! command -v nvcc >/dev/null 2>&1; then
  echo "[gpu] nvcc not found"
  echo "[gpu] install the CUDA toolkit and optionally set CUDAToolkit_ROOT"
  exit 1
fi

echo "[gpu] nvcc detected at $(command -v nvcc)"
if [[ -n "${CUDAToolkit_ROOT:-}" ]]; then
  echo "[gpu] using CUDAToolkit_ROOT=${CUDAToolkit_ROOT}"
fi
echo "[gpu] using Rust toolchain ${ZKCG_GPU_TOOLCHAIN}"

echo "[gpu] compiling the ICICLE-enabled prover"
(
  cd "$ROOT_DIR"
  env CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo +"$ZKCG_GPU_TOOLCHAIN" check -p zkcg-halo2-prover --no-default-features --features icicle-gpu
)

if [[ "${ZKCG_GPU_RUN:-0}" == "1" ]]; then
  echo "[gpu] running the ICICLE-enabled prover binary"
  (
    cd "$ROOT_DIR"
    env CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo +"$ZKCG_GPU_TOOLCHAIN" run -p zkcg-halo2-prover --no-default-features --features icicle-gpu
  )
else
  echo "[gpu] compile check passed"
  echo "[gpu] set ZKCG_GPU_RUN=1 to run the prover binary after a successful compile"
fi
