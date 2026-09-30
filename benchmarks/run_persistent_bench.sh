#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REGISTRY_BIN="${REGISTRY_BIN:-${REPOSITORY_ROOT}/target/release/teaql-registry}"
REGISTRY_PORT="${REGISTRY_PORT:-18081}"
READY_TIMEOUT_SECONDS="${READY_TIMEOUT_SECONDS:-180}"
RUN_ID="${BENCH_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-persistent}"
RESULTS_DIR="${BENCH_RESULTS_DIR:-${SCRIPT_DIR}/results/${RUN_ID}}"
RUN_DIR="$(mktemp -d /tmp/teaql-registry-persistent-bench.XXXXXX)"
SERVER_LOG="${RESULTS_DIR}/server.txt"
MEMORY_LOG="${RESULTS_DIR}/memory.csv"
SERVER_PID=""
MONITOR_PID=""
SOURCE_GIT_COMMIT="$(git -C "${REPOSITORY_ROOT}" rev-parse HEAD)"
SOURCE_GIT_DIRTY="$(if [[ -z "$(git -C "${REPOSITORY_ROOT}" status --porcelain)" ]]; then echo false; else echo true; fi)"

required_variables=(
  ADMIN_PASSWORD
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_URL
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_USER
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_PASSWORD
  S3_ENDPOINT
  S3_ACCESS_KEY
  S3_SECRET_KEY
  S3_BUCKET
)

if [[ ! "${READY_TIMEOUT_SECONDS}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: READY_TIMEOUT_SECONDS must be a positive integer" >&2
  exit 2
fi

for variable_name in "${required_variables[@]}"; do
  if [[ -z "${!variable_name:-}" ]]; then
    echo "error: ${variable_name} must be set" >&2
    exit 2
  fi
done

if [[ ! -x "${REGISTRY_BIN}" ]]; then
  echo "error: release binary not found at ${REGISTRY_BIN}" >&2
  echo "hint: cargo build --release -p teaql-registry" >&2
  exit 2
fi

cleanup() {
  if [[ -n "${MONITOR_PID}" ]]; then kill "${MONITOR_PID}" 2>/dev/null || true; fi
  if [[ -n "${SERVER_PID}" ]]; then kill "${SERVER_PID}" 2>/dev/null || true; fi
  rm -rf "${RUN_DIR}"
}
trap cleanup EXIT INT TERM

mkdir -p "${RESULTS_DIR}"
export PORT="${REGISTRY_PORT}"
export CREDENTIALS_DIR="${RUN_DIR}/credentials"
export RUST_LOG="${RUST_LOG:-warn}"
export ALLOW_ANONYMOUS_READ=false
export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"
export TOKIO_WORKER_STACK_SIZE="${TOKIO_WORKER_STACK_SIZE:-16777216}"

"${REGISTRY_BIN}" >"${SERVER_LOG}" 2>&1 &
SERVER_PID=$!

for attempt in $(seq 1 "${READY_TIMEOUT_SECONDS}"); do
  if curl -fsS "http://127.0.0.1:${REGISTRY_PORT}/help" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
    echo "error: registry exited before becoming ready" >&2
    sed -n '1,160p' "${SERVER_LOG}" >&2
    exit 1
  fi
  if [[ "${attempt}" == "${READY_TIMEOUT_SECONDS}" ]]; then
    echo "error: registry did not become ready within ${READY_TIMEOUT_SECONDS} seconds" >&2
    exit 1
  fi
  sleep 1
done

echo "recorded_at_utc,rss_kib" >"${MEMORY_LOG}"
(
  while kill -0 "${SERVER_PID}" 2>/dev/null; do
    rss_kib="$(ps -o rss= -p "${SERVER_PID}" | tr -d ' ')"
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ),${rss_kib:-0}" >>"${MEMORY_LOG}"
    sleep 1
  done
) &
MONITOR_PID=$!

REGISTRY_URL="http://127.0.0.1:${REGISTRY_PORT}" \
  CREDENTIALS_DIR="${CREDENTIALS_DIR}" \
  SOURCE_GIT_COMMIT="${SOURCE_GIT_COMMIT}" \
  SOURCE_GIT_DIRTY="${SOURCE_GIT_DIRTY}" \
  BENCH_MODE=persistent-s3 \
  BENCH_RUN_ID="${RUN_ID}" \
  BENCH_RESULTS_DIR="${RESULTS_DIR}" \
  "${SCRIPT_DIR}/run_bench.sh"

echo "Persistent benchmark evidence written to ${RESULTS_DIR}"
