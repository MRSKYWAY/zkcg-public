#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${IMAGE_NAME:-zkcg-api-smoke}"
CONTAINER_NAME="${CONTAINER_NAME:-zkcg-api-smoke}"
STATE_DIR="${STATE_DIR:-$(pwd)/.tmp/docker-state}"

mkdir -p "${STATE_DIR}"

docker build -t "${IMAGE_NAME}" -f Dockerfile .
docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true

docker run -d \
  --name "${CONTAINER_NAME}" \
  -p 8080:8080 \
  -v "${STATE_DIR}:/data" \
  "${IMAGE_NAME}" >/dev/null

cleanup() {
  docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:8080/healthz >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

"$(dirname "$0")/smoke_api.sh" http://127.0.0.1:8080
