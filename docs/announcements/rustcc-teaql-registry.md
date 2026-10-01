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

Registry 很适合检验一个业务框架是否真的能处理复杂场景：它既有多租户和权限边界，又有大量列表查询，还要承受高频写入、版本竞争和跨实体关系读取。在 TeaQL Registry 中，TeaQL（下文简称 TQ）不是只负责把表映射成 Rust struct，而是把查询意图、租户上下文、变更审计、Mutation Policy 治理和数据加载状态一起带进业务代码。本文中的 Mutation Policy Approval 基于 TeaQL Rust Runtime 5.0.5。

这套 TeaQL 开发工具链已经支持 Java、Rust、TypeScript、Go、Swift、.NET 和 Python 七个运行时。同一份语义模型可以生成对应语言的类型化领域库与应用工作区，并通过模型感知 Assist 提供查询、分页、创建、修改、删除和表达式等开发指导。TeaQL Registry 选择 Rust 实现，但下面这些 TQ 编程模型并不只服务于 Rust。

### 1. 在 TQ 执行入口集中实施租户隔离

多租户隔离最怕依赖开发者记忆：如果要求每个 Service 都手写一次 `filter_by_tenant(...)`，迟早会有新接口漏掉。TeaQL Registry 把隔离放在 TQ 的统一执行入口 `RequestPolicy` 中。每一条查询在交给数据服务之前，都会经过 `enforce_select`；策略从可信 context 取得当前租户，并把条件与原查询合并：

```rust
impl RequestPolicy for TeaQLRegistryTenantRequestPolicy {
    fn enforce_select(
        &self,
        context: &UserContext,
        query: &mut SelectQuery,
    ) -> Result<(), RuntimeError> {
        // 集中维护模型中带 tenant 关系的实体集合
        if tenant_scoped_entities.contains(&query.entity.as_str()) {
            let tenant_id = context
                .get_resource::<TenantInfo>()
                .map(|tenant| tenant.tenant_id)
                .unwrap_or(1);
            let tenant_filter = Expr::eq("tenant_id", tenant_id);
            query.filter = Some(match query.filter.take() {
                Some(existing) => existing.and_expr(tenant_filter),
                None => tenant_filter,
            });
        }
        Ok(())
    }
}
```

HTTP 认证层只负责解析 Token、`X-TeaQL-Tenant` 或带租户限定的用户名，确认用户属于目标租户，然后创建请求级 TQ Runtime。`set_tenant(...)` 会同时写入 `TenantInfo` 并安装上述策略。后续业务查询只接收这个 context，不再传递 `tenant_id`，也不需要重复拼租户条件：

```rust
let repositories = RepositoryService::list(&tenant_context).await?;
let users = SecurityService::list_users(&tenant_context).await?;
```

项目的多租户集成测试会创建两个 context，然后用完全相同、没有租户参数的调用分别查询仓库、BlobStore 和用户，验证结果只来自各自租户。隔离因此是“默认发生”的基础设施能力，而不是散落在业务代码里的编码约定。

PAT 也绑定 `tenant_id + user_id + username`；即使原生包管理器把 Token 放在 Basic Auth 密码位置，服务端仍会重新校验这三者与当前租户是否一致。

### 2. 分页查询仍然保留类型和意图

管理端的服务日志查询是一个直接例子。过滤条件、稳定排序和分页都通过模型生成的 `Q` API 表达：

```rust
let rows = Q::service_logs_minimal()
    .select_self_fields()
    .order_by_id_desc()
    .offset(offset as u64, limit as u64)
    .comment("what: query registry service logs with bounded pagination")
    .purpose("why: render best-effort operational package history")
    .execute_for_list(context)
    .await?;
```

这段查询故意没有 `filter_by_tenant(...)`：租户条件会在 TQ 执行入口由 `RequestPolicy` 自动注入。`order_by_id_desc()` 给翻页提供确定顺序，`offset/limit` 限制单次物化规模；字段选择、关系选择和业务过滤方法则来自模型生成代码，字段改名或关系不存在会在编译阶段暴露。对于需要同时返回分页元数据的列表，TQ 还提供 `execute_for_page(context, offset, limit)`，返回 `SmartList<T>`，并由 Runtime 统一执行硬上限和查询策略。

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

