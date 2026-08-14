pub mod cargo_registry;
pub mod docker_registry;
pub mod gomod_registry;
pub mod npm_registry;
pub mod nuget_registry;
pub mod pypi_registry;
pub mod repository_content;
pub mod rest_blobstores;
pub mod rest_components;
pub mod rest_repositories;
pub mod rest_security;
pub mod rest_status;
pub mod rest_tenants;

pub use repository_content::AppState;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, patch, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_app(state: AppState) -> Router {
    let rest_router = Router::new()
        // Tenants Management
        .route("/tenants", get(rest_tenants::list_tenants).post(rest_tenants::create_tenant))
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
        .route(
            "/blobstores/s3",
            post(rest_blobstores::create_s3_blobstore),
        )
        .route(
            "/blobstores/file",
            post(rest_blobstores::create_file_blobstore),
        )
        // Security Users, Roles, Privileges, Anonymous
        .route("/security/users", get(rest_security::list_users).post(rest_security::create_user))
        .route("/security/roles", get(rest_security::list_roles))
        .route("/security/privileges", get(rest_security::list_privileges))
        .route(
            "/security/anonymous",
            get(rest_security::get_anonymous_config).put(rest_security::update_anonymous_config),
        )
        // Components and Assets
        .route("/components", get(rest_components::list_components))
        .route("/assets", get(rest_components::list_assets))
        // Status
        .route("/status", get(rest_status::status_ok))
        .route("/status/writable", get(rest_status::status_writable));

    let docker_repo_router = Router::new()
        .route("/tags/list", get(docker_registry::handle_tags_list))
        .route("/blobs/uploads/", post(docker_registry::handle_blob_upload_init))
        .route(
            "/blobs/uploads/:uuid",
            patch(docker_registry::handle_blob_upload_chunk).put(docker_registry::handle_blob_upload_finish),
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
        // Docker Registry v2 ping & repo operations
        .route("/v2", get(docker_registry::handle_v2_ping))
        .route("/v2/", get(docker_registry::handle_v2_ping))
        .nest("/v2/:name", docker_repo_router)
        // Cargo Sparse Index & Crates
        .route("/repository/:name/config.json", get(cargo_registry::handle_cargo_config))
        .route("/repository/:name/api/v1/crates/:crate/:version/download", get(cargo_registry::handle_cargo_download))
        .route("/repository/:name/api/v1/crates/new", put(cargo_registry::handle_cargo_publish))
        .route("/repository/:name/cargo/index/*index_path", get(cargo_registry::handle_cargo_sparse_index))
        // PyPI Simple Index & Upload
        .route("/repository/:name/simple/", get(pypi_registry::handle_pypi_simple_root))
        .route("/repository/:name/simple/:project/", get(pypi_registry::handle_pypi_simple_package))
        .route("/repository/:name/packages/:filename", get(pypi_registry::handle_pypi_get_package_file))
        .route("/repository/:name/pypi/upload", post(pypi_registry::handle_pypi_upload))
        // NuGet v3
        .route("/repository/:name/v3/index.json", get(nuget_registry::handle_nuget_service_index))
        .route("/repository/:name/v3/flatcontainer/:id/index.json", get(nuget_registry::handle_nuget_package_versions))
        .route("/repository/:name/v3/flatcontainer/:id/:version/:package_file", get(nuget_registry::handle_nuget_get_package))
        .route("/repository/:name/v3/package", put(nuget_registry::handle_nuget_push))
        // NPM
        .route("/repository/:name/npm/:package_name", get(npm_registry::handle_npm_get_package).put(npm_registry::handle_npm_publish))
        .route("/repository/:name/npm/:package_name/-/:tarball", get(npm_registry::handle_npm_get_tarball))
        // Go Modules (GOPROXY)
        .route("/repository/:name/gomod/*path", get(gomod_registry::handle_gomod_get).put(gomod_registry::handle_gomod_put))
        // Generic Maven & Raw repository content: /repository/:name/*path
        .route(
            "/repository/:name/*path",
            get(repository_content::handle_get_content)
                .head(repository_content::handle_head_content)
                .put(repository_content::handle_put_content),
        )
        // REST API v1
        .nest("/service/rest/v1", rest_router)
        .layer(DefaultBodyLimit::disable())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
