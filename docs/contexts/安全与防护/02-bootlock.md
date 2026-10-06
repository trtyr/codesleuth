# bootlock跨进程锁（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
bootlock 跨进程锁是仓库级引导互斥原语：以 `<repo>/.codesleuth/boot.lock` 上的 flock（`std::fs::File::try_lock`）把 graph 引导（init/index --force）与向量索引构建两层 check-then-act 操进程间串行化；acquire 返回 Won/Lost/超时三态（Lost 与超时统一以 CS4016 INDEX_LOCKED 判负退出，不降级），BootLockGuard 在 Drop 时 unlock，引导段结束即释放，不跨 LLM 调用、不罩 serve。

## 证据列表
1. 模块定位：仓库级引导锁 `<repo>/.codesleuth/boot.lock`，用 `std::fs::File::try_lock`（flock，Rust 1.89 稳定）把 graph 引导（init / index --force）与向量构建两层 check-then-act 索引引导在进程间串行化，进程崩溃由内核放锁无陈旧锁。
   - src/bootlock.rs:1-13（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:24-27（审计 #4）
2. 三态结果：acquire 返回 `BootLock::Won(BootLockGuard)` 锁空闲即得、`BootLock::Lost` 等待后拿到（对手刚完成引导，判负退出）、轮询超过 timeout（默认 300s，与 INDEX_TIMEOUT 对齐）→ CS4016 INDEX_LOCKED 结构化错误。
   - src/bootlock.rs:27-33（审计 #2）
   - src/bootlock.rs:50-114（审计 #2）
3. 守卫与释放：`BootLockGuard` 保存 File 句柄，Drop 中调用 `self.file.unlock()` 双保险释放；acquire_guard 封装判负语义——Won 直接拿 guard 干活，Lost → CS4016 INDEX_LOCKED 带 hint「重跑本命令即可」。
   - src/bootlock.rs:38-48（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
4. 调用关系：模块经 `src/lib.rs:7 pub mod bootlock` 挂载；下游三处消费——`src/tools/graph.rs:78` codegraph 引导入口（init/index --force 段，进锁后 `drop(guard)` 放锁再 `McpClient::spawn` serve）、`src/cli.rs:162` 手动 `index --vector` 构建、`src/cli.rs:565` run 时向量构建兜底引导，三处共用同一把 boot.lock。
   - src/lib.rs:1-15（审计 #21）
   - src/tools/graph.rs:75-89（审计 #19）
   - src/cli.rs:161-166（审计 #15）
   - src/cli.rs:563-569（审计 #15）

## 死胡同
- 用 grep 搜 bootlock/BootLock/boot\.lock 全仓只命中 4 个文件，已覆盖 lib.rs/cli.rs/tools/graph.rs/bootlock.rs 全部消费点，无需进一步。
- 未读取 errors.rs 中 INDEX_LOCKED 常量定义（仅在 bootlock.rs:16 use 与 :79/:94/:109 引用处见其码值），不影响本侦察结论。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=27333ms · tokens=46397

