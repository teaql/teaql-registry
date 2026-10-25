# npm and Maven proxy development

Development checkout: `~/workspace-ext/teaql-regist-proxy`, branch
`codex/registry-proxy-validation`, based on `95a4a68`.

## Deployment

Developer environments connect only to the internal registry. The registry can
fetch public dependencies from explicitly configured upstream repositories.
Use durable filesystem or S3 blob storage plus PostgreSQL for persistent caches.

| Repository recipe | Upstream remote URL | Client registry URL |
| --- | --- | --- |
| `npm-proxy` | `https://registry.npmjs.org` | `http://REGISTRY/repository/npm-proxy/npm/` |
| `maven2-proxy` | `https://repo.maven.apache.org/maven2` | `http://REGISTRY/repository/maven-central/` |

The repository names in the final URL must match the names configured in the
registry. Set `online=true`; `READ_ONLY` is valid for a proxy because cache writes
are internal operations. Client PUT/publication is still rejected for proxies.
Configure production authentication and the public base URL through the existing
registry settings.

For npm, point `npm install --registry` or `.npmrc` at the npm client URL above.
For Maven, configure the client URL as the desired repository or mirror in
`settings.xml`. A `mirrorOf=*` mirror routes dependency and plugin repository
traffic through that endpoint; upstream must contain the requested artifacts.

## Implementation verified by the regression harness

- Local cache lookup precedes upstream HTTP GET.
- Downloaded artifacts populate the blob store and database even when the
  repository publication policy is read-only.
- npm metadata preserves dependencies, engines, integrity and other upstream
  JSON fields; tarball URLs are rewritten to the internal npm route.
- Standard scoped and unscoped npm registry tarball paths are supported.
- npm tarball downloads and Maven GET/HEAD use the repository dispatcher.
- Upstream 404 results are negatively cached for 300 seconds; other HTTP errors
  surface as errors. Negative entries are keyed by repository ID and upstream.
- Cached npm metadata, tarballs and Maven files remain readable when the upstream
  is unavailable.

## Boundaries

Cache hits currently have no positive TTL or metadata refresh. Warming a package
index alone does not download its archives or transitive dependencies. A fully
isolated deployment must warm the complete dependency closure, including build
plugins and resources fetched by installation scripts.

This change handles standard npm registry tarball paths (`/<package>/-/<file>`).
Nonstandard archive paths return an error rather than advertising an external
archive URL to the client. Authenticated private upstreams and protocol-specific
proxy behavior for other ecosystems require separate verification.

The existing group engine discovers same-format repositories, with special
handling for `maven-public`; it is not an explicit ordered membership or namespace
routing policy. A private package miss can reach a proxy member. Do not describe
the group endpoint as guaranteeing private namespace isolation. Keep private
packages in hosted repositories and, for npm scopes, explicitly map private
scopes to the hosted endpoint. No upstream upload is implemented by ProxyEngine;
it issues GET requests only.

## Reproduce

Use an isolated PostgreSQL database and export the three
`TEAQL_REGISTRY_SERVICE_CORE_DATABASE_*` variables described in README.md.
The new harness uses its own in-process HTTP upstream and memory blob store.

```sh
cargo test -p teaql-registry --test proxy_integration_test -- --nocapture
```

To require real npm and Maven clients on PATH:

```sh
TEAQL_TEST_NATIVE_NPM=1 TEAQL_TEST_NATIVE_MAVEN=1 \
  cargo test -p teaql-registry --test proxy_integration_test -- --nocapture
```

Both native clients use fresh local caches after the upstream server has stopped.
The npm fixture includes a scoped transitive dependency. Maven resolves a remote
parent POM during `validate`; this does not claim a full Java compilation or
plugin dependency-closure test. Neither test needs public internet access.

The repository-wide gate is `bash scripts/verify.sh`, requiring isolated S3
configuration in addition to PostgreSQL. Verification output for this checkout
is retained in `evidence/proxy-validation/`.

To include live public-upstream downloads, add `TEAQL_TEST_PUBLIC_PROXY=1`.
This opt-in check downloads the `is-number@7.0.0` archive through the npm proxy
and the `commons-lang3:3.14.0` POM through the Maven proxy. It requires outbound
internet access from the registry test process.

## Observed results (2026-10-25)

- `cargo check -p teaql-registry --tests`: passed.
- HTTP proxy regression and native npm installation: passed.
- Live npmjs archive and Maven Central POM downloads: passed.
- Native npm fresh-cache installation after upstream shutdown: passed.
- Native Maven fresh-repository parent POM resolution after upstream shutdown: passed.

Native/public output: `evidence/proxy-validation/native-and-public.log`.
The repository-wide verification output is `evidence/proxy-validation/verify.log`.

- Full `scripts/verify.sh` gate: passed; 114 tests passed, zero failures.
  Formatting, generated-domain checks, application Clippy, workspace tests and
  patch hygiene all passed. Generated-domain dead-code warnings predate this change.
- Native clients: Node.js 22.20.0, npm 10.9.3, Maven 3.9.16, Java 25.0.2.

Development-only test services are `teaql-regist-proxy-test-pg`
(localhost:25439) and `teaql-regist-proxy-test-s3` (localhost:25440).
Build artifacts use `/tmp/teaql-regist-proxy-target`.