完整加载很重要：目标实体首先经过集中租户策略查询，TQ 的 checker 可以看到完整业务状态，实体原始版本也能参与乐观锁判断。`audit_as(...)` 让每次持久化都携带明确的业务原因；保存继续使用同一个请求 context，使查询、修改和审计处在同一条请求链路中。

### 4. 在事务之前完成 Mutation Policy Approval

`audit_as(...)` 解决的是“这次修改为什么发生”，Mutation Policy Approval 解决的是另一个问题：“允许这类修改的规则本身，是否经过了明确评审？”这不是订单、工单一类业务单据的人工审批流，而是对应用自定义写策略的版本化批准。

TeaQL 5.0.5 会在 Checker/Fix 和完整 Graph Planning 之后生成不可变的 `MutationPlan`。其中包含根实体、审计原因、Create/Update/Delete/Recover 操作、原始版本以及实际变更字段。Runtime 在开启数据库事务、执行第一条写入之前，把整份计划交给 `MutationPolicy::review`：

```text
实体变更
   │
   ▼
Checker / Fix ──► 不可变 MutationPlan
                         │
                         ▼
                MutationPolicy::review
                    │              │
                  Deny           Allow
                    │              │
          事务开始前返回错误     精确查找 Policy Approval
                                   │
                                   ▼
                           事务写入 + 审计事件
```

应用策略可以对整张变更图做判断，而不必把规则散落在每个 Service 方法中。例如，Registry 可以要求每次写入都带审计原因，并限制一次保存携带的操作数量：

```rust
use teaql_runtime::{
    MutationDecision, MutationPlan, MutationPolicy, MutationPolicyIdentity, UserContext,
};

struct RegistryMutationPolicy;

impl MutationPolicy for RegistryMutationPolicy {
    fn identity(&self) -> MutationPolicyIdentity {
        MutationPolicyIdentity::new(
            "teaql-registry-write-policy",
            "1.0.0",
            "sha256:<reviewed-policy-fingerprint>",
        )
    }

    fn review(&self, _context: &UserContext, plan: &MutationPlan) -> MutationDecision {
        let missing_reason = plan
            .audit_reason
            .as_deref()
            .map(str::trim)
            .map(str::is_empty)
            .unwrap_or(true);
        if missing_reason {
            return MutationDecision::deny(
                "REGISTRY-AUDIT-REASON-REQUIRED",
                "registry mutations require an audit reason",
                ["audit_reason"],
            );
        }
        if plan.operations.len() > 128 {
            return MutationDecision::deny(
                "REGISTRY-MUTATION-TOO-LARGE",
                "one registry mutation may contain at most 128 operations",
                ["operations"],
            );
        }
        MutationDecision::allow()
    }
}
```

Policy 通过后，Runtime 再向可信的 `MutationPolicyApprovalProvider` 查询批准记录。批准必须同时匹配 Policy 的 `id + version + fingerprint`；只改版本号或代码指纹而沿用旧批准，都会被识别为未批准。Policy Registry、Approval Provider 和告警 Sink 都在服务端组装 `UserContext` 时安装，HTTP 或 TFP 请求不能从外部替换它们。

这里还有一个值得明确的兼容边界：没有安装客户 Policy 时，Runtime 允许原有写入并产生 `MUTATION-POLICY-001`；安装了客户 Policy 但没有精确批准时，会产生 `MUTATION-POLICY-002`。这两个状态目前是治理告警，不会冒充强制审批。真正的 Policy `Deny` 则会在事务开始前终止保存，保证没有部分数据落库。允许执行时，Policy 身份、Approval 状态、告警码和变更字段摘要会随 `MutationGovernanceSnapshot` 进入审计事件，让“哪一版规则批准了哪类变更”成为可追踪证据。

这对 AI Agent 尤其有意义：Agent 仍然可以高频修改对象，但能够放行写入的规则是有身份、有版本、有指纹的；策略代码变化后必须重新批准，不能静默继承上一版信任。

### 5. 用 E 表达式区分“空值”和“根本没加载”

Rust 的 `Option` 能表达“有值或为空”，但数据库投影还有第三种状态：字段或关联根本没有被查询。若把后两者都当成 `None`，程序可能把一个漏写 `select` 的编码错误误判成正常业务空值，并继续产生错误结果。

TQ 根据模型生成 `E` 表达式，保留三种状态：

- `Value(value)`：字段已加载且有值；
- `Null`：字段已加载，业务上确实为空；
- `NotLoaded`：查询没有预加载该字段或关联，是程序错误。

