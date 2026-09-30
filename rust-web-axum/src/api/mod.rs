pub mod cargo_registry;
pub mod composer_registry;
pub mod conan_registry;
pub mod dart_registry;
pub mod docker_registry;
pub mod gomod_registry;
pub mod help;
pub mod hex_registry;
pub mod metrics;
pub mod npm_registry;
pub mod nuget_registry;
pub mod pypi_registry;
pub mod repository_content;
pub mod rest_blobstores;
pub mod rest_components;
pub mod rest_governance;
pub mod rest_repositories;
pub mod rest_security;
pub mod rest_service_logs;
pub mod rest_status;
pub mod rest_tenants;
pub mod rest_tokens;
pub mod rest_webhooks;
pub mod rubygems_registry;
pub mod search;
pub mod swift_registry;

pub use repository_content::AppState;

use crate::security::request_context::authenticate_request;
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, patch, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

pub(crate) fn public_base_url(headers: &axum::http::HeaderMap) -> String {
    if let Ok(configured) = std::env::var("PUBLIC_BASE_URL") {
        let configured = configured.trim().trim_end_matches('/');
        if !configured.is_empty() {
            return configured.to_string();
        }
    }

    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| *value == "http" || *value == "https")
        .unwrap_or("http");
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(axum::http::header::HOST))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("localhost:8081");
    format!("{scheme}://{host}")
}

