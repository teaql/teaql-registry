# TeaQL Registry performance evidence

This report describes the retained candidate-build benchmark from
`results/20260909-persistent-s3/`. It replaces the earlier manually summarized
numbers, which did not retain raw samples for every claimed protocol.

## Scope and environment

- Commit: `1f2f8517d7afe0a9bcdab5494dc67232125d09e8`
- Worktree at test start: clean
- Recorded: 2026-09-08 16:50:43 UTC
- Host: Linux 6.8.0-136-generic, Intel Core i7-10750H
- Metadata: local PostgreSQL 16 container
- Blob data: local S3-compatible RustFS over HTTP
- Registry: optimized release build with operational log level `warn`
- Samples: 5 iterations for every operation
- Client: curl 7.81.0

The route benchmark measures authenticated Registry HTTP endpoints. It does
not claim package-manager startup or dependency-resolution performance. Native
publish/consume compatibility is covered independently under
`../evidence/native-clients/20260909-native-clients/`.

## Results

| Format | Operation | Payload | Average | Min / max | Throughput |
| --- | --- | ---: | ---: | ---: | ---: |
| Maven | upload | 5 MiB | 275.951 ms | 240.785 / 307.782 ms | 152.0 Mbps |
| Maven | download | 5 MiB | 78.461 ms | 60.237 / 105.465 ms | 534.6 Mbps |
| Docker | upload | 50 MiB | 2,056.845 ms | 1,920.769 / 2,331.281 ms | 203.9 Mbps |
| Docker | download | 50 MiB | 270.331 ms | 203.748 / 352.570 ms | 1,551.5 Mbps |
| npm | upload | 1 MiB | 133.882 ms | 99.700 / 188.036 ms | 62.7 Mbps |
| npm | download | 1 MiB | 49.123 ms | 46.127 / 53.454 ms | 170.8 Mbps |
| PyPI | upload | 3 MiB | 222.944 ms | 206.851 / 251.798 ms | 112.9 Mbps |
| PyPI | download | 3 MiB | 65.906 ms | 60.797 / 78.061 ms | 381.8 Mbps |
| Cargo | upload | 1 MiB | 129.787 ms | 117.013 / 154.117 ms | 64.6 Mbps |
| Cargo | download | 1 MiB | 56.503 ms | 54.806 / 59.750 ms | 148.5 Mbps |
| GoMod | upload | 0.5 MiB | 113.256 ms | 85.956 / 147.885 ms | 37.0 Mbps |
| GoMod | download | 0.5 MiB | 53.890 ms | 38.533 / 64.878 ms | 77.8 Mbps |
| NuGet | upload | 1 MiB | 119.033 ms | 105.961 / 134.623 ms | 70.5 Mbps |
| NuGet | download | 1 MiB | 77.097 ms | 66.260 / 85.261 ms | 108.8 Mbps |
| Raw | upload | 10 MiB | 508.696 ms | 419.622 / 688.240 ms | 164.9 Mbps |
| Raw | download | 10 MiB | 92.902 ms | 72.919 / 113.462 ms | 902.9 Mbps |

Registry RSS ranged from 39.6 MiB to 221.2 MiB during this run. The 50 MiB
Docker transfer was the largest payload. These numbers describe this machine
and local topology only; they are not generalized production capacity claims.

## Evidence and reproduction

- `summary.csv`: aggregate values shown above
- `samples.csv`: every HTTP status and raw curl duration
- `environment.txt`: exact commit, cleanliness, host, tools, and payload hashes
- `memory.csv`: one-second RSS observations
- `server.txt`: server output for the run

Reproduce with the commands and required environment documented in
`README.md`. Every non-2xx response fails the benchmark; failed requests are
never converted to zero-duration samples.
