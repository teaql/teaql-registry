# Registry benchmarks

These benchmarks measure authenticated HTTP upload/download paths through the
Registry and its configured blob store. They exercise the real wire endpoint
for all eight repository formats with deterministic sizes, but they are not a
substitute for native package-manager conformance tests.

Every successful run retains:

- `environment.txt`: commit, dirty state, host, curl version, mode, and payload hashes;
- `samples.csv`: every HTTP status and raw curl duration;
- `summary.csv`: averages, extrema, and payload throughput;
- `memory.csv` and `server.txt` for persistent-mode runs.

Any non-2xx response fails the run. A failed request is never converted into a
zero-duration measurement.

## Run against an existing Registry

```bash
export REGISTRY_URL=http://127.0.0.1:8081
export CREDENTIALS_DIR=/path/to/registry/credentials
export BENCH_MODE=memory
export BENCH_ITERATIONS=10
./benchmarks/run_bench.sh
```

Set `ADMIN_PASSWORD` instead of `CREDENTIALS_DIR` only when the password is
already supplied securely by the environment.

## Run a persistent S3 benchmark

Build a release binary and provide an already-created, dedicated PostgreSQL
database plus S3-compatible storage:

```bash
cargo build --release -p teaql-registry

export TEAQL_REGISTRY_SERVICE_CORE_DATABASE_URL=postgresql://user:password@127.0.0.1:5432/registry_benchmark
export TEAQL_REGISTRY_SERVICE_CORE_DATABASE_USER=user
export TEAQL_REGISTRY_SERVICE_CORE_DATABASE_PASSWORD=password
export S3_ENDPOINT=http://127.0.0.1:9010
export S3_ACCESS_KEY=benchmark-user
export S3_SECRET_KEY=benchmark-secret
export S3_BUCKET=teaql-registry-benchmark
export ADMIN_PASSWORD='a dedicated strong benchmark credential'

./benchmarks/run_persistent_bench.sh
```

The script starts and stops only the Registry process it owns. It does not
kill unrelated processes, drop databases, or remove persistent blob data.

The route benchmark requires standard Unix tools plus `curl`, `base64`, and
`perl` (used only to frame the Cargo publish wire payload).

## Native package-manager conformance

`scripts/verify_native_clients.sh` publishes and consumes temporary packages
with Maven, npm, Twine/pip, Cargo, Go, dotnet, and Docker. Raw storage is
verified with authenticated HTTP because it has no ecosystem package manager.
The script creates a one-day PAT, revokes it on exit, redacts credentials from
retained logs, and fails on the first unsuccessful native command.

Go refuses to send credentials to an HTTP `GOPROXY`. For a local HTTP-only
run, start the Registry with `ALLOW_ANONYMOUS_READ=true`; authenticated Go
module reads should otherwise be verified through the production HTTPS URL.
