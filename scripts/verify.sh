#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

required_variables=(
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_URL
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_USER
  TEAQL_REGISTRY_SERVICE_CORE_DATABASE_PASSWORD
)

for variable_name in "${required_variables[@]}"; do
  if [[ -z "${!variable_name:-}" ]]; then
    echo "error: ${variable_name} must be set" >&2
    echo "hint: start PostgreSQL, then export the three database variables documented in README.md" >&2
    exit 2
  fi
done

export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"
export TOKIO_WORKER_STACK_SIZE="${TOKIO_WORKER_STACK_SIZE:-16777216}"

cd "${REPOSITORY_ROOT}"

echo "[1/5] Checking formatting"
cargo fmt -p teaql-registry -p registry-tui -- --check

echo "[2/5] Checking the generated domain library"
cargo check -p teaql-registry-service-core --all-targets

echo "[3/5] Linting application-owned code"
cargo clippy -p teaql-registry -p registry-tui --all-targets --no-deps -- -D warnings

echo "[4/5] Running the complete test suite serially"
cargo test --workspace -- --test-threads=1

echo "[5/5] Checking patch hygiene"
git diff --check

echo "TeaQL Registry verification passed"
