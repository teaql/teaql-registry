# TeaQL Registry

> **AI-Native Multi-Format Artifact Registry for Autonomous AI Coding Workflows & High-Throughput CI/CD**

---

## Overview & Core Positioning

**TeaQL Registry** is both a practical, AI-native artifact registry and a reference application demonstrating how TeaQL can be used to build a real-world, multi-protocol infrastructure service.

It is purpose-built as an **in-cluster / local intermediate artifact exchange and caching hub for AI Coding Agents and high-throughput CI/CD pipelines**.

```mermaid
graph LR
    subgraph "AI Agent & CI/CD Pipeline (High Frequency Loops)"
        A[AI Agent / CI Runner] -->|1. Rapid Micro-Build & Verification| B[(TeaQL Registry - Local Hub)]
        B -.->|2. Automatic GC for Failed / Experimental Builds| D[BlobStore GC & Retention]
    end
    B -->|3. Publish Final Verified Release Only| C[(Enterprise Master Registry: Nexus / JFrog)]
```

### Why AI-Native Coding Workflows Need a Dedicated Registry

Autonomous AI coding agents (such as SWE-bench benchmark runners, code-repair agents, and multi-agent systems) operate in rapid, autonomous execution loops:  
`Write Code -> Package -> Deploy / Test -> Observe Feedback -> Patch & Re-test`.

1. **Zero Rate Limiting & No WAF Blocking**:
   - AI agents generate hundreds of micro-releases and package queries in minutes. Standard cloud registries (e.g. GitHub Packages, Docker Hub, npmjs) quickly throttle these bursts with HTTP 429 (Too Many Requests) or WAF bot-challenge blocks.
2. **Sub-5ms Intra-Cluster Latency**:
   - Local or cluster-internal loopback routing cuts round-trip times from 200ms+ down to 1–5ms, eliminating network bottlenecks in tight AI execution loops.
3. **Air-Gapped & Egress-Controlled Sandbox Compliance**:
   - In enterprise security setups, AI sandboxes are blocked from accessing public internet to prevent prompt and source code leakage. TeaQL Registry acts as an internal proxy cache and private package repository.
4. **Inter-Agent Artifact Exchange**:
   - Allows multiple cooperating AI agents to share intermediate SNAPSHOTs, wheels, crates, and containers across microservices without external exposure.
5. **Aggressive Ephemeral Lifecycle & Garbage Collection**:
   - Automatically prunes obsolete build artifacts and collects orphaned binary blobs via content-addressed deduplication (SHA-256).

---

## Complementary to Nexus & JFrog (Not a Replacement)

TeaQL Registry is **not intended to replace enterprise master registries** like Sonatype Nexus or JFrog Artifactory. Instead, it serves as an **upstream high-speed L1 cache and staging layer**:

| Dimension | Enterprise Master (Nexus / JFrog Artifactory) | TeaQL Registry (Near-Edge Hub) |
| :--- | :--- | :--- |
| **Primary Role** | **Final Releases & Permanent Archival** | **AI Agent Loops & Ephemeral CI/CD Intermediate Builds** |
| **Retention Policy** | Long-term immutable version storage | Short-lived (minutes/hours), high churn, auto GC |
| **Security Scope** | Deep SBOM compliance, Xray vulnerability scanning | High-throughput streaming, SHA-256 deduplication, proxy caching |
| **Deployment** | Centralized enterprise datacenter or managed cloud | Sidecar / local container co-located with AI sandboxes and runners |

---

## Key Features

- **8 Package Ecosystems Supported Out of the Box**:
  - **Docker Registry v2**: Monolithic & chunked layer push/pull, manifest management.
  - **Maven2**: Release and snapshot JAR/POM uploads, SHA-1 / SHA-256 checksums.
  - **NPM**: Standard `npm publish` / `npm install` and tarball distribution.
  - **PyPI**: Python Wheel/Tarball hosting and Simple Index (`/simple/`).
  - **Cargo (Rust)**: Sparse Index protocol support, crate publish and download.
  - **Go Modules**: GOPROXY specification compliance (`.info`, `.mod`, `.zip`).
  - **NuGet (.NET)**: NuGet v3 Flat Container protocol and `dotnet nuget push`.
  - **Raw**: Arbitrary binary tools, archives, and files over HTTP.
