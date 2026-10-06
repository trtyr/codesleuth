# 向量索引构建流水线（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
向量索引构建流水线把代码仓库切块、嵌入并向量化后落进项目本地的 SQLite 单文件索引（.codesleuth/indexes/<fingerprint>/vectors.db），为语义检索（RecallEngine/HNSW 召回）提供下游数据源，并通过 text_hash 增量复用与失效块 GC 保持索引与代码同步。

## 证据列表
1. 功能价值：把仓库代码切块→组装→嵌入→SQLite 落地，供向量召回使用；text_hash 未变的 chunk 整体复用、向量零开销。
   - src/vector/build.rs:1-5（审计 #2）
   - src/vector/build.rs:24-30（审计 #2）
2. 入口与关键文件：src/vector/build.rs:25 build_vector_index 编排整条流水线；src/vector/store.rs:306 project_index_dir 指定索引存于 <repo>/.codesleuth/，store.rs:310 index_path 具体为 indexes/<fingerprint>/vectors.db；src/vector/mod.rs 是模块汇总。
   - src/vector/build.rs:25-105（审计 #2）
   - src/vector/store.rs:303-312（审计 #9）
   - src/vector/mod.rs:1-14（审计 #4）
3. 运作方式：CLI 两个入口（src/cli.rs:163 手动预建、src/cli.rs:565 run 时引导补建，均在 bootlock 引导锁内）调用 build_vector_index；后者从 codegraph.db 取符号→plan_chunks 切块→比对 existing_hashes 做 text_hash 复用→compose_input 组装→embed.embed 网络调用→store.commit_build 在单事务内完成 GC 删除+upsert+meta 写入（store.rs 原子落库）。
   - src/cli.rs:163-168（审计 #17）
   - src/cli.rs:565-583（审计 #17）
   - src/vector/build.rs:31-92（审计 #2）
4. 交互关系：上游依赖 codegraph 符号库（build.rs:34-35 读 .codegraph/codegraph.db）与嵌入客户端 EmbedClient（模型/维度/模式变更则旧向量全部作废，build.rs:40-47）；下游消费方是 RecallEngine（recall.rs:34 open 从索引库构建 HNSW 影子召回），cli.rs:585 构建完成后立即打开该索引供检索用。
   - src/vector/build.rs:34-35（审计 #2）
   - src/vector/recall.rs:33-36（审计 #24）
   - src/cli.rs:584-589（审计 #17）
   - src/vector/build.rs:40-47（审计 #2）
   - src/vector/build.rs:49-57（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=39901ms · tokens=31835

