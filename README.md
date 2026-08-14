# TeaQL Registry

TeaQL Registry is a high-performance, extensible multi-format package and artifact repository service built with the TeaQL framework and Rust.

---

## Features

- **Multi-Format Package Support (8 Package Managers)**:
  - **Docker Registry v2**: Full support for Docker CLI image layer push/pull, monolithic and chunked uploads, and manifest management.
  - **Maven2**: Release and Snapshot artifact uploads/downloads (POM, JAR, and SHA-1/MD5 checksum verification).
  - **NPM**: Standard NPM CLI package publish, metadata retrieval, and tarball distribution.
  - **PyPI**: Python Wheel/Tarball distribution, Simple Index (`/simple/`) endpoints, and `pip` installation.
  - **Go Modules**: GOPROXY specification compliance (`@v/list`, `.info`, `.mod`, `.zip`).
  - **Cargo (Rust)**: Cargo Sparse Index support and crate publishing/downloading.
  - **NuGet (.NET)**: NuGet v3 Service Index, Flat Container endpoints, and `dotnet nuget push`.
  - **Raw**: Arbitrary binary and document file uploads and downloads over HTTP.
- **Runtime-Boundary Multi-Tenancy**:
  - Tenant data boundaries are enforced centrally at the runtime boundary via `TeaQLRegistryTenantRequestPolicy` on `UserContext`.
- **Pluggable Storage Abstraction**:
  - Polymorphic `BlobStore` trait supporting S3 Object Storage (`S3BlobStore` with RustFS / MinIO / AWS S3), POSIX filesystem (`FileBlobStore`), and memory (`MemoryBlobStore`).
  - Streaming uploads, chunking, and automated content hashing (SHA-1, SHA-256, MD5).
- **Security & Access Control**:
  - Role-based access control (RBAC), privilege matching, and anonymous access management.

---

## Repository Structure

```text
├── models/             # TeaQL domain entity definitions (model.xml)
├── rust-lib-core/      # Generated core models and query layer (teaql-registry-core)
├── rust-web-axum/      # Format engines, storage service, REST APIs, and test suite (teaql-registry)
└── demo-components/    # Client integration examples and configurations
```

---

## Getting Started

### 1. Start S3 Storage (RustFS / MinIO)
```bash
docker run -d --name teaql-rustfs \
  -p 9010:9000 -p 9011:9001 \
  -e RUSTFS_ACCESS_KEY=rustfsadmin \
  -e RUSTFS_SECRET_KEY=rustfsadmin \
  rustfs/rustfs:latest
```

### 2. Environment Configuration
```bash
export TEAQL_REGISTRY_CORE_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nexus_db"
export TEAQL_REGISTRY_CORE_DATABASE_USER="postgres"
export TEAQL_REGISTRY_CORE_DATABASE_PASSWORD="postgres"
export S3_ENDPOINT="http://127.0.0.1:9010"
export S3_ACCESS_KEY="rustfsadmin"
export S3_SECRET_KEY="rustfsadmin"
export S3_BUCKET="teaql-blobs"
export S3_REGION="us-east-1"
export PORT=8081
```

### 3. Run Tests
```bash
cargo test -- --test-threads=1
```

### 4. Start Service
```bash
cargo run --bin teaql-registry
```

The service will listen on `http://0.0.0.0:8081`.
