# TeaQL Registry

> **AI-Native Multi-Format Artifact Registry for AI Coding Workflows**  
> 专为 AI Native 自动化编程工作流与高频 CI/CD 流水线设计的超轻量、低延迟多格式制品中枢。

---

## 核心定位：AI Native 编码工作流基础设施

在 AI Agent 驱动的代码生成与自主验证（Agentic Coding Loops）场景中，智能体在“编写代码 -> 打包构建 -> 运行测试 -> 观察报错 -> 修正重试”的高密闭环中运行。

传统面向长期归档的企业级制品库（如 JFrog / Nexus）在 AI 工作流中面临重型占用、网络延迟及云端频次风控等瓶颈。TeaQL Registry 专为 **AI Native Coding Workflow** 设计：

1. **规避公网风控与 API 限流（Zero Throttling & WAF Free）**：
   - AI Agent 密集的构建发布往往在短时间内产生上千次请求，自建内网节点彻底杜绝公有云 429（Too Many Requests）或 WAF 恶意行为拦截。
2. **毫秒级内网读写延迟（<5ms Latency）**：
   - 本地化或容器集群内网直通传输，消除公网 200ms+ 的往返延迟，显著提升单个 AI 修复循环的执行效率。
3. **隔离沙箱与数据防泄露（Air-Gapped Sandbox Ready）**：
   - 适配禁止出网的 AI 执行沙箱，作为局域网内的依赖 Proxy 缓存与私有包托管中枢，保障源码与提示词安全。
4. **多 Agent 协同构建中转（Inter-Agent Dependency Resolution）**：
   - 支撑微服务或多模块场景下多个 AI Agent 之间传递临时 SNAPSHOT 或预编译构建产物。
5. **针对临时制品的自动垃圾回收（Retention & GC）**：
   - 内置保留策略与孤儿 Blob 回收，按版本数量自动清理中间实验产物，防止磁盘膨胀。
6. **双端轻量运维支持（Web Console & TUI）**：
   - 提供 Snowflake 风格的内嵌 Web 管理控制台，以及专为 SSH 堡垒机环境设计的 2.6MB 独立终端客户端 `registry-tui`。

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
- **独立终端 TUI 客户端 (`registry-tui`)**：
  - 纯 Rust 单静态二进制（2.6MB），在 SSH 堡垒机中直接查看指标、检索构件、生成临时令牌与触发 GC。
- **自动化存储治理与垃圾回收**：
  - 支持多维保留策略（最大版本保留数、快照清理），以及底层孤儿 Blob 的物理垃圾回收。
- **CI/CD 与 Agent 认证集成**：
  - 支持 Personal Access Token（PAT，`tql_pat_*`）与 Webhook 事件通知，支持主流 CLI 工具免配置集成。
- **多后端存储抽象**：
  - 支持 S3 对象存储（兼容 RustFS / MinIO / AWS S3）、本地文件系统与内存存储，底层采用 SHA-256 内容寻址自动去重。

---

## 目录结构

```text
teaql-registry/
├── console/             # Snowflake 风格 React 19 前端控制台源码
├── registry-tui/        # 独立终端 TUI 运维客户端源码 (registry-tui)
├── models/              # TeaQL 领域实体与元数据定义 (model.xml)
├── rust-lib-core/       # 强类型元数据操作与审计层 (teaql-registry-core)
├── rust-web-axum/       # 协议引擎、S3 存储流、REST API 与内嵌 UI (teaql-registry)
├── demo-components/     # 8 大包格式的演示构件与一键发布脚本
└── docker-compose.yml   # 一键拉起 PostgreSQL + RustFS + TeaQL Registry 环境
```

---

## 快速上手

### 1. Docker Compose 一键启动（推荐）

```bash
docker compose up -d
```

服务就绪后，各端点如下：
- **Web 控制台**：`http://localhost:8081/`（默认管理员账号：`admin` / `admin123`）
- **REST API**：`http://localhost:8081/service/rest/v1/...`
- **Prometheus 监控指标**：`http://localhost:8081/metrics`
- **S3 对象存储（RustFS）**：`http://localhost:9010`

### 2. 使用终端 TUI 客户端（适合 SSH / 堡垒机）

```bash
# 编译或直接运行 TUI 客户端
cargo run --release -p registry-tui

# 或指定远程内网后端与访问令牌
registry-tui --endpoint http://10.0.0.10:8081 --token tql_pat_xxx
```

### 3. 发布全格式演示构件

执行自带脚本，一键向运行中的服务推送全部 8 种格式的示例包：
```bash
./demo-components/publish_all_demos.sh
```

### 4. 本地源码编译运行

```bash
# 1. 启动依赖存储与数据库
docker compose up -d postgres rustfs

# 2. 导出环境变量并启动
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

### 5. 运行全量测试套件

```bash
cargo test -- --test-threads=1
```
