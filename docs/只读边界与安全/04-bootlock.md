# bootlock 引导锁（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
bootlock（D014 索引引导锁）是一把仓库级跨进程文件锁 `.codesleuth/boot.lock`，用于把并发进程的索引引导（codegraph init / index --force 与向量索引构建）串行化，防止两路 check-then-act 竞争撞库；竞争败者以 CS4016 INDEX_LOCKED 判负退出而不降级（重跑秒过），从而保证调用方拿到完整工具面。

## 证据列表
1. bootlock 用 flock 锁文件把两层「check-then-act」索引引导在进程间串行化，背景是 2026-10-05 双进程并发 init 事故（图与向量层共用同一问题）
   - src/bootlock.rs:1-14（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:7-14（审计 #4）
2. 入口在 src/bootlock.rs：acquire 实现 try_lock 首试 + 500ms 轮询等待（Won/Lost 三态，超时 300s），acquire_guard 把 Lost/超时统一收残为 INDEX_LOCKED 结构化错误
   - src/bootlock.rs:50-114（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
3. 三个调用点：graph.rs start_with_bin 的 init/index --force 引导、cli.rs 的 index --vector 手动构建、cli.rs run 时的向量构建，均通过 `?` 传播判负退出
   - src/tools/graph.rs:75-84（审计 #13）
   - src/cli.rs:157-166（审计 #15）
4. 上游依赖：errors.rs 的 CS4016 INDEX_LOCKED / REPO_NOT_READABLE 错误码；实现依赖 std::fs::File::try_lock（Rust 1.89，无新外部依赖），Guard Drop 时自动 unlock
   - src/bootlock.rs:16（审计 #2）
   - src/bootlock.rs:12-13（审计 #2）
   - src/bootlock.rs:35-48（审计 #2）
5. 下游消费：向量构建（build_vector_index，chunk 集合是 codegraph 图符号表的投影）在锁内执行；锁只罩 init/build，不跨 LLM 调用、不罩 serve
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:14（审计 #4）
   - src/bootlock.rs:116-119（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16377ms · tokens=28372