pub fn build_app(state: AppState) -> Router {
    let rest_router = Router::new()
        // Tenants Management
        .route(
            "/tenants",
            get(rest_tenants::list_tenants).post(rest_tenants::create_tenant),
        )
        .route("/tenants/:id", get(rest_tenants::get_tenant))
        // Repositories
        .route("/repositories", get(rest_repositories::list_repositories))
        .route(
            "/repositories/:format/:type/:name",
            get(rest_repositories::get_repository),
        )
        .route(
            "/repositories/:format/:type",
            post(rest_repositories::create_repository),
        )
        // Blobstores
        .route("/blobstores", get(rest_blobstores::list_blobstores))
        .route("/blobstores/s3", post(rest_blobstores::create_s3_blobstore))
        .route(
            "/blobstores/file",
            post(rest_blobstores::create_file_blobstore),
        )
        // Security Users, Roles, Privileges, Anonymous
        .route(
            "/security/users",
            get(rest_security::list_users).post(rest_security::create_user),
        )
        .route("/security/roles", get(rest_security::list_roles))
        .route("/security/privileges", get(rest_security::list_privileges))
        .route(
            "/security/anonymous",
            get(rest_security::get_anonymous_config),
        )
        // Personal Access Tokens
        .route(
            "/tokens",
            get(rest_tokens::handle_list_tokens).post(rest_tokens::handle_create_token),
        )
        .route(
            "/tokens/:id",
            axum::routing::delete(rest_tokens::handle_revoke_token),
        )
        // Webhooks
        .route(
            "/webhooks",
            get(rest_webhooks::handle_list_webhooks).post(rest_webhooks::handle_create_webhook),
        )
        .route(
            "/webhooks/:id",
            axum::routing::delete(rest_webhooks::handle_delete_webhook),
        )
        .route("/webhooks/test", post(rest_webhooks::handle_test_webhook))
        // Governance & Storage Ops
        .route("/gc/run", post(rest_governance::handle_run_gc))
        .route("/cleanup/run", post(rest_governance::handle_run_cleanup))
        // Components and Assets Search & Management
        .route("/components", get(rest_components::list_components))
        .route("/assets", get(rest_components::list_assets))
        .route("/search", get(search::handle_search_components))
        .route("/search/assets", get(search::handle_search_assets))
        // Status
        .route("/status", get(rest_status::status_ok))
        .route("/status/writable", get(rest_status::status_writable))
        // Service Logs (per-tenant best-effort operational history)
        .route("/service-logs", get(rest_service_logs::list_service_logs));

    let docker_repo_router = Router::new()
        .route("/tags/list", get(docker_registry::handle_tags_list))
        .route(
            "/blobs/uploads/",
            post(docker_registry::handle_blob_upload_init),
        )
        .route(
            "/blobs/uploads/:uuid",
            patch(docker_registry::handle_blob_upload_chunk)
                .put(docker_registry::handle_blob_upload_finish),
        )
        .route(
            "/blobs/:digest",
            get(docker_registry::handle_blob_get).head(docker_registry::handle_blob_head),
        )
        .route(
            "/manifests/:reference",
            get(docker_registry::handle_manifest_get)
                .head(docker_registry::handle_manifest_head)
                .put(docker_registry::handle_manifest_put),
        );

    Router::new()
        // Embedded UI Console
        .route("/", get(crate::ui::handle_index))
        .route("/ui", get(crate::ui::handle_index))
        .route("/console", get(crate::ui::handle_index))
        .route("/help", get(help::handle_help))
        .route("/assets/*path", get(crate::ui::handle_assets))
        // Prometheus Metrics
        .route("/metrics", get(metrics::handle_metrics))
        // Docker Registry v2 ping & repo operations
        .route("/v2", get(docker_registry::handle_v2_ping))
        .route("/v2/", get(docker_registry::handle_v2_ping))
        .nest("/v2/:name", docker_repo_router)
        // Cargo Sparse Index & Crates
        .route(
            "/repository/:name/config.json",
            get(cargo_registry::handle_cargo_config),
        )
        .route(
            "/repository/:name/cargo/index/config.json",
            get(cargo_registry::handle_cargo_config),
        )
        .route(
            "/repository/:name/api/v1/crates/:crate/:version/download",
            get(cargo_registry::handle_cargo_download),
        )
        .route(
            "/repository/:name/api/v1/crates/new",
            put(cargo_registry::handle_cargo_publish).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/cargo/index/*index_path",
            get(cargo_registry::handle_cargo_sparse_index),
        )
        // PyPI Simple Index & Upload
        .route(
            "/repository/:name/simple/",
            get(pypi_registry::handle_pypi_simple_root),
        )
        .route(
            "/repository/:name/simple/:project/",
            get(pypi_registry::handle_pypi_simple_package),
        )
        .route(
            "/repository/:name/packages/:filename",
            get(pypi_registry::handle_pypi_get_package_file),
        )
        .route(
            "/repository/:name/pypi/upload",
            post(pypi_registry::handle_pypi_upload).layer(DefaultBodyLimit::disable()),
        )
        // NuGet v3
        .route(
            "/repository/:name/v3/index.json",
            get(nuget_registry::handle_nuget_service_index),
        )
        .route(
            "/repository/:name/v3/flatcontainer/:id/index.json",
            get(nuget_registry::handle_nuget_package_versions),
        )
        .route(
            "/repository/:name/v3/flatcontainer/:id/:version/:package_file",
            get(nuget_registry::handle_nuget_get_package),
        )
        .route(
            "/repository/:name/v3/package",
            put(nuget_registry::handle_nuget_push).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/v3/package/",
            put(nuget_registry::handle_nuget_push).layer(DefaultBodyLimit::disable()),
        )
        // NPM
        .route(
            "/repository/:name/npm/:package_name",
            get(npm_registry::handle_npm_get_package).put(npm_registry::handle_npm_publish),
        )
        .route(
            "/repository/:name/npm/:package_name/-/:tarball",
            get(npm_registry::handle_npm_get_tarball),
        )
        // Go Modules (GOPROXY)
        .route(
            "/repository/:name/gomod/*path",
            get(gomod_registry::handle_gomod_get)
                .put(gomod_registry::handle_gomod_put)
                .layer(DefaultBodyLimit::disable()),
        )
        // Swift Package Registry (SE-0292 / SE-0391)
        .route(
            "/repository/:name/swift/identifiers",
            get(swift_registry::handle_swift_identifiers),
        )
        .route(
            "/repository/:name/swift/:scope/:package/:version/Package.swift",
            get(swift_registry::handle_swift_manifest),
        )
        .route(
            "/repository/:name/swift/:scope/:package/:resource",
            get(swift_registry::handle_swift_release_resource)
                .put(swift_registry::handle_swift_publish)
                .layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/swift/:scope/:package",
            get(swift_registry::handle_swift_list_releases),
        )
        .route(
            "/repository/:name/swift",
            axum::routing::options(swift_registry::handle_swift_options),
        )
        // Dart Hosted Pub Repository v2
        .route(
            "/repository/:name/dart/api/packages/versions/new",
            get(dart_registry::handle_dart_new_upload),
        )
        .route(
            "/repository/:name/dart/api/packages/versions/newUpload",
            post(dart_registry::handle_dart_upload).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/dart/api/packages/versions/newUpload/complete/:package/:version",
            get(dart_registry::handle_dart_finalize),
        )
        .route(
            "/repository/:name/dart/api/packages/:package",
            get(dart_registry::handle_dart_versions),
        )
        .route(
            "/repository/:name/dart/api/packages/:package/versions/:version",
            get(dart_registry::handle_dart_version),
        )
        .route(
            "/repository/:name/dart/packages/:package/versions/:archive",
            get(dart_registry::handle_dart_archive),
        )
        // RubyGems push, download, and Bundler Compact Index
        .route(
            "/repository/:name/rubygems/api/v1/gems",
            post(rubygems_registry::handle_rubygems_push).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/rubygems/gems/:filename",
            get(rubygems_registry::handle_rubygems_download),
        )
        .route(
            "/repository/:name/rubygems/versions",
            get(rubygems_registry::handle_rubygems_versions),
        )
        .route(
            "/repository/:name/rubygems/info/:gem_name",
            get(rubygems_registry::handle_rubygems_info),
        )
        .route(
            "/repository/:name/rubygems/names",
            get(rubygems_registry::handle_rubygems_names),
        )
        // Composer 2 metadata and dist archives
        .route(
            "/repository/:name/composer/packages.json",
            get(composer_registry::handle_composer_root),
        )
        .route(
            "/repository/:name/composer/p2/*package_path",
            get(composer_registry::handle_composer_p2),
        )
        .route(
            "/repository/:name/composer/dist/:vendor/:package/:filename",
            get(composer_registry::handle_composer_dist)
                .put(composer_registry::handle_composer_publish)
                .layer(DefaultBodyLimit::disable()),
        )
        // Conan 2 revisions REST API
        .route(
            "/repository/:name/conan/v1/ping",
            get(conan_registry::handle_conan_ping),
        )
        .route(
            "/repository/:name/conan/v2/users/authenticate",
            get(conan_registry::handle_conan_authenticate),
        )
        .route(
            "/repository/:name/conan/v2/users/check_credentials",
            get(conan_registry::handle_conan_check_credentials),
        )
        .route(
            "/repository/:name/conan/v2/conans/search",
            get(conan_registry::handle_conan_search),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions",
            get(conan_registry::handle_conan_recipe_revisions),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/latest",
            get(conan_registry::handle_conan_recipe_latest),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/files",
            get(conan_registry::handle_conan_recipe_snapshot),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/files/*file_path",
            get(conan_registry::handle_conan_recipe_file_get)
                .put(conan_registry::handle_conan_recipe_file_put)
                .layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/search",
            get(conan_registry::handle_conan_package_search),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/packages/:package_id/revisions",
            get(conan_registry::handle_conan_package_revisions),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/packages/:package_id/latest",
            get(conan_registry::handle_conan_package_latest),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/packages/:package_id/revisions/:package_revision/files",
            get(conan_registry::handle_conan_package_snapshot),
        )
        .route(
            "/repository/:name/conan/v2/conans/:package/:version/:user/:channel/revisions/:recipe_revision/packages/:package_id/revisions/:package_revision/files/*file_path",
            get(conan_registry::handle_conan_package_file_get)
                .put(conan_registry::handle_conan_package_file_put)
                .layer(DefaultBodyLimit::disable()),
        )
        // Hex publish API and signed Registry v2
        .route(
            "/repository/:name/hex/api/publish",
            post(hex_registry::handle_hex_publish).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/repository/:name/hex/repo/public_key",
            get(hex_registry::handle_hex_public_key),
        )
        .route(
            "/repository/:name/hex/repo/names",
            get(hex_registry::handle_hex_names),
        )
        .route(
            "/repository/:name/hex/repo/versions",
            get(hex_registry::handle_hex_versions),
        )
        .route(
            "/repository/:name/hex/repo/packages/:package",
            get(hex_registry::handle_hex_package),
        )
        .route(
            "/repository/:name/hex/repo/tarballs/:filename",
            get(hex_registry::handle_hex_tarball),
        )
        // Generic Maven & Raw repository content: /repository/:name/*path
        .route(
            "/repository/:name/*path",
            get(repository_content::handle_get_content)
                .head(repository_content::handle_head_content)
                .put(repository_content::handle_put_content)
                .layer(DefaultBodyLimit::disable()),
        )
        // REST API v1
        .nest("/service/rest/v1", rest_router)
        // Streaming handlers do not buffer this amount; the layer is a hard
        // safety ceiling for both streaming and legacy protocol extractors.
        .layer(RequestBodyLimitLayer::new(5 * 1024 * 1024 * 1024usize))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_request,
        ))
        .with_state(state)
}
