#!/bin/bash
# TeaQL Registry — Intranet Performance Benchmark
# Tests upload/download latency for all 8 artifact formats in pure in-memory mode.
set -euo pipefail

REGISTRY_URL="${REGISTRY_URL:-http://127.0.0.1:8081}"
CREDS_DIR="${CREDENTIALS_DIR:-/var/lib/teaql-registry/credentials}"
RESULTS_FILE="/tmp/teaql-bench-results.csv"
TMPDIR_BENCH="/tmp/teaql-bench-payloads"

# Read global admin password from credentials file
if [ -f "$CREDS_DIR/global-admin.txt" ]; then
    ADMIN_PASS=$(grep '^admin / ' "$CREDS_DIR/global-admin.txt" | sed 's/^admin \/ //')
else
    : "${ADMIN_PASSWORD:?Set ADMIN_PASSWORD or provide $CREDS_DIR/global-admin.txt}"
    ADMIN_PASS="$ADMIN_PASSWORD"
fi
AUTH="-u admin:${ADMIN_PASS}"

echo "=== TeaQL Registry Intranet Performance Benchmark ==="
echo "Registry: $REGISTRY_URL"
echo "Mode: Pure In-Memory"
echo "Auth user: admin"
echo ""

# Verify connectivity
if ! curl -sf "$REGISTRY_URL/help" > /dev/null 2>&1; then
    echo "ERROR: Cannot reach registry at $REGISTRY_URL"
    exit 1
fi
echo "Registry connectivity: OK"

# Generate test payloads of typical sizes
mkdir -p "$TMPDIR_BENCH"

generate_payload() {
    local file="$1"
    local size_bytes="$2"
    dd if=/dev/urandom of="$file" bs=1024 count=$((size_bytes / 1024)) 2>/dev/null
}

echo "Generating test payloads..."
generate_payload "$TMPDIR_BENCH/maven-jar-5m.bin"    5242880    # 5 MB
generate_payload "$TMPDIR_BENCH/docker-layer-50m.bin" 52428800   # 50 MB
generate_payload "$TMPDIR_BENCH/npm-tgz-2m.bin"      2097152    # 2 MB
generate_payload "$TMPDIR_BENCH/pypi-whl-3m.bin"     3145728    # 3 MB
generate_payload "$TMPDIR_BENCH/cargo-crate-1m.bin"  1048576    # 1 MB
generate_payload "$TMPDIR_BENCH/gomod-zip-500k.bin"  524288     # 500 KB
generate_payload "$TMPDIR_BENCH/nuget-nupkg-5m.bin"  5242880    # 5 MB
generate_payload "$TMPDIR_BENCH/raw-bin-10m.bin"     10485760   # 10 MB
echo "Payloads ready."

ITERATIONS="${BENCH_ITERATIONS:-10}"
RUN_ID=$$

echo ""
echo "Running benchmarks ($ITERATIONS iterations each, run_id=$RUN_ID)..."
echo "format,operation,size_bytes,iterations,total_ms,avg_ms,min_ms,max_ms,throughput_mbps" > "$RESULTS_FILE"

# Benchmark function using awk for floating point
bench() {
    local format="$1"
    local op="$2"
    local size="$3"
    shift 3

    local min=999999
    local max=0
    local total=0

    for i in $(seq 1 $ITERATIONS); do
        local raw_time
        raw_time=$(curl -sf -o /dev/null -w '%{time_total}' $AUTH "$@" 2>/dev/null || echo "0")
        local ms
        ms=$(echo "$raw_time" | awk '{printf "%d", $1 * 1000}')
        [ -z "$ms" ] && ms=0
        total=$((total + ms))
        [ "$ms" -lt "$min" ] && min=$ms
        [ "$ms" -gt "$max" ] && max=$ms
    done

    local avg=$((total / ITERATIONS))
    local size_mb
    size_mb=$(echo "$size" | awk '{printf "%.2f", $1 / 1048576}')
    local throughput="0"
    if [ "$avg" -gt 0 ]; then
        throughput=$(echo "$size_mb $avg" | awk '{printf "%.1f", $1 * 1000 / $2 * 8}')
    fi

    printf "  %-12s %-10s %6s MB  avg=%4d ms  min=%4d ms  max=%4d ms  ~%s Mbps\n" \
        "$format" "$op" "$size_mb" "$avg" "$min" "$max" "$throughput"
    echo "$format,$op,$size,$ITERATIONS,$total,$avg,$min,$max,$throughput" >> "$RESULTS_FILE"
}

echo ""
echo "--- 1. Maven2 (5 MB JAR) ---"
bench "Maven2" "upload" 5242880 \
    -X PUT "$REGISTRY_URL/repository/maven-releases/com/bench/perf/1.0.$RUN_ID/perf-1.0.$RUN_ID-bench.jar" \
    -H "Content-Type: application/java-archive" \
    --data-binary "@$TMPDIR_BENCH/maven-jar-5m.bin"

