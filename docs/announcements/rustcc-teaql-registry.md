# 开源｜TeaQL Registry：用 Rust 做一个面向 AI Agent 与 CI/CD 的多格式制品仓库

> 建议投稿栏目：大家的项目

最近我们开源了 [TeaQL Registry](https://github.com/teaql/teaql-registry)。它是一个使用 Rust、Axum、Tokio、TeaQL 和 PostgreSQL 构建的多格式制品仓库，目标不是替代 Nexus 或 JFrog 这样的企业主仓库，而是部署在 AI 编码沙箱、构建节点或集群内部，承担高频、短生命周期的中间制品交换与缓存。

## 为什么又做一个 Registry？

AI Agent 和高吞吐 CI 的工作方式与人工发版不太一样：一次任务可能反复经历“改代码—打包—部署—测试—再修改”，产生很多只存活几分钟或几小时的 snapshot、crate、wheel、容器层和工具链压缩包。把这些中间产物全部推到中心仓库，会引入网络延迟、限流，也会让长期仓库承担不必要的清理压力。

TeaQL Registry 更像是靠近 Runner 的 L1 Registry：

```text
AI Agent / CI Runner
        │ 高频 publish / pull
        ▼
TeaQL Registry（本地或集群内）
        │ 只提升最终验证通过的版本
        ▼
Nexus / JFrog / 云端主仓库
```

## 目前支持什么？

项目目前支持 14 种制品格式：Docker Registry v2、Maven2、npm、PyPI、Cargo、Go Modules、NuGet、Swift Package Registry、Dart Pub、RubyGems、Composer、Conan 2、Hex，以及 Raw/Generic。

这里的重点是“软件开发过程中的组件”，而不是面向终端用户的软件商店。比如跨平台预编译工具链，我们建议直接用 Raw/Generic 保存按 OS、架构和版本组织的 `.tar.gz` / `.zip`；如果交付物本身是容器镜像，就使用 Docker/OCI；如果它属于某个语言生态，再选择对应的原生协议。

服务端支持 S3 兼容存储（RustFS、MinIO、AWS S3）、文件系统和内存模式，Blob 使用 SHA-256 内容寻址去重。管理侧提供多租户、RBAC、PAT、审计记录、保留策略和孤儿 Blob GC，并带有嵌入式 Web Console 与独立的 Ratatui TUI。

![TeaQL Registry TUI](https://raw.githubusercontent.com/teaql/teaql-registry/main/docs/images/teaql-registry-tui.png)

TUI 适合只有 SSH 的跳板机或构建节点，可以查询服务状态、仓库和组件，执行搜索、检查、清理、GC 与临时令牌生成。

## Rust 在这里解决了什么？

Registry 的数据面有大量流式上传下载、校验、协议路由和并发请求。Rust 让我们可以在一个进程里完成协议适配、元数据服务和 Blob 流式传输，同时把资源生命周期和错误路径尽量前移到编译期处理。Axum/Tokio 负责 HTTP 与异步 I/O，TeaQL 负责领域模型、权限和可审计的数据变更，PostgreSQL 保存元数据；Web Console 被嵌入最终二进制，容器运行时镜像可以保持很小。

另一个实际收益是部署形态：服务端和 `registry-tui` 都可以构建成独立二进制，适合放进构建集群或受限网络环境。

## 五分钟启动

```bash
git clone https://github.com/teaql/teaql-registry.git
cd teaql-registry
cp .env.example .env
# 在 .env 中设置 POSTGRES_PASSWORD 和 S3_SECRET_KEY
docker compose up -d
```

启动后可访问：

- Web Console：`http://localhost:8081/`
- REST API：`http://localhost:8081/service/rest/v1/...`
- Prometheus Metrics：`http://localhost:8081/metrics`
- 快速帮助：`http://localhost:8081/help`

TUI 可以这样运行：

```bash
cargo run -p registry-tui -- \
  --endpoint http://localhost:8081 \
  --username admin
```

管理员初始密码默认随机生成并写入凭据目录，也可以在首次启动前通过 `ADMIN_PASSWORD` 显式设置；项目没有内置默认密码。

## 当前状态与我们希望得到的反馈

仓库已经有 workspace 测试、协议生命周期测试和原生客户端验证；Cargo 与 Conan 2 已通过真实客户端发布/消费流程。不同生态的边角兼容性仍然很多，尤其是代理/聚合仓库行为、签名元数据以及各版本客户端差异，这些正是我们接下来希望继续打磨的部分。

如果你正在做 AI Agent、内部构建平台、离线开发环境，或者只是对“用 Rust 实现多协议 Registry”感兴趣，欢迎试用、提 Issue 或 PR。我们尤其希望听到下面几类反馈：

1. 你最常用、但目前还缺失的开发制品格式；
2. Proxy / Group 仓库在真实团队里的使用方式；
3. 跨平台预编译工具链的命名、索引和分发体验；
4. 哪些协议值得优先补齐更严格的官方兼容测试。

项目地址：<https://github.com/teaql/teaql-registry>

许可证：Apache-2.0
