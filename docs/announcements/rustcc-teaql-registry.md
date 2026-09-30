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

## TeaQL 在这个项目里做了什么？

Registry 很适合检验一个业务框架是否真的能处理复杂场景：它既有多租户和权限边界，又有大量列表查询，还要承受高频写入、版本竞争和跨实体关系读取。在 TeaQL Registry 中，TeaQL（下文简称 TQ）不是只负责把表映射成 Rust struct，而是把查询意图、租户上下文、变更审计和数据加载状态一起带进业务代码。

### 1. 把租户隔离绑定到可信请求上下文

每个请求会先解析 Token、`X-TeaQL-Tenant` 或带租户限定的用户名，确认用户确实属于目标租户，然后基于共享连接池创建一个请求级 TQ Runtime：

```rust
let mut runtime = service_runtime_from_pool(pool.clone()).await?;
runtime.init_registry_context(self.blobstore.clone());
runtime.set_tenant(tenant_id, tenant_name);
```

后续仓库、组件、用户和日志的查询或写入都只接收这个请求级 context。租户、身份、权限、请求策略和数据服务由可信 context 注入，而不是让 HTTP JSON 参数临时决定。这样协议处理器不需要在每条查询里重复拼接一段容易漏掉的租户 SQL，也不会因为某个新接口忘记过滤条件而意外跨租户读取。

PAT 也绑定 `tenant_id + user_id + username`；即使原生包管理器把 Token 放在 Basic Auth 密码位置，服务端仍会重新校验这三者与当前租户是否一致。

### 2. 分页查询仍然保留类型和意图

管理端的服务日志查询是一个直接例子。过滤条件、稳定排序和分页都通过模型生成的 `Q` API 表达：

```rust
let rows = Q::service_logs_minimal()
    .select_self_fields()
    .filter_by_tenant(tenant_id)
    .order_by_id_desc()
    .offset(offset as u64, limit as u64)
    .comment("what: query registry service logs with bounded pagination")
    .purpose("why: render best-effort operational package history")
    .execute_for_list(context)
    .await?;
```

`order_by_id_desc()` 给翻页提供确定顺序，`offset/limit` 限制单次物化规模；字段选择、关系选择和过滤方法则来自模型生成代码，字段改名或关系不存在会在编译阶段暴露。对于需要同时返回分页元数据的列表，TQ 还提供 `execute_for_page(context, offset, limit)`，返回 `SmartList<T>`，并由 Runtime 统一执行硬上限和查询策略。

这里的 `comment` 与 `purpose` 也不是装饰：它们把“查询了什么”和“为什么查询”带到执行阶段，缺失查询意图可以被策略拒绝，便于审计和诊断慢查询。

### 3. 修改数据走完整实体、校验和审计链

项目不会把外部 DTO 直接翻译成 `UPDATE`。修改组件前先通过同一个租户 context 加载完整实体，再调用模型生成的 `update_xxx` 方法，最后附加业务原因并保存。例如删除组件采用软删除：

```rust
let rows = Q::components()
    .select_self_fields()
    .with_id_is(component_id)
    .limit(1)
    .comment("what: query registry component metadata")
    .purpose("why: resolve package components and their assets")
    .execute_for_list(context)
    .await?;

if let Some(mut component) = rows.into_iter().next() {
    component.update_kind("deleted");
    component.update_name(format!("[DELETED_{}]", component.id()));
    component
        .audit_as("Deleting component")
        .save_with(context)
        .await?;
}
```

完整加载很重要：TQ 的 checker 可以看到完整业务状态，实体原始版本也能参与乐观锁判断。`audit_as(...)` 让每次持久化都携带明确的业务原因；保存仍然只接受原请求 context，因此修改无法绕开同一套租户、权限与审计边界。

### 4. 用 E 表达式区分“空值”和“根本没加载”

Rust 的 `Option` 能表达“有值或为空”，但数据库投影还有第三种状态：字段或关联根本没有被查询。若把后两者都当成 `None`，程序可能把一个漏写 `select` 的编码错误误判成正常业务空值，并继续产生错误结果。

TQ 根据模型生成 `E` 表达式，保留三种状态：

- `Value(value)`：字段已加载且有值；
- `Null`：字段已加载，业务上确实为空；
- `NotLoaded`：查询没有预加载该字段或关联，是程序错误。

例如读取 Asset 的 Blob 关联和路径时，可以写成：

```rust
let blob = E::asset(&asset).get_asset_blob().eval();
let path = E::asset(&asset)
    .get_path()
    .or_if_null("unknown-path".to_owned());
```

`eval()` 会把已加载的值或空值转换成 `Option`，但遇到 `NotLoaded` 会带着缺失访问路径立即失败；`or_if_null(...)` 也只为真正的数据库 `NULL` 提供默认值，不会掩盖漏加载。当前服务中直接读取已完整加载的标量仍然很方便，而涉及最小投影和跨关系遍历时，E 表达式可以让“数据为空”和“程序写错了查询”保持清晰边界。这种尽早失败比在很远的业务分支里产生错误制品元数据更容易定位，也更适合 Registry 这类基础设施服务。

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
