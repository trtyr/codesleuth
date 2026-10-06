# 路径围栏（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
路径围栏（Fence）是只读边界安全层：保证所有工具目标路径解析后落在目标仓库根目录内，词汇越界（..、绝对路径）与 symlink 逃逸一律拒绝并返回 FENCE_DENIED 结构化错误，为调用方提供「读不越出仓库」的安全保证。入口 src/fence.rs:30（resolve），Fence 定义于 src/fence.rs:7，由 src/cli.rs:239 在启动时以仓库根创建并注入工具注册表。运作链：cli.rs 创建 Fence → ReadTool 持有 Arc<Fence> → read.rs:86 每次读文件前调 fence.resolve → resolve 先词汇预检（normalize 折叠 .. 后 starts_with_root 检查，fence.rs:37/59/75），再 dunce::canonicalize 解 symlink 后复检（fence.rs:43-48），两道都不过即 FENCE_DENIED。上游依赖 errors 模块的 CsError/FENCE_DENIED/REPO_NOT_FOUND 与 dunce::canonicalize；下游消费 ReadTool；FuzzyEngine 仅用 Fence::new 取规范化根目录定界 picker（fuzzy.rs:24-27），不消费 resolve；tests/adversarial.rs 覆盖 symlink/绝对/相对越界攻击面。

## 证据列表
1. 路径围栏确保所有工具目标路径解析后落在仓库根内，symlink 逃逸拒绝，越界返回 FENCE_DENIED 结构化错误
   - src/fence.rs:1-2（审计 #2）
   - src/fence.rs:27-55（审计 #2）
2. 入口 resolve 于 fence.rs:30，Fence 在 cli.rs:239 创建并注入 ReadTool
   - src/fence.rs:7-30（审计 #2）
   - src/cli.rs:239-241（审计 #7）
3. 调用链：ReadTool 调 fence.resolve；resolve 先词汇预检（normalize+starts_with_root）再 canonicalize 解 symlink 复检
   - src/tools/read.rs:86（审计 #18）
   - src/fence.rs:37-48（审计 #2）
   - src/fence.rs:58-88（审计 #2）
4. 交互：依赖 errors 模块错误码与 dunce::canonicalize；FuzzyEngine 仅用 Fence 取规范化根目录；tests/adversarial.rs 覆盖 symlink/绝对/相对越界
   - src/fence.rs:4（审计 #2）
   - src/tools/fuzzy.rs:24-27（审计 #23）
   - tests/adversarial.rs:152-158（审计 #25）

## 死胡同
无

## 置信度
high

## 统计
turns=8 · tool_calls=9 · duration=26288ms · tokens=46455

