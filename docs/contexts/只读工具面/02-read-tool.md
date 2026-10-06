# read工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
`read` 是仓库内的只读强读工具，作为侦察链路唯一的事实来源：按 offset/limit 分页读取文件，输出「行号:行哈希锚点|内容」格式，并对二进制、非 UTF-8、空文件、offset 越界等情形给出诚实声明。已阅读 src/tools/read.rs（228 行全文）、src/tools/mod.rs、src/cli.rs:240-254 及工具调用图，可确认它在 ToolRegistry 中作为 `read` 名称注册的只读工具之一，由 CLI 启动时注入。

## 证据列表
1. `ReadTool` 是只读工具面 `Tool` trait 的实现，结构体定义在 src/tools/read.rs:15-23，通过 `new(fence: Arc<Fence>)` 注入沙箱。
   - src/tools/read.rs:15-23（审计 #2）
2. `Tool` trait 定义在 src/tools/mod.rs:14-22，要求实现 `name`/`description`/`parameters`/`execute`；`ReadTool` 注册名为 `"read"`（src/tools/read.rs:46-48），并声明参数 `{path, offset?, limit?}`（src/tools/read.rs:54-64）。
   - src/tools/mod.rs:14-43（审计 #7）
   - src/tools/read.rs:46-64（审计 #2）
3. `execute` 实现核心六特性：行号+12-bit 行哈希锚点（src/tools/read.rs:135）、offset/limit 分页与续读 offset（src/tools/read.rs:120, 138-143）、二进制探测拒倾倒（src/tools/read.rs:96-103, 32-42）、lossy UTF-8 编码消毒并声明 U+FFFD（src/tools/read.rs:105-107, 121-128）、空文件与 offset 越界的诚实反馈（src/tools/read.rs:110-118）。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:25-42（审计 #2）
4. 路径访问受 `Fence` 沙箱约束：execute 通过 `self.fence.resolve(&path_arg)`（src/tools/read.rs:86）将相对路径锁定在仓库根内，越界或非普通文件返回 `CsError(REPO_NOT_READABLE)`（src/tools/read.rs:87-92）。`DEFAULT_LIMIT=200`、`MAX_LIMIT=2000`（src/tools/read.rs:12-13）。
   - src/tools/read.rs:12-13（审计 #2）
   - src/tools/read.rs:86-94（审计 #2）
5. `ReadTool` 在 CLI 启动时由 src/cli.rs:247 通过 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))` 注入 `ToolRegistry`，与 `FileFinderTool`、`GrepTool` 并列（src/cli.rs:248-250），作为只读工具面的首个注册项。
   - src/cli.rs:244-251（审计 #7）
6. `ReadTool` 同样在三处集成测试中被注册使用：tests/adversarial.rs:34、tests/replay.rs:66、tests/layered_live.rs:30，且 src/tools/read.rs:149-228 自身有 5 个单元测试覆盖锚点/分页/二进制/lossy/空文件/越界/缺参等路径。
   - src/tools/read.rs:149-228（审计 #2）
7. 依赖项：`CsError`/`CsResult`/`REPO_NOT_READABLE`/`USER_INPUT`（src/tools/read.rs:4, 71, 88-94, 220-226）、`Fence`（src/tools/read.rs:5, 86）、`Tool` trait（src/tools/read.rs:6, 45-146）、`async_trait`（src/tools/read.rs:7, 44, 66）、`serde_json::Value`（src/tools/read.rs:8, 54-64, 66-84）、`sha2::Sha256`（src/tools/read.rs:9, 27-28）。
   - src/tools/read.rs:1-30（审计 #2）
   - src/tools/read.rs:66-94（审计 #2）
   - src/tools/read.rs:220-226（审计 #2）

## 死胡同
- grep 检索 `tools::read::ReadTool` / `use.*tools::read` 命中 0 文件——src/cli.rs 使用的是 `tools::read::ReadTool::new(...)` 限定路径全称而非 use 别名（src/cli.rs:247）。
- grep 复合 `register.*read` 模式命中 0 文件，因 src/cli.rs:247 是多 token 构造 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))`，单条 plain 模式被空格拆分。改用 `ReadTool::new` 检索后正常命中。
- explore 报告 `ReadTool` 仅 1 个 caller in `src/tools/read.rs`、⚠️ no covering tests——这是 codegraph 仅扫到单文件直接 new 调用的盲点，实际在 src/cli.rs:247 与三处集成测试（tests/adversarial.rs:34, tests/replay.rs:66, tests/layered_live.rs:30）均有 register 链路。

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=34437ms · tokens=50845

