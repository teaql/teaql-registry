# TeaQL Registry Architecture & Design

TeaQL Registry is an extensible multi-format artifact and package repository service designed with Rust and the TeaQL framework.

---

## 1. Core Architectural Principles

1. **Context-Centric Decoupling (`UserContext`)**:
   Infrastructure services (Storage, Format Registries, Multi-Tenancy Policy, Authentication State) are encapsulated as type-safe resources on the TeaQL runtime context (`UserContext` / `ServiceRuntime`).
   Handlers and business services depend only on `ctx: &UserContext` rather than coupling directly with concrete storage engines or complex application structs.

2. **Polymorphic Storage Abstraction (`BlobStore`)**:
   All artifact read, write, check, and deletion workflows operate against the `BlobStore` trait. Concrete backends (S3/RustFS/MinIO, POSIX Filesystem, or In-Memory) can be swapped or dynamically managed without touching API or format engine logic.

3. **Open-Closed Repository Protocol Dispatch (`RepositoryHandler` & `RepositoryRegistry`)**:
   Each package management ecosystem (Docker, Maven, NPM, PyPI, Cargo, Go, NuGet, Raw) is modeled as a pluggable `RepositoryHandler`. Adding new package formats requires implementing the handler interface and registering it, leaving existing dispatcher logic closed for modification.

4. **Runtime Boundary Multi-Tenancy (`NexusTenantRequestPolicy`)**:
   Tenant data boundary enforcement occurs at the TeaQL query execution boundary. Queries automatically inject tenant scoping filters via `RequestPolicy` hooks.

---

## 2. Component Hierarchy

```
                               ┌──────────────────────────────────────────────┐
                               │                 UserContext                  │
                               │  (Passed across handlers & domain services)  │
                               ├──────────────────────────────────────────────┤
                               │  - TenantInfo (Multi-tenant context)         │
                               │  - NexusTenantRequestPolicy (SQL enforcement)│
                               │  - BlobStoreManager (Storage registry)       │
                               │  - RepositoryRegistry (Format handlers)      │
                               └──────────────────────────────────────────────┘
                                                       │
                           ┌───────────────────────────┴───────────────────────────┐
                           ▼                                                       ▼
               ┌───────────────────────┐                               ┌───────────────────────┐
               │   RepositoryHandler   │                               │       BlobStore       │
               └───────────────────────┘                               └───────────────────────┘
                           │                                                       │
         ┌─────────────────┼─────────────────┐                   ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼                   ▼                 ▼                 ▼
   DockerHandler     MavenHandler       NpmHandler          S3BlobStore       FileBlobStore     MemoryBlobStore
 (Docker v2 APIs)   (POM / JAR / Sha)  (Tarball / JSON)   (RustFS / MinIO)   (Local Disk / NFS)  (Unit Testing)
```

---

## 3. Storage Layer (`BlobStore` Trait)

The `BlobStore` trait provides the unified contract for blob lifecycle operations and hash calculations:

```rust
#[async_trait]
pub trait BlobStore: Send + Sync {
    /// Initialize storage backend (e.g. bucket verification/creation)
    async fn init(&self) -> Result<()>;

    /// Store binary data and calculate sha1, sha256, and md5 hashes
    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo>;

    /// Retrieve binary data by blob reference identifier
    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes>;

    /// Delete stored blob
    async fn delete_blob(&self, blob_ref: &str) -> Result<()>;

    /// Verify blob existence
    async fn exists_blob(&self, blob_ref: &str) -> Result<bool>;

    /// Identifier name of the store instance
    fn store_name(&self) -> &str;
}
```

### Storage Implementations:
- **`S3BlobStore`**: Production-ready S3 client with AWS SigV4 request signing. Compatible with **RustFS**, **MinIO**, **AWS S3**, and **Aliyun OSS**.
- **`FileBlobStore`**: Directory-partitioned filesystem store (`/content/ab/ab1234...`).
- **`MemoryBlobStore`**: Concurrent in-memory store for isolated, zero-external-dependency unit tests.
- **`BlobStoreManager`**: Manages dynamic multi-store routing (e.g. per-tenant or per-repository storage targets).

---

## 4. Repository Protocol Layer (`RepositoryHandler`)

A `Repository` in the system is characterized by:
- **Format**: `maven2`, `docker`, `npm`, `pypi`, `cargo`, `gomod`, `nuget`, `raw`.
- **Type**: `hosted` (write/read), `proxy` (remote cache), `group` (virtual router).

The `RepositoryHandler` interface decouples request dispatching from format-specific serialization:

```rust
#[async_trait]
pub trait RepositoryHandler: Send + Sync {
    fn format_name(&self) -> &'static str;

    async fn get(
        &self,
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>>;

    async fn put(
        &self,
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<()>;
}
```

---

## 5. Context Integration (`RegistryContextExt`)

Extension methods implemented on `ServiceRuntime` / `UserContext`:

```rust
pub trait RegistryContextExt {
    fn set_tenant(&mut self, tenant_id: u64, tenant_name: &str);
    fn tenant_id(&self) -> u64;
    fn tenant_name(&self) -> &str;

    fn set_blobstore(&mut self, blobstore: Arc<dyn BlobStore>);
    fn blobstore(&self) -> Arc<dyn BlobStore>;
    fn blobstore_manager(&self) -> Option<Arc<BlobStoreManager>>;

    fn set_repository_registry(&mut self, registry: RepositoryRegistry);
    fn repository_registry(&self) -> Option<Arc<RepositoryRegistry>>;

    fn init_registry_context(&mut self, blobstore: Arc<dyn BlobStore>);
}
```

---

## 6. Testing Strategy

All layers are verified using automated end-to-end and contract tests:
1. **Contract Tests**: `test_*_blobstore_contract` runs identical assertion suites against `MemoryBlobStore`, `FileBlobStore`, and `S3BlobStore` (RustFS).
2. **Protocol Integration Tests**: End-to-end lifecycle verification for Cargo, Docker, Go Modules, Maven, NPM, NuGet, PyPI, and Raw.
3. **Multi-Tenancy Isolation Tests**: Verifies tenant query boundary enforcement and storage isolation.
