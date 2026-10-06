# Fence路径围栏（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
仓库的路径围栏（Fence）位于 src/fence.rs，是读类工具访问仓库前统一的硬边界：任何传入路径必须最终解析到 repo root 之内，越界（绝对路径出库、`..` 逃逸、symlink 指向库外）一律返回 CS3003（FENCE_DENIED）。其采用「词汇层预检 + dunce::canonicalize 解 symlink 复检」两道关卡，并在 CLI 启动时由 src/cli.rs:245 装配为 Arc<Fence> 注入 ReadTool 与 FuzzyEngine。

## 证据列表
1. Fence 是仓库只读边界的第二层：所有工具的目标路径必须解析到 repo root 之内，symlink 逃逸一律拒绝，越界返 CS3003 结构化报错。
   - src/fence.rs:1-2（审计 #2）
2. Fence::new 接收仓库根并用 dunce::canonicalize 规范化，失败时返回 REPO_NOT_FOUND（CS3001）。
   - src/fence.rs:12-21（审计 #2）
   - src/errors.rs:42-44（审计 #19）
3. resolve 实现「词汇层预检 + canonicalize 后复检」两道防御：先对 candidate_abs 做 starts_with_root 判定，再用 dunce::canonicalize 解析 symlink 后再判一次，两次越界都返 FENCE_DENIED；不存在的路径返 REPO_NOT_FOUND。
   - src/fence.rs:27-55（审计 #2）
4. 词汇层归一化 normalize 仅折叠 `.` 与 `..`，不解 symlink；围栏前缀判定 starts_with_root 在 Windows 下按 ASCII 大小写不敏感比较，其余平台严格比较。
   - src/fence.rs:58-88（审计 #2）
5. FENCE_DENIED 错误码为 CS3003，对应退出码 4。
   - src/errors.rs:44（审计 #19）
   - src/fence.rs:114-115（审计 #2）
6. CLI 启动期由 cli.rs 装配 Arc<Fence> 并注入 ReadTool 与 FuzzyEngine；同时 FuzzyEngine::new 内部自建一份 Fence 仅用于取规范化根作为模糊检索 base_path。
   - src/cli.rs:245-250（审计 #21）
   - src/tools/fuzzy.rs:22-37（审计 #25）
   - src/tools/read.rs:15-22（审计 #23）
7. ReadTool 在 call 阶段先调 self.fence.resolve(&path_arg) 守门，通过后才落盘读取；越界/逃逸由 Fence 抛 FENCE_DENIED。
   - src/tools/read.rs:86-93（审计 #23）
8. 单测覆盖：内部存在路径放行、symlink 逃逸拒绝、绝对路径越界拒绝、不存在路径返 REPO_NOT_FOUND。
   - src/fence.rs:94-132（审计 #2）
9. 对抗性集成测试 assertion_2_fence_rejects_escapes 验证 innocent.txt（symlink 指向库外 .ssh_keys）、/etc/passwd、../../etc/passwd 三种敌对路径均被 Fence 以 FENCE_DENIED 拒绝。
   - tests/adversarial.rs:141-161（审计 #4）

## 死胡同
- 用 `\.resolve\(` 仅匹配到 src/fence.rs 与 tests/adversarial.rs 的测试调用与 src/tools/read.rs:86 一处生产调用，src/tools/fuzzy.rs 内未发现对 fence.resolve 的调用（仅借 fence.root() 取规范化根）。
- 未在仓库中找到其他对 Fence::new 的生产调用点之外的非测试构造方（grep 命中的 9 处中 5 处为 tests/* 与 src/fence.rs 内 #[cfg(test)]）。

## 置信度
high

## 统计
turns=7 · tool_calls=13 · duration=33460ms · tokens=49780

