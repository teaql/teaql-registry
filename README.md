# TeaQL Registry

TeaQL Registry 是基于 TeaQL 框架与 Rust 构建的通用软件包与制品仓库服务。

---

## 核心特性

- **多包格式支持（8 种包管理器）**：
  - **Docker Registry v2**：支持 Docker Client 镜像分层推送、拉取、分块上传与 Manifest 管理。
  - **Maven2**：支持 Release / Snapshot 构件（POM、JAR、SHA1/MD5 校验和）上传与拉取。
  - **NPM**：支持标准 NPM CLI 协议的包发布、元数据查询与 Tarball 下载。
  - **PyPI**：支持 Python Wheel / Tarball 发布与 Simple Index (`/simple/`) 索引查询及 `pip` 安装。
  - **Go Modules**：符合 GOPROXY 规范的 `@v/list`、`.info`、`.mod` 与 `.zip` 接口。
  - **Cargo (Rust)**：支持 Cargo Sparse Index 稀疏索引与 Crate 发布/下载。
  - **NuGet (.NET)**：支持 NuGet v3 Service Index、Flat Container 与 `dotnet nuget push`。
  - **Raw**：支持任意结构化文件的 HTTP PUT/GET 直传。
- **多租户隔离**：
  - 在运行时边界（`UserContext` 的 `NexusTenantRequestPolicy`）集中实施租户数据范围约束，无需在服务层手动拼接过滤条件。
- **存储与安全体系**：
  - 基于本地文件系统（FileBlobStore）的分块去重与内容校验（SHA-1、SHA-256、MD5）。
  - 基于 RBAC 的权限模型、用户管理与匿名访问控制。

---

## 模块结构

```text
├── models/             # TeaQL 实体模型定义 (model.xml)
├── rust-lib-core/      # 由官方代码生成器产出的核心模型与查询层
├── rust-app-console/   # 格式引擎、存储服务、REST API 与测试套件
└── demo-components/    # 客户端调用示例与集成配置
```

---

## 运行与测试

### 1. 数据库准备
服务依赖 PostgreSQL 数据库存储元数据。

```bash
# 环境变量配置示例
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nexus_db"
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_USER="postgres"
export NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_PASSWORD="postgres"
export PORT=8081
```

### 2. 运行测试套件
```bash
cargo test -- --test-threads=1
```

### 3. 启动服务
```bash
cargo run --bin nexus-repository-service-core-workspace
```

启动后，服务将在 `http://0.0.0.0:8081` 监听请求。
