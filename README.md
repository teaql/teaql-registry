# TeaQL Registry

TeaQL Registry 是一个基于 Rust 与 TeaQL 框架构建的高性能、超轻量多格式制品库服务，**核心定位是为 AI 自动化流水线（Agent Loops）与高频 CI/CD 提供低延迟、无外部限流的临时中间制品高速中转与缓存中枢**。

---

## 核心定位与设计目标

与面向长期归档和合规审计的传统大型制品库（如 JFrog / Nexus）不同，TeaQL Registry 专为高频、密集的自动化构建与验证流程设计：

1. **服务于 AI 自动化编程流水线（Agent Loops）**：
   - AI 编码智能体在“生成代码 -> 构建打包 -> 测试运行 -> 修正重试”的快速闭环中，会在极短时间内产生大量微小迭代包与高密 API 请求。
   - 本地化或集群内网部署，消除外网往返延迟（RTT 从 200ms+ 降至 1~5ms），且**无外部公有云的 API 频次限制与 WAF 风控拦截（避免 429 Too Many Requests）**。
2. **高频 CI/CD 流水线中间制品中转**：
   - 为跨 Stage 构建产物传递、分支构建（PR / Feature Branch）提供快速存取。
   - 内置保留策略（Retention Policy）与垃圾回收（BlobStore GC），支持按版本数量自动淘汰临时制品，防止存储堆积。
3. **极低资源开销与就近部署**：
   - **单二进制 / 6.5MB 极简容器**，常驻内存仅 20MB~40MB，冷启动毫秒级，可作为 Sidecar 或集群本地服务与 CI Runner 同机部署。

---

## 核心特性

- **8 大主流包管理生态全覆盖**：
  - **Docker Registry v2**：支持 Docker CLI 分层推送拉取、块上传与 Manifest 校验。
  - **Maven2**：支持 Release / Snapshot 构建（JAR / POM / SHA-1 / SHA-256 校验）。
  - **NPM**：支持 `npm publish` / `npm install` 与 Tarball 分发。
  - **PyPI**：支持 Python Wheel / Tarball 与 Simple Index（`/simple/`）。
  - **Cargo (Rust)**：支持 Sparse Index 协议与 crate 发布下载。
  - **Go Modules**：符合 GOPROXY 标准规范（`.info`, `.mod`, `.zip`）。
  - **NuGet (.NET)**：支持 NuGet v3 Flat Container 与 `dotnet nuget push`。
  - **Raw**：支持任意通用二进制工具链、压缩包与文档文件的直接存取。
- **Snowflake 风格轻量 Web 控制台**：
  - 内嵌 React 19 + TypeScript 控制台，提供仓库概览、构件多维检索、一键复制包管理器安装命令、存储运维与 PAT 令牌管理。
- **自动化存储治理与垃圾回收**：
  - 支持多维保留策略（最大版本保留数、快照清理），以及底层孤儿 Blob 的物理垃圾回收。
- **CI/CD 认证与集成**：
  - 支持 Personal Access Token（PAT，`tql_pat_*`）与 Webhook 事件通知，支持主流 CLI 工具免配置集成。
- **多后端存储抽象**：
  - 支持 S3 对象存储（兼容 RustFS / MinIO / AWS S3）、本地文件系统与内存存储，底层采用 SHA-256 内容寻址自动去重。

---

## 目录结构

```text
teaql-registry/
├── console/             # Snowflake 风格 React 19 前端控制台源码
├── models/              # TeaQL 领域实体与元数据定义 (model.xml)
├── rust-lib-core/       # 强类型元数据操作与审计层 (teaql-registry-core)
├── rust-web-axum/       # 协议引擎、S3 存储流、REST API 与内嵌 UI (teaql-registry)
├── demo-components/     # 8 大包格式的演示构件与一键发布脚本
└── docker-compose.yml   # 一键拉起 PostgreSQL + RustFS + TeaQL Registry 环境
```

---

## 快速上手

### 方式一：Docker Compose 一键启动（推荐）

```bash
docker compose up -d
```

启动完成后，系统各端点如下：
- **Web 控制台**：`http://localhost:8081/`（默认管理员账号：`admin` / `admin123`）
- **REST API**：`http://localhost:8081/service/rest/v1/...`
- **Prometheus 监控指标**：`http://localhost:8081/metrics`
- **S3 对象存储（RustFS）**：`http://localhost:9010`

### 方式二：发布全格式演示构件

执行自带脚本，一键向运行中的服务推送全部 8 种格式的示例包：
```bash
./demo-components/publish_all_demos.sh
```

### 方式三：本地源码编译运行

1. **启动依赖存储与数据库**：
   ```bash
   docker compose up -d postgres rustfs
   ```

2. **编译并启动后端**：
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

   cargo run --bin teaql-registry
   ```

3. **运行全量测试套件**：
   ```bash
   cargo test -- --test-threads=1
   ```
