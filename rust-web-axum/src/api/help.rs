use axum::{
    http::header,
    response::{IntoResponse, Response},
};

/// Plain text help document returned by GET /help
const HELP_TEXT: &str = r#"TeaQL Registry — Quick Start Guide
=====================================

1. INITIAL CREDENTIALS
   On first startup, the system generates secure random passwords for:
   - The global admin user (admin)
   - 3 tenants (development, test, UAT) × 2 users each (admin, developer)

   All passwords are written to the credentials directory.

   Default path: /var/lib/teaql-registry/credentials/
   Custom path:  set CREDENTIALS_DIR env var before startup.

   Files created:
     global-admin.txt      — global admin password
     all-credentials.txt   — combined listing (global + all tenants)
     development.txt       — development tenant only
     test.txt              — test tenant only
     uat.txt               — UAT tenant only

   To use a specific global admin password instead of a generated one,
   set the ADMIN_PASSWORD env var (subject to strength validation).

   Keep this directory private and persistent; existing credentials are retained
   across restarts unless an explicit password override is configured.

2. ENVIRONMENT VARIABLES
   ADMIN_PASSWORD          — explicitly set or reset the global admin password
   ALLOW_ANONYMOUS_READ    — allow unauthenticated repository reads (default: false)
   CREDENTIALS_DIR         — directory for initial tenant credential files
   BLOB_ROOT_DIR           — root path for tenant blob storage
   MEMORY_MODE=true        — enable in-memory volatile storage
   PORT                    — HTTP listen port (default: 8081)
   PUBLIC_BASE_URL         — externally reachable URL used in package metadata
   RUST_LOG                — log level filter (default: info)
   S3_ENDPOINT             — S3-compatible storage endpoint
   S3_ACCESS_KEY           — S3 access key
   S3_SECRET_KEY           — S3 secret key
   S3_BUCKET               — S3 bucket name
   S3_REGION               — S3 region

3. REST API OVERVIEW
   Base URL: http://<host>:<port>

   Tenants:
     GET    /service/rest/v1/tenants         — list all tenants
     POST   /service/rest/v1/tenants         — create a new tenant
     GET    /service/rest/v1/tenants/:id     — get tenant details

   Repositories:
     GET    /service/rest/v1/repositories    — list all repositories
     POST   /service/rest/v1/repositories/:format/:type — create repository

   Security:
     GET    /service/rest/v1/security/users  — list users
     POST   /service/rest/v1/security/users  — create user
     GET    /service/rest/v1/security/roles  — list roles

   Tokens:
     GET    /service/rest/v1/tokens          — list personal access tokens
     POST   /service/rest/v1/tokens          — create token
     DELETE /service/rest/v1/tokens/:id      — revoke token

   Monitoring:
     GET    /metrics                         — Prometheus metrics
     GET    /service/rest/v1/status          — health check

   Service Logs (per-tenant, best-effort operational history):
     GET    /service/rest/v1/service-logs    — query logs
            ?tenant_id=<id>                  — filter by tenant
            &log_type=service|system         — filter by type
            &username=<name>                 — filter by user
            &action=upload|download          — filter by action
            &size=50                         — page size (max 200)

4. PACKAGE REGISTRY ENDPOINTS
   Maven:  PUT /repository/<name>/<path>       (deploy JARs/POMs)
   Docker: POST /v2/<name>/blobs/uploads/      (initiate push)
   NPM:    PUT /repository/<name>/npm/<pkg>    (npm publish)
   PyPI:   POST /repository/<name>/pypi/upload (twine upload)
   Cargo:  PUT /repository/<name>/api/v1/crates/new
   Go:     GET /repository/<name>/gomod/<path> (GOPROXY)
   NuGet:  PUT /repository/<name>/v3/package   (dotnet nuget push)
   Swift:  PUT /repository/<name>/swift/<scope>/<package>/<version>
           (swift package-registry publish)
   Raw:    PUT /repository/<name>/<path>       (arbitrary files)

5. WEB CONSOLE
   Open http://<host>:<port>/ in a browser for the graphical console.

6. TERMINAL TUI CLIENT
   For SSH bastion / jump host environments:
     registry-tui --endpoint http://<host>:<port> --token tql_pat_xxx

7. MORE INFORMATION
   Full documentation: https://github.com/nichochar/teaql-registry
"#;

pub async fn handle_help() -> Response {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        HELP_TEXT,
    )
        .into_response()
}