- **Pure In-Memory High-Performance Mode (`--memory-mode` / `MEMORY_MODE=true`)**:
  - In-memory volatile RAM blob storage designed for specialized scenarios requiring maximum throughput and zero I/O latency.
  - **Strict Single Latest Version Retention**: Every artifact automatically evicts prior versions upon publishing a new version to bound memory consumption.
  - **Estimated Memory Footprint**:
    - *Code-only dependencies (100–200 NPM/Cargo/PyPI/Go/Java modules)*: ~100MB – 300MB RAM.
    - *Containers & binaries (30–50 microservice images / executables)*: ~1GB – 3GB RAM.
  - **Recommendation**: For the vast majority of workflows, standard **filesystem storage** (or local S3) is the recommended default and fully sufficient, delivering sub-5ms latency via OS page cache without significant RAM overhead.
- **Embedded Snowflake-Style Web Console**:
  - Modern React 19 + TypeScript SPA embedded directly inside the binary (zero static file dependencies).
  - Low cognitive load, instant package manager install snippet generators, and storage metrics.
- **Standalone Terminal TUI Client (`registry-tui`)**:
  - Zero-dependency, 2.6MB single static binary built with Ratatui for SSH jump host / bastion environments.
- **Automated Lifecycle Governance & GC**:
  - Retention policies (keep latest N versions, snapshot cleanup) and physical orphaned blob deletion.
- **CI/CD Security & Integration**:
  - Personal Access Tokens (`tql_pat_*`) with scope enforcement and HMAC-signed webhook event delivery.
- **Polymorphic Storage Engine**:
  - Pluggable S3-compatible backend (RustFS / MinIO / AWS S3), POSIX filesystem, or in-memory storage with content-addressed SHA-256 deduplication.

---

## Project Structure

```text
teaql-registry/
├── console/             # Snowflake-style React 19 + TypeScript web console
├── registry-tui/        # Standalone terminal TUI client (registry-tui)
├── models/              # TeaQL domain entity and metadata schema (model.xml)
├── rust-lib-core/       # Type-safe model operations & audited data layer (teaql-registry-core)
├── rust-web-axum/       # Protocol engines, S3 storage, REST APIs & embedded UI (teaql-registry)
├── demo-components/     # Demo sample artifacts for all 8 formats & publish script
└── docker-compose.yml   # Complete environment setup (PostgreSQL + RustFS + Registry)
```

---

## Quick Start

### 1. Run with Docker Compose (Recommended)

```bash
docker compose up -d
```

Service endpoints will be available at:
- **Web Console**: `http://localhost:8081/` (Default credentials: `admin` / `admin123`)
- **REST API**: `http://localhost:8081/service/rest/v1/...`
- **Prometheus Metrics**: `http://localhost:8081/metrics`
- **S3 Object Storage (RustFS)**: `http://localhost:9010`

### 2. Run the Terminal TUI Client (`registry-tui`)

Designed for bastion hosts and SSH sessions where browser HTTP access is restricted:

```bash
# Run directly via Cargo
cargo run --release -p registry-tui

# Connect to a remote internal endpoint with a Personal Access Token
registry-tui --endpoint http://10.0.0.10:8081 --token tql_pat_xxx
```

### 3. Seed Demo Packages (All 8 Formats)

Publish live sample artifacts across all 8 ecosystems in one command:
```bash
./demo-components/publish_all_demos.sh
```

### 4. Build and Run from Source

```bash
# 1. Start storage and database dependencies
docker compose up -d postgres rustfs

# 2. Configure environment and launch service
export TEAQL_REGISTRY_CORE_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nexus_db"
export TEAQL_REGISTRY_CORE_DATABASE_USER="postgres"
export TEAQL_REGISTRY_CORE_DATABASE_PASSWORD="postgres"
export S3_ENDPOINT="http://127.0.0.1:9010"
export S3_ACCESS_KEY="rustfsadmin"
export S3_SECRET_KEY="rustfsadmin"
export S3_BUCKET="teaql-blobs"
export S3_REGION="us-east-1"
export PORT=8081

cargo run --bin teaql-registry
```

### 5. Pure In-Memory High-Performance Mode

> **Resource Sizing & Guidance**: In-memory mode holds all binary payloads directly in RAM and consumes significantly more memory (estimated **100MB–300MB** for code packages; **1GB–3GB** if storing container images/binaries). Standard **filesystem or local S3 storage is the recommended default** for almost all workflows. Use memory mode specifically when running inside ephemeral stateless containers or when benchmarking ultra-low-latency agent loops.

```bash
# Via CLI flag
cargo run --bin teaql-registry -- --memory-mode

# Or via environment variable
MEMORY_MODE=true cargo run --bin teaql-registry
```

### 6. Run the Test Suite

```bash
cargo test -- --test-threads=1
```

---

## License

Dual-licensed under MIT OR Apache-2.0.
