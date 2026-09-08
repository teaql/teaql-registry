#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REGISTRY_URL="${REGISTRY_URL:-http://127.0.0.1:8081}"
CREDENTIALS_DIR="${CREDENTIALS_DIR:-/var/lib/teaql-registry/credentials}"
BENCH_ITERATIONS="${BENCH_ITERATIONS:-10}"
BENCH_MODE="${BENCH_MODE:-unspecified}"
RUN_ID="${BENCH_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RESULTS_DIR="${BENCH_RESULTS_DIR:-${SCRIPT_DIR}/results/${RUN_ID}}"
PAYLOAD_DIR="$(mktemp -d /tmp/teaql-registry-bench.XXXXXX)"
RESPONSE_FILE="${PAYLOAD_DIR}/response.txt"

cleanup() {
  rm -rf "${PAYLOAD_DIR}"
}
trap cleanup EXIT

if [[ ! "${BENCH_ITERATIONS}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: BENCH_ITERATIONS must be a positive integer" >&2
  exit 2
fi

if [[ -f "${CREDENTIALS_DIR}/global-admin.txt" ]]; then
  ADMIN_PASSWORD_VALUE="$(sed -n 's/^admin \/ //p' "${CREDENTIALS_DIR}/global-admin.txt" | head -n 1)"
else
  : "${ADMIN_PASSWORD:?Set ADMIN_PASSWORD or provide ${CREDENTIALS_DIR}/global-admin.txt}"
  ADMIN_PASSWORD_VALUE="${ADMIN_PASSWORD}"
fi

if [[ -z "${ADMIN_PASSWORD_VALUE}" ]]; then
  echo "error: administrator credential is empty" >&2
  exit 2
fi

mkdir -p "${RESULTS_DIR}"
RAW_RESULTS="${RESULTS_DIR}/samples.csv"
SUMMARY_RESULTS="${RESULTS_DIR}/summary.csv"
AUTH=(--user "admin:${ADMIN_PASSWORD_VALUE}")

curl -fsS "${AUTH[@]}" "${REGISTRY_URL}/help" >/dev/null

generate_payload() {
  local output_path="$1"
  local byte_count="$2"
  head -c "${byte_count}" /dev/urandom >"${output_path}"
}

generate_payload "${PAYLOAD_DIR}/maven.bin" 5242880
generate_payload "${PAYLOAD_DIR}/docker.bin" 52428800
generate_payload "${PAYLOAD_DIR}/npm.bin" 1048576
generate_payload "${PAYLOAD_DIR}/pypi.bin" 3145728
generate_payload "${PAYLOAD_DIR}/cargo.bin" 1048576
generate_payload "${PAYLOAD_DIR}/gomod.bin" 524288
generate_payload "${PAYLOAD_DIR}/nuget.bin" 1048576
generate_payload "${PAYLOAD_DIR}/raw.bin" 10485760

NPM_PACKAGE="bench-perf-${RUN_ID}"
NPM_TARBALL="${NPM_PACKAGE}-0.0.1.tgz"
NPM_SHASUM="$(sha1sum "${PAYLOAD_DIR}/npm.bin" | awk '{print $1}')"
{
  printf '{"_id":"%s","name":"%s","dist-tags":{"latest":"0.0.1"},"versions":{"0.0.1":{"name":"%s","version":"0.0.1","dist":{"shasum":"%s","tarball":"%s"}}},"_attachments":{"%s":{"contentType":"application/gzip","data":"' "${NPM_PACKAGE}" "${NPM_PACKAGE}" "${NPM_PACKAGE}" "${NPM_SHASUM}" "${NPM_TARBALL}" "${NPM_TARBALL}"
  base64 -w 0 "${PAYLOAD_DIR}/npm.bin"
  printf '","length":1048576}}}'
} >"${PAYLOAD_DIR}/npm-publish.json"

CARGO_PACKAGE="bench-perf-${RUN_ID}"
CARGO_METADATA="{\"name\":\"${CARGO_PACKAGE}\",\"vers\":\"0.1.0\",\"deps\":[],\"features\":{},\"authors\":[\"TeaQL benchmark\"],\"description\":\"Registry protocol benchmark\"}"
perl -e '
  use strict; use warnings;
  my ($metadata, $input, $output) = @ARGV;
  open my $in, "<:raw", $input or die $!;
  local $/; my $crate = <$in>;
  open my $out, ">:raw", $output or die $!;
  print {$out} pack("V", length($metadata)), $metadata, pack("V", length($crate)), $crate;
' "${CARGO_METADATA}" "${PAYLOAD_DIR}/cargo.bin" "${PAYLOAD_DIR}/cargo-publish.bin"

{
  echo "run_id=${RUN_ID}"
  echo "recorded_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "git_commit=$(git -C "${REPOSITORY_ROOT}" rev-parse HEAD)"
  echo "git_dirty=$(if [[ -z "$(git -C "${REPOSITORY_ROOT}" status --porcelain)" ]]; then echo false; else echo true; fi)"
  echo "registry_url=${REGISTRY_URL}"
  echo "mode=${BENCH_MODE}"
  echo "iterations=${BENCH_ITERATIONS}"
  echo "kernel=$(uname -srmo)"
  echo "cpu=$(awk -F: '/model name/{gsub(/^[ \t]+/, "", $2); print $2; exit}' /proc/cpuinfo)"
  echo "curl=$(curl --version | head -n 1)"
  for payload in "${PAYLOAD_DIR}"/*.bin; do
    echo "payload.$(basename "${payload}").sha256=$(sha256sum "${payload}" | awk '{print $1}')"
  done
} >"${RESULTS_DIR}/environment.txt"

echo "format,operation,size_bytes,iteration,http_status,seconds" >"${RAW_RESULTS}"
echo "format,operation,size_bytes,iterations,avg_ms,min_ms,max_ms,throughput_mbps" >"${SUMMARY_RESULTS}"

benchmark_request() {
  local format="$1"
  local operation="$2"
  local size_bytes="$3"
  shift 3
  local sample_file="${PAYLOAD_DIR}/samples-${format}-${operation}.csv"
  : >"${sample_file}"

  for iteration in $(seq 1 "${BENCH_ITERATIONS}"); do
    local args=()
    local argument
    for argument in "$@"; do
      args+=("${argument//__ITER__/${iteration}}")
    done

    local measurement
    measurement="$(curl -sS -o "${RESPONSE_FILE}" -w '%{http_code},%{time_total}' "${AUTH[@]}" "${args[@]}")"
    local status="${measurement%%,*}"
    local seconds="${measurement#*,}"
    if [[ ! "${status}" =~ ^2[0-9][0-9]$ ]]; then
      echo "error: ${format} ${operation} iteration ${iteration} returned HTTP ${status}" >&2
      sed -n '1,20p' "${RESPONSE_FILE}" >&2
      exit 1
    fi

    echo "${iteration},${status},${seconds}" >>"${sample_file}"
    echo "${format},${operation},${size_bytes},${iteration},${status},${seconds}" >>"${RAW_RESULTS}"
  done

  local statistics
  statistics="$(awk -F, -v size="${size_bytes}" '
    NR == 1 { min = $3; max = $3 }
    { total += $3; if ($3 < min) min = $3; if ($3 > max) max = $3 }
    END {
      avg = total / NR;
      mbps = avg > 0 ? (size * 8 / 1000000) / avg : 0;
      printf "%.3f,%.3f,%.3f,%.1f", avg * 1000, min * 1000, max * 1000, mbps
    }
  ' "${sample_file}")"
  echo "${format},${operation},${size_bytes},${BENCH_ITERATIONS},${statistics}" >>"${SUMMARY_RESULTS}"
  printf '%-8s %-8s %s\n' "${format}" "${operation}" "${statistics}"
}

MAVEN_PATH="com/bench/perf/${RUN_ID}.0.__ITER__/perf-${RUN_ID}.0.__ITER__.jar"
benchmark_request Maven upload 5242880 -X PUT "${REGISTRY_URL}/repository/maven-releases/${MAVEN_PATH}" -H "Content-Type: application/java-archive" --data-binary "@${PAYLOAD_DIR}/maven.bin"
benchmark_request Maven download 5242880 "${REGISTRY_URL}/repository/maven-releases/com/bench/perf/${RUN_ID}.0.1/perf-${RUN_ID}.0.1.jar"

DOCKER_DIGEST="sha256:$(sha256sum "${PAYLOAD_DIR}/docker.bin" | awk '{print $1}')"
benchmark_request Docker upload 52428800 -X POST "${REGISTRY_URL}/v2/bench-${RUN_ID}/blobs/uploads/?digest=${DOCKER_DIGEST}" -H "Content-Type: application/octet-stream" --data-binary "@${PAYLOAD_DIR}/docker.bin"
benchmark_request Docker download 52428800 "${REGISTRY_URL}/v2/bench-${RUN_ID}/blobs/${DOCKER_DIGEST}"

benchmark_request npm upload 1048576 -X PUT "${REGISTRY_URL}/repository/npm-hosted/npm/${NPM_PACKAGE}" -H "Content-Type: application/json" --data-binary "@${PAYLOAD_DIR}/npm-publish.json"
benchmark_request npm download 1048576 "${REGISTRY_URL}/repository/npm-hosted/npm/${NPM_PACKAGE}/-/${NPM_TARBALL}"

benchmark_request PyPI upload 3145728 -X POST "${REGISTRY_URL}/repository/pypi-hosted/pypi/upload" -F "name=bench-perf-${RUN_ID}" -F "version=0.0.__ITER__" -F "content=@${PAYLOAD_DIR}/pypi.bin;filename=bench_perf_${RUN_ID}-0.0.__ITER__-py3-none-any.whl;type=application/x-wheel+zip"
benchmark_request PyPI download 3145728 "${REGISTRY_URL}/repository/pypi-hosted/packages/bench_perf_${RUN_ID}-0.0.1-py3-none-any.whl"

benchmark_request Cargo upload 1048576 -X PUT "${REGISTRY_URL}/repository/cargo-hosted/api/v1/crates/new" -H "Content-Type: application/octet-stream" --data-binary "@${PAYLOAD_DIR}/cargo-publish.bin"
benchmark_request Cargo download 1048576 "${REGISTRY_URL}/repository/cargo-hosted/api/v1/crates/${CARGO_PACKAGE}/0.1.0/download"

benchmark_request GoMod upload 524288 -X PUT "${REGISTRY_URL}/repository/gomod-hosted/gomod/example.com/bench/perf-${RUN_ID}/@v/v0.1.__ITER__.zip" -H "Content-Type: application/zip" --data-binary "@${PAYLOAD_DIR}/gomod.bin"
benchmark_request GoMod download 524288 "${REGISTRY_URL}/repository/gomod-hosted/gomod/example.com/bench/perf-${RUN_ID}/@v/v0.1.1.zip"

benchmark_request NuGet upload 1048576 -X PUT "${REGISTRY_URL}/repository/nuget-hosted/v3/package?id=bench.perf.${RUN_ID}&version=0.1.0" -H "Content-Type: application/octet-stream" --data-binary "@${PAYLOAD_DIR}/nuget.bin"
benchmark_request NuGet download 1048576 "${REGISTRY_URL}/repository/nuget-hosted/v3/flatcontainer/bench.perf.${RUN_ID}/0.1.0/bench.perf.${RUN_ID}.0.1.0.nupkg"

benchmark_request Raw upload 10485760 -X PUT "${REGISTRY_URL}/repository/raw-hosted/dist/${RUN_ID}/artifact-__ITER__.bin" -H "Content-Type: application/octet-stream" --data-binary "@${PAYLOAD_DIR}/raw.bin"
benchmark_request Raw download 10485760 "${REGISTRY_URL}/repository/raw-hosted/dist/${RUN_ID}/artifact-1.bin"

echo "Benchmark evidence written to ${RESULTS_DIR}"
