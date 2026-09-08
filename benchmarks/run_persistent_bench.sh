#!/bin/bash
set -euo pipefail

DB_CONTAINER="teaql-current-postgres"
REGISTRY_BIN="/home/philip/shared-cargo-target/release/teaql-registry"

echo "=== Setup Persistent Mode Benchmark ==="
# Cleanup db and old server
pkill -x 'teaql-registry' 2>/dev/null || true
docker exec $DB_CONTAINER psql -U postgres -c "DROP DATABASE IF EXISTS teaql_registry_bench" 2>/dev/null || true
docker exec $DB_CONTAINER psql -U postgres -c "CREATE DATABASE teaql_registry_bench"
rm -rf /tmp/teaql-registry-credentials /tmp/teaql-bench-server.log

# Ensure S3 bucket exists by making a dummy request or relying on S3BlobStore::init()
# (S3BlobStore::init() creates bucket if not exist in rustfs)

export TEAQL_REGISTRY_CORE_DATABASE_URL="host=127.0.0.1 port=15432 dbname=teaql_registry_bench user=postgres password=postgres"
export TEAQL_REGISTRY_CORE_DATABASE_USER="postgres"
export TEAQL_REGISTRY_CORE_DATABASE_PASSWORD="postgres"
export PORT=8081
export CREDENTIALS_DIR="/tmp/teaql-registry-credentials"
export RUST_LOG=warn

# We don't set MEMORY_MODE=true.
nohup $REGISTRY_BIN > /tmp/teaql-bench-server.log 2>&1 &
SERVER_PID=$!
echo "Started server with PID $SERVER_PID"

echo "Waiting for server to be ready..."
for i in $(seq 1 30); do
    if curl -sf http://127.0.0.1:8081/help > /dev/null 2>&1; then
        echo "Server ready after ${i}s"
        break
    fi
    sleep 1
done

# Start memory monitoring
echo "Starting memory monitor..."
MEM_LOG="/tmp/bench_memory.log"
> $MEM_LOG
(
    while kill -0 $SERVER_PID 2>/dev/null; do
        # Record docker postgres memory (in MB)
        PG_MEM=$(docker stats --no-stream --format "{{.MemUsage}}" $DB_CONTAINER | awk '{print $1}')
        # Record rust registry memory (RSS in MB)
        RUST_MEM=$(ps -o rss= -p $SERVER_PID | awk '{print $1/1024 "MiB"}')
        echo "$(date -Iseconds) | PG: $PG_MEM | Registry: $RUST_MEM" >> $MEM_LOG
        sleep 2
    done
) &
MONITOR_PID=$!

echo "Running benchmark script..."
# Using iteration=5 for persistent mode to save time, or 10. We will use 10.
export CREDENTIALS_DIR="/tmp/teaql-registry-credentials"
export BENCH_ITERATIONS=10
bash /home/philip/githome/teaql-registry/benchmarks/run_bench.sh > /tmp/teaql-bench-persistent.log 2>&1

echo "Benchmark finished. Stopping server and monitor..."
kill $SERVER_PID 2>/dev/null || true
kill $MONITOR_PID 2>/dev/null || true

echo "=== Memory Log Summary ==="
cat $MEM_LOG

echo "=== Result CSV ==="
cat /tmp/teaql-bench-results.csv