例如读取 Asset 的 Blob 关联和路径时，查询端必须先明确选择要遍历的关联，然后再用 `E` 读取：

```rust
let assets = Q::assets_minimal()
    .select_path()
    .select_asset_blob_with(
        Q::asset_blobs_minimal()
            .select_blob_ref()
            .select_sha256_checksum(),
    )
    .with_id_is(asset_id)
    .limit(1)
    .comment("what: load one asset and its blob metadata")
    .purpose("why: verify package content before download")
    .execute_for_list(context)
    .await?;

let asset = assets.into_iter().next().expect("asset exists");
let blob = E::asset(&asset).get_asset_blob().eval();
let path = E::asset(&asset).get_path().unwrap();
```

`eval()` 会把已加载的值或空值转换成 `Option`，但遇到 `NotLoaded` 会带着缺失访问路径立即失败；`or_if_null(...)` 也只为真正的数据库 `NULL` 提供默认值，不会掩盖漏加载。

当前实现已经把这套规则落到了主要读取链路，而不再只是示例。Asset、AssetBlob 和 Component 查询使用明确投影；下载、搜索、清理以及各制品生态的读取路径会一次性选择 `AssetBlob` 关系，再在 Service 边界通过 `E` 构造只读模型。这样既消除了逐个 `asset_blob_id` 回查造成的 N+1，也让漏掉关系投影时立即以 `NotLoaded` 暴露。集成测试同时覆盖了“标量已投影可正常读取”“关系未投影必须快速失败”“关系投影后可读取”，以及稳定、互不重叠的生成式分页。写入仍然使用生成的 `update_*`、`audit_as(...)` 和 context-only save；`E` 只负责安全读取，不混入 mutation 路径。

为了让这条规则不只依赖开发者或 AI 的自觉，`cargo teaql` 还提供了按语言拆分的本地检查命令，覆盖 Rust、Java、Kotlin、Python、C#、Go、Swift 和 TypeScript。检查器从 KSML/XML 模型识别实体关系，并验证应用代码是否绕过生成的 `E` facade；找不到关系元数据或可用的 `E` 时会失败关闭，而不会给出“假通过”。例如 TypeScript 检查同时解析 `.ts` 和 `.tsx`：

```bash
cargo teaql typescript-expression-check --source src --format json
```

`item.platform?.name` 这样的直接关联遍历会报告 `TQL-TYPESCRIPT-EXPR-001`，而 `E.workItem(item).platform().name().eval()` 可以通过。确有历史兼容需求时，可以在 `.teaql/typescript-expression-check.yml` 中记录精确到文件、函数、规则号且带原因的审计例外，避免用宽泛忽略把检查整体关掉。

## 5 秒启动

镜像和 `.env` 已准备好时，启动只有一条命令：

```bash
docker compose up -d
```

服务端和 TUI 都是 Rust 编译出的独立二进制，没有 JVM 或脚本运行时的预热过程；在本地或集群节点热启动时，应用进程通常可以在数秒内进入工作状态。这里的“5 秒”指应用自身的启动体验，首次拉取镜像以及等待 PostgreSQL、S3 就绪的时间仍取决于网络和部署环境。

首次部署只需先准备仓库和环境变量：

```bash
git clone https://github.com/teaql/teaql-registry.git
cd teaql-registry
cp .env.example .env
# 在 .env 中设置 POSTGRES_PASSWORD 和 S3_SECRET_KEY
# 然后执行上面的 docker compose up -d
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

仓库已经有 workspace 测试、协议生命周期测试和覆盖多个生态的原生客户端验证。不同生态的边角兼容性仍然很多，尤其是代理/聚合仓库行为、签名元数据以及各版本客户端差异，这些正是我们接下来希望继续打磨的部分。

如果你正在做 AI Agent、内部构建平台、离线开发环境，或者只是对“用 Rust 实现多协议 Registry”感兴趣，欢迎试用、提 Issue 或 PR。我们尤其希望听到下面几类反馈：

1. 你最常用、但目前还缺失的开发制品格式；
2. Proxy / Group 仓库在真实团队里的使用方式；
3. 跨平台预编译工具链的命名、索引和分发体验；
4. 哪些协议值得优先补齐更严格的官方兼容测试。

项目地址：<https://github.com/teaql/teaql-registry>

许可证：Apache-2.0
