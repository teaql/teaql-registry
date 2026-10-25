# Native proxy verification — 2026-10-25

Development checkout: `~/workspace-ext/teaql-regist-proxy` on
`codex/registry-proxy-validation`, based on `95a4a681b955` with the implementation
changes in this worktree.

**PASS: 12 protocols × 3 phases = 36 native acceptance phases.** Every protocol
reached a real upstream on its cold run. Every cache replay and offline run
performed zero upstream requests. The registry was restarted after the upstream
gateway stopped; each phase used new client cache/project directories. Docker
images were removed between phases and each pull completed a layer download.

| Protocol | Cold upstream requests | Cached requests | Offline after restart |
| --- | ---: | ---: | ---: |
| Cargo | 3 | 0 | 0 |
| PyPI / pip | 3 | 0 | 0 |
| Go modules + signed checksum database | 22 | 0 | 0 |
| NuGet v3 over HTTPS, signatures enabled | 4 | 0 | 0 |
| Docker Hub / OCI index, manifest and layers | 10 | 0 | 0 |
| Swift registry | 4 | 0 | 0 |
| Dart pub | 2 | 0 | 0 |
| RubyGems | 4 | 0 | 0 |
| Composer v2 | 3 | 0 | 0 |
| Conan v2 | 10 | 0 | 0 |
| Hex / Mix, signed metadata | 4 | 0 | 0 |
| Raw / curl | 1 | 0 | 0 |

The native commands also built, imported, restored, or ran the consumed artifacts;
Conan verified the downloaded binary package header and Raw compared exact bytes.
Swift/Conan upstream fixtures were published with their native clients to separate
TeaQL hosted repositories. Raw used an HTTP-uploaded fixture. Other protocols used
public upstreams (crates.io, PyPI, proxy.golang.org and sum.golang.org, NuGet.org,
Docker Hub, pub.dev, RubyGems.org, Packagist and repo.hex.pm).

NuGet used an internal HTTPS endpoint and a locally trusted disposable test
certificate. Package signature verification remained enabled. The test disabled
SDK workload advertising-manifest background updates with
`DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true` to keep the restore/build fixture
deterministic. Go checksum validation and Hex metadata signature validation also
remained enabled.

**Repository gate PASS:** `scripts/verify.sh`; 114 workspace tests, zero
failures. Formatting, generated-domain checks, application Clippy, workspace
tests, and patch hygiene all passed. npm and Maven native regressions were enabled
in this run in addition to the twelve-protocol native acceptance gate.

## Evidence

- Acceptance run: `evidence/all-proxy/run-20261025-194125`.
- Summary: `evidence/all-proxy/run-20261025-194125/summary.json`.
- Upstream access records: `evidence/all-proxy/run-20261025-194125/upstream.jsonl`.
- Per-client and registry process logs are in the same run directory.
- Complete native output: `evidence/all-proxy/final-native.log`.
- Complete repository gate: `evidence/all-proxy/verify.log`.
- Source hashes: `evidence/all-proxy/source-manifest.json`.
- Configuration, implementation boundaries, and reproduction:
  [proxy-protocols.md](proxy-protocols.md).

Disposable client files, TLS private keys and downloaded artifacts remain local
and are excluded from Git.

## Toolchain snapshot

```text
cargo 1.97.1 (c980f4866 2026-06-30)
pip 26.2 from /home/philip/.local/lib/python3.10/site-packages/pip (python 3.10)
go version go1.18.1 linux/amd64
8.0.132
Swift version 6.3.3 (swift-6.3.3-RELEASE)
Docker version 29.1.3, build 29.1.3-0ubuntu3~22.04.2
Conan version 2.33.0
ruby@sha256:730b265fc537e93f4ad335031ba95f8a9df88dc75bd9462968b92f368a115f6d
composer@sha256:b6dcd72ca98fa19d0e5b8476de179859704891fad06dd9e0b96833d4c430a5fc
dart@sha256:6acc5a8ef67e606891dcbb51d8c02214d53356409fd253abe28d9fefaa7391ce
elixir@sha256:69cce719597560821541a8d3c283620633f4dafe1c5df51e898f5097c6467f77
```
