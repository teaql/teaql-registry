# Package proxy protocols

The protocol-specific GET/HEAD adapters run **after authentication**, use the
request's tenant context, and accept only read operations on proxy repositories.
The existing hosted/publication handlers retain their own behavior.

## Repository configuration

Create an online repository with the indicated `*-proxy` recipe, repository type
`PROXY`, publication policy `READ_ONLY`, and the upstream URL below. Replace
`NAME` in client URLs with the configured repository name. `BASE` is the internal
registry URL. Use persistent PostgreSQL and filesystem/S3 blob storage.

| Protocol | Recipe | Example upstream URL | Client endpoint |
| --- | --- | --- | --- |
| Maven | `maven2-proxy` | `https://repo.maven.apache.org/maven2` | `BASE/repository/NAME/` |
| npm | `npm-proxy` | `https://registry.npmjs.org` | `BASE/repository/NAME/npm/` |
| Cargo sparse | `cargo-proxy` | `https://index.crates.io` | `sparse+BASE/repository/NAME/cargo/index/` |
| PyPI | `pypi-proxy` | `https://pypi.org` | `BASE/repository/NAME/simple/` |
| Go modules | `gomod-proxy` | `https://proxy.golang.org` | `BASE/repository/NAME/gomod` |
| NuGet v3 | `nuget-proxy` | `https://api.nuget.org/v3/index.json` | `BASE/repository/NAME/v3/index.json` |
| Docker/OCI | `docker-proxy` | `https://registry-1.docker.io` | `REGISTRY_HOST/NAME/library/busybox:1.37.0` |
| Swift registry | `swift-proxy` | Upstream Swift registry base URL | `BASE/repository/NAME/swift` |
| Dart pub | `dart-proxy` | `https://pub.dev` | `BASE/repository/NAME/dart` |
| RubyGems | `rubygems-proxy` | `https://rubygems.org` | `BASE/repository/NAME/rubygems/` |
| Composer v2 | `composer-proxy` | `https://repo.packagist.org` | `BASE/repository/NAME/composer` |
| Conan v2 | `conan-proxy` | Upstream Conan remote base URL | `BASE/repository/NAME/conan` |
| Hex | `hex-proxy` | `https://repo.hex.pm` | `BASE/repository/NAME/hex/repo` |
| Raw HTTP | `raw-proxy` | Upstream file server base URL | `BASE/repository/NAME/PATH` |

For Docker, `NAME` selects the configured proxy; the remaining path is the
upstream image name. Nested upstream image paths are supported.

NuGet repository signatures and their certificate URLs require HTTPS. Set the
registry public URL to its HTTPS address or pass the correct HTTPS proxy headers.
The native harness uses a disposable local TLS certificate trusted by its client;
it does not disable signature verification.

Go checksum database requests stay inside the registry through
`/gomod/sumdb/sum.golang.org/`. The registry fetches signed records from
`https://sum.golang.org` by default. An administrator may set
`TEAQL_PROXY_SUMDB_BASE_URL` to a trusted internal checksum database mirror. Go's
native signature/checksum validation remains enabled. This is useful when the
configured module upstream does not implement the optional checksum proxy.

Use Hex's mirror configuration for a public hex.pm proxy so the client retains
the upstream repository identity and trusted public key. Signed Hex metadata is
forwarded unchanged rather than regenerated or re-signed by TeaQL Registry.

## Implementation

- Cache identity includes upstream URL and the requested Accept representation.
- Headers required by Swift, Docker, Conan and other clients persist with the body.
- Metadata-discovered archive, index and certificate links are rewritten to
  internal endpoints. Their mappings persist in the repository's blob metadata.
  Clients cannot supply an arbitrary upstream URL to the link endpoint.
- PyPI supports JSON and HTML simple pages, including archive hash fragments and
  separately fetched `.metadata` files.
- Cargo rewrites its download configuration and supports download URL templates.
- NuGet rewrites service resources, package metadata and signing-certificate URLs.
- Composer v2 minified metadata is expanded before archive URLs are rewritten;
  installation-reporting endpoints are not advertised.
- Docker supports upstream bearer challenges, manifest lists/OCI indexes,
  manifests and layers. Manifest bytes and digests remain unchanged.
