# Verification evidence

This directory keeps reproducible evidence separate from feature claims. The
latest retained baseline is tied to source commit
`59be352985af9c61c4d516324080c6698b139311`.

## Current baseline

| Layer | Result | Evidence |
|---|---:|---|
| Controlled workspace verification | 97 passed, 0 failed | [`verification/20261001-expression-best-practice-complete`](verification/20261001-expression-best-practice-complete/) |
| Native package-manager conformance | 17 PASS, 1 N/A | [`native-clients/20261001-expression-best-practice-native`](native-clients/20261001-expression-best-practice-native/) |
| Persistent-storage benchmark | Retained historical run | [`../benchmarks/results/20260909`](../benchmarks/results/20260909/) |

The workspace verification runs five gates: Rust formatting, generated TeaQL
library checking, strict Clippy for application-owned crates, the complete
serial workspace test suite, and patch hygiene. The native suite exercises the
actual ecosystem tools against a running Registry rather than calling handlers
directly. The exact per-suite counts are retained in
[`test-suites.tsv`](verification/20261001-expression-best-practice-complete/test-suites.tsv),
and the complete 683-line test transcript is retained as `verify.log` beside it.

## Tested format matrix

| Format | Protocol/integration coverage | Native client retained |
|---|---|---|
| Cargo | publish, sparse index dependency metadata, download | Cargo 1.97.1 publish + clean consumer check |
| Maven 2 | hosted upload/download and metadata behavior | Maven 3.9.16 deploy + dependency get |
| npm | package lifecycle and scoped packages | npm 11.17.0 publish + clean install |
| PyPI | package lifecycle and simple-index behavior | Python 3.10.12/Twine publish + pip install |
| Go Modules | module lifecycle and proxy paths | Go 1.18.1 `go mod download`; publish is N/A |
| NuGet | package lifecycle and multipart upload | .NET SDK 8.0.130 push + restore |
| Swift | registry lifecycle | Swift 6.3.3 publish + resolve + build |
| Docker/OCI | registry push/pull lifecycle | Docker 29.1.3 push + pull |
| Raw | authenticated binary upload/download and SHA-256 | curl PUT + GET with digest comparison |
| Dart Pub | package lifecycle | Protocol integration test |
| RubyGems | package lifecycle | Protocol integration test |
| Composer | package lifecycle | Protocol integration test |
| Conan 2 | recipe/package lifecycle | Protocol integration test |
| Hex | registry lifecycle and signature behavior | Protocol integration test |

“Protocol integration test” and “native client retained” are intentionally
different evidence levels. A format without a retained native-client row is not
presented as having been tested with its ecosystem CLI.

## Tested versions

The 2026-10-01 baseline used:

| Component | Version |
|---|---|
| TeaQL runtime/core/data-service/PostgreSQL provider | 5.0.5 |
| `cargo-teaql` | 2.0.14 |
| Rust / Cargo | 1.97.1 |
| PostgreSQL | 16.15 (`postgres:16-alpine`) |
| RustFS | 1.0.0-beta.12 |
| Axum / Tokio / reqwest | 0.7.9 / 1.53.1 / 0.12.28 |
| Ratatui | 0.29.0 |
| React / TypeScript / Vite | 18.3.1 / 5.9.3 / 6.4.3 |

Exact package-manager versions are retained in each native run's
`environment.txt`; exact resolved Rust dependencies are retained with the
workspace verification.

## TeaQL capability evidence

| Capability | Current evidence | Confidence |
|---|---|---|
| Central tenant isolation | `test_user_context_tenant_isolation` provisions two tenants and verifies repository visibility through tenant-bound `UserContext` instances without caller-side tenant filters | Explicit integration test |
| Data mutation | Repository, blob, component, asset, tenant, security, token, and webhook create/update/delete paths are exercised across the integration suite | Broad integration coverage |
| Pagination | Component and Asset read services use deterministic ID ordering with generated `execute_for_page`; `generated_component_pages_are_stable_and_non_overlapping` covers page boundaries | Explicit service integration test; aggregate search pagination still needs database pushdown |
| Mutation Approval | TeaQL 5.0.5 is resolved and mutation paths pass, but no retained test installs an allow/deny policy and asserts its approval/audit snapshot | Gap |
| `E` expressions | `e_expression_distinguishes_selected_values_from_missing_relations` asserts projected scalar success, missing-relation fail-fast behavior, and selected relation traversal | Explicit integration test |

This table is deliberately stricter than “the build passed.” Mutation Approval
and database-pushed aggregate search pagination remain visible follow-up work;
they should not be represented as runtime-verified behavior until those
assertions and logs exist.

## Reproduce

With a fresh PostgreSQL database and the three documented database variables:

```bash
VERIFICATION_RUN_ID=my-run \
VERIFICATION_POSTGRES_VERSION="PostgreSQL 16.x" \
./scripts/record_verification.sh
```

Against a running Registry:

```bash
NATIVE_RUN_ID=my-native-run \
REGISTRY_URL=https://registry.example.test \
ADMIN_PASSWORD='...' \
NATIVE_READ_PROFILE=authenticated-https \
./scripts/verify_native_clients.sh
```

Both recorders reject an existing output directory so a rerun cannot silently
overwrite an earlier result or collide with immutable package versions.

## Scope and boundaries

- The current workspace baseline starts from a fresh disposable schema. It is
  evidence for clean installation, not for in-place migration from every older
  schema.
- The retained 2026-10-01 native run used loopback HTTP. Read-only access was
  enabled for consumption because Go refuses credentials in an insecure HTTP
  `GOPROXY`; all publication operations still used a one-day scoped PAT.
  Production-style authenticated Go consumption should be rerun over HTTPS.
- Logs redact the administrator password and PAT. No database URL or database
  credential is written by `record_verification.sh`.