bench "Maven2" "download" 5242880 \
    "$REGISTRY_URL/repository/maven-releases/com/bench/perf/1.0.$RUN_ID/perf-1.0.$RUN_ID-bench.jar"

echo ""
echo "--- 2. Docker v2 (50 MB monolithic blob upload) ---"
DOCKER_DIGEST=$(sha256sum "$TMPDIR_BENCH/docker-layer-50m.bin" | awk '{print "sha256:"$1}')
bench "Docker" "upload" 52428800 \
    -X POST "$REGISTRY_URL/v2/bench-image/blobs/uploads/?digest=$DOCKER_DIGEST" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@$TMPDIR_BENCH/docker-layer-50m.bin"

bench "Docker" "download" 52428800 \
    "$REGISTRY_URL/v2/bench-image/blobs/$DOCKER_DIGEST"

echo ""
echo "--- 3. NPM (2 MB tarball via direct PUT) ---"
bench "NPM" "upload" 2097152 \
    -X PUT "$REGISTRY_URL/repository/npm-hosted/npm/@bench/perf-pkg/-/perf-pkg-1.0.$RUN_ID.tgz" \
    -H "Content-Type: application/gzip" \
    --data-binary "@$TMPDIR_BENCH/npm-tgz-2m.bin"

bench "NPM" "download" 2097152 \
    "$REGISTRY_URL/repository/npm-hosted/npm/@bench/perf-pkg/-/perf-pkg-1.0.$RUN_ID.tgz"

echo ""
echo "--- 4. PyPI (3 MB wheel) ---"
bench "PyPI" "upload" 3145728 \
    -X PUT "$REGISTRY_URL/repository/pypi-hosted/packages/bench_perf-1.0.$RUN_ID-py3-none-any.whl" \
    -H "Content-Type: application/x-wheel+zip" \
    --data-binary "@$TMPDIR_BENCH/pypi-whl-3m.bin"

bench "PyPI" "download" 3145728 \
    "$REGISTRY_URL/repository/pypi-hosted/packages/bench_perf-1.0.$RUN_ID-py3-none-any.whl"

echo ""
echo "--- 5. Cargo (1 MB crate) ---"
bench "Cargo" "upload" 1048576 \
    -X PUT "$REGISTRY_URL/repository/cargo-hosted/api/v1/crates/bench-perf-crate/0.1.$RUN_ID/download" \
    -H "Content-Type: application/gzip" \
    --data-binary "@$TMPDIR_BENCH/cargo-crate-1m.bin"

bench "Cargo" "download" 1048576 \
    "$REGISTRY_URL/repository/cargo-hosted/api/v1/crates/bench-perf-crate/0.1.$RUN_ID/download"

echo ""
echo "--- 6. Go Modules (500 KB zip) ---"
bench "GoMod" "upload" 524288 \
    -X PUT "$REGISTRY_URL/repository/gomod-hosted/gomod/github.com/bench/perf/@v/v1.0.$RUN_ID.zip" \
    -H "Content-Type: application/zip" \
    --data-binary "@$TMPDIR_BENCH/gomod-zip-500k.bin"

bench "GoMod" "download" 524288 \
    "$REGISTRY_URL/repository/gomod-hosted/gomod/github.com/bench/perf/@v/v1.0.$RUN_ID.zip"

echo ""
echo "--- 7. NuGet (5 MB nupkg) ---"
bench "NuGet" "upload" 5242880 \
    -X PUT "$REGISTRY_URL/repository/nuget-hosted/v3/flatcontainer/bench.perf/1.0.$RUN_ID/bench.perf.1.0.$RUN_ID.nupkg" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@$TMPDIR_BENCH/nuget-nupkg-5m.bin"

bench "NuGet" "download" 5242880 \
    "$REGISTRY_URL/repository/nuget-hosted/v3/flatcontainer/bench.perf/1.0.$RUN_ID/bench.perf.1.0.$RUN_ID.nupkg"

echo ""
echo "--- 8. Raw (10 MB binary) ---"
bench "Raw" "upload" 10485760 \
    -X PUT "$REGISTRY_URL/repository/raw-hosted/dist/bench/v1.0.$RUN_ID/teaql-cli-bench.tar.gz" \
    -H "Content-Type: application/gzip" \
    --data-binary "@$TMPDIR_BENCH/raw-bin-10m.bin"

bench "Raw" "download" 10485760 \
    "$REGISTRY_URL/repository/raw-hosted/dist/bench/v1.0.$RUN_ID/teaql-cli-bench.tar.gz"

echo ""
echo "=== Benchmark Complete ==="
echo "Raw CSV data: $RESULTS_FILE"

# Cleanup
rm -rf "$TMPDIR_BENCH"