- Per-artifact locks collapse concurrent cache misses. Public upstream credentials
  or a client's registry authorization headers are not forwarded to other origins.
- Successful artifacts and protocol metadata persist independently of the client's
  cache. A 404 has a persistent 300-second negative entry; other upstream errors
  are surfaced rather than converted to missing packages.

The protocol-specific adapters are in
`rust-web-axum/src/api/proxy_protocols.rs` and
`rust-web-axum/src/engine/remote.rs`. Maven/Raw use the existing generic dispatcher;
npm uses its dedicated metadata/tarball adapter.

## Native acceptance gate

Build the disposable localhost-only server, then run the native gate:

```sh
CARGO_INCREMENTAL=0 cargo build -p teaql-registry --example proxy_validation_server
python3 scripts/verify_proxy_clients.py \
  --binary /path/to/target/debug/examples/proxy_validation_server
```

The gate requires Docker, Cargo/Rust, Python/pip, Go, .NET 8, Swift, OpenSSL, and
Conan at `.proxy-client-venv/bin/conan`. It uses official `dart:stable`,
`ruby:3.4-alpine`, `composer:2`, and `elixir:1.18-alpine` client images. Install or
pull these tools before running; the gate does not install production services.
`--protocols cargo,pypi,...` selects a subset.

The script owns the disposable `teaql-all-proxy-pg` container on localhost:25441
and replaces it at the start of a fresh run. It uses localhost ports 28441
(registry), 28443 (test HTTPS), and 28445 (upstream gateway). The example server
uses an unsecured test identity and binds only to localhost; it is not a
production deployment entry point.

Every selected protocol executes three phases:

1. **Cold**: new client cache, real upstream download, then build/import/run the
   result. At least one upstream request must be recorded.
2. **Cached**: another new client cache, same operation; upstream request count
   must remain unchanged.
3. **Offline**: stop the upstream gateway, restart the registry against the same
   PostgreSQL/file blobs, then repeat with another empty client cache. Requests
   must succeed without contacting the upstream.

The Docker image is removed between phases; the gate additionally requires a
layer-download completion in each native pull log. Go checksum validation and
NuGet signature verification remain enabled. The .NET test disables SDK workload
advertising-manifest background updates with
`DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true`, so the cache comparison covers
the specified package restore/build rather than unrelated SDK update downloads. Client HTTP(S) proxy variables point to an unreachable endpoint, with only
loopback exempted; package source configuration points at the internal registry.
The registry's upstream gateway can fetch public artifacts during online phases.

Swift and Conan use actual TeaQL hosted upstream repositories populated through
`swift package-registry publish` and `conan create/upload`; the other package
protocols use public upstreams. Raw uses a native HTTP-uploaded fixture. This
keeps the Swift/Conan run reproducible without depending on a third-party private
registry account. The gateway records accesses and rewrites metadata links so
that stopping it disconnects every artifact source, including CDN archive links.

Evidence is written under `evidence/all-proxy/run-*/`: per-client logs,
`summary.json`, `upstream.jsonl`, and registry process logs. Disposable client
files, test TLS keys and downloaded artifacts are excluded from Git.

## Current boundaries

This is an on-demand cache: metadata lookup does not prefetch all archives or the
entire dependency closure. Dependency installation scripts, Git dependencies and
non-registry toolchain downloads require their own internal sources.

Positive cache entries currently remain until retention/administrative eviction;
there is no automatic positive TTL or conditional index refresh. Use a fresh
cache/repository when testing an upstream metadata update. Proxy/group namespace
routing is separate; the existing group engine does not guarantee private
namespace isolation. Authenticated private upstreams and unusual vendor-specific
protocol extensions need their own profiles beyond the upstreams verified here.

Protocol references: [Cargo index](https://doc.rust-lang.org/cargo/reference/registry-index.html),
[PyPI simple API](https://packaging.python.org/en/latest/specifications/simple-repository-api/),
[NuGet repository signatures](https://learn.microsoft.com/en-us/nuget/api/repository-signatures-resource),
[Docker bearer authentication](https://distribution.github.io/distribution/spec/auth/),
[Composer repositories](https://getcomposer.org/doc/05-repositories.md),
[Hex mirrors](https://hex.pm/docs/mirrors).
