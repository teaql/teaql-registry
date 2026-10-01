#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="${VERIFICATION_REPOSITORY_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
RUN_ID="${VERIFICATION_RUN_ID:-$(date -u +%Y%m%d%H%M%S)}"
RESULTS_DIR="${VERIFICATION_RESULTS_DIR:-${REPOSITORY_ROOT}/evidence/verification/${RUN_ID}}"

if [[ -e "${RESULTS_DIR}" ]]; then
  echo "error: verification evidence already exists: ${RESULTS_DIR}" >&2
  echo "hint: choose a new VERIFICATION_RUN_ID or VERIFICATION_RESULTS_DIR" >&2
  exit 2
fi

TRACKED_DIFF_SHA256="$(git -C "${REPOSITORY_ROOT}" diff --binary HEAD | sha256sum | awk '{print $1}')"
UNTRACKED_MANIFEST_SHA256="$({
  git -C "${REPOSITORY_ROOT}" ls-files --others --exclude-standard -z |
    while IFS= read -r -d '' relative_path; do
      sha256sum "${REPOSITORY_ROOT}/${relative_path}"
    done
} | LC_ALL=C sort | sha256sum | awk '{print $1}')"

mkdir -p "${RESULTS_DIR}"
LOG_FILE="${RESULTS_DIR}/verify.log"
SOURCE_GIT_COMMIT="$(git -C "${REPOSITORY_ROOT}" rev-parse HEAD)"
SOURCE_GIT_DIRTY="$(if [[ -z "$(git -C "${REPOSITORY_ROOT}" status --porcelain)" ]]; then echo false; else echo true; fi)"

first_line() {
  "$@" 2>&1 | head -n 1
}

{
  echo "run_id=${RUN_ID}"
  echo "recorded_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "git_commit=${SOURCE_GIT_COMMIT}"
  echo "git_dirty=${SOURCE_GIT_DIRTY}"
  echo "tracked_diff_sha256=${TRACKED_DIFF_SHA256}"
  echo "untracked_manifest_sha256=${UNTRACKED_MANIFEST_SHA256}"
  echo "platform=$(uname -srvmo)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  if cargo teaql --version >/dev/null 2>&1; then
    echo "cargo_teaql=$(first_line cargo teaql --version)"
  else
    echo "cargo_teaql=not installed"
  fi
  echo "postgres_server=${VERIFICATION_POSTGRES_VERSION:-not recorded; set VERIFICATION_POSTGRES_VERSION}"
} >"${RESULTS_DIR}/environment.txt"

cd "${REPOSITORY_ROOT}"
cargo tree -p teaql-registry --depth 1 >"${RESULTS_DIR}/direct-dependencies.txt"

set +e
set -o pipefail
"${REPOSITORY_ROOT}/scripts/verify.sh" 2>&1 | tee "${LOG_FILE}"
VERIFY_STATUS=${PIPESTATUS[0]}
set +o pipefail
set -e

PASSED_TESTS="$({
  grep -Eo 'test result: ok\. [0-9]+ passed' "${LOG_FILE}" || true
} | awk '{ total += $4 } END { print total + 0 }')"
FAILED_TESTS="$({
  grep -Eo 'test result: (ok|FAILED)\. [0-9]+ passed; [0-9]+ failed' "${LOG_FILE}" || true
} | awk '{ total += $6 } END { print total + 0 }')"
if [[ "${VERIFY_STATUS}" -eq 0 ]]; then
  RESULT=PASS
else
  RESULT=FAIL
fi

cat >"${RESULTS_DIR}/summary.md" <<EOF
# Workspace verification ${RUN_ID}

- Result: **${RESULT}**
- Source commit: \`${SOURCE_GIT_COMMIT}\`
- Source dirty before verification: \`${SOURCE_GIT_DIRTY}\`
- Recorded at: \`$(date -u +%Y-%m-%dT%H:%M:%SZ)\`
- Tests reported: **${PASSED_TESTS} passed, ${FAILED_TESTS} failed**
- Verification gates: formatting, generated library check, strict application
  Clippy, serial workspace tests, and patch hygiene

See \`environment.txt\`, \`direct-dependencies.txt\`, and \`verify.log\` in this
directory for the machine-readable context and complete command output. Database
credentials and the database URL are deliberately not retained.
EOF

echo "Verification evidence written to ${RESULTS_DIR} (${RESULT})"
exit "${VERIFY_STATUS}"
