# TeaQL Registry

TeaQL Registry is a multi-format package and artifact repository service built with the TeaQL framework and Rust.

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
  - Tenant data boundaries are enforced centrally at the runtime boundary via `NexusTenantRequestPolicy` on `UserContext`.
- **Storage & Security**:
  - Local filesystem blob storage (`FileBlobStore`) with content hashing and integrity checks (SHA-1, SHA-256, MD5).
  - Role-based access control (RBAC), privilege matching, and anonymous access management.

---

## Repository Structure

```text
├── models/             # TeaQL domain entity definitions (model.xml)
├── rust-lib-core/      # Generated core models and query layer
├── rust-app-console/   # Format engines, storage service, REST APIs, and test suite
└── demo-components/    # Client integration examples and configurations
```

---

## Getting Started

### 1. Database Configuration
The service requires PostgreSQL for metadata persistence.

```bash
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nexus_db"
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_USER="postgres"
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_PASSWORD="postgres"
export PORT=8081
```

### 2. Run Tests
```bash
cargo test -- --test-threads=1
```

### 3. Start Service
```bash
cargo run --bin nexus-repository-service-core-workspace
```

The service will listen on `http://0.0.0.0:8081`.
