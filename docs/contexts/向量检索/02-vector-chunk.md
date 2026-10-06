# 向量切块（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量切块（chunk）位于 src/vector/chunk.rs，以 codegraph 符号表为边界真源，按「主块 / 超大滑窗 / 兜底」三层规则把仓库源码切成可嵌入的 Chunk；主入口 plan_chunks（chunk.rs:258）由 build_vector_index（build.rs:25）调用，产物经 compose_input、VectorStore、RecallEngine、CLI 报告消费。

## 证据列表
1. 模块定位为「codegraph 符号表为边界真源 + 三层规则」：层1 符号主块、层2 超大滑窗、层3 兜底。
   - src/vector/chunk.rs:1-5（审计 #4）
2. 关键常量：MAX_CHUNK_CHARS=6000、SUB_WINDOW_LINES=100、SUB_OVERLAP_LINES=15、FALLBACK_WINDOW_LINES=100、FALLBACK_OVERLAP_LINES=20；KEPT_KINDS=function/method/struct/class/interface/impl/trait。
   - src/vector/chunk.rs:14-47（审计 #4）
3. codegraph 适配器：symbols_from_codegraph 只读打开 SQLite nodes 表返回 SymbolRow 列表；relations_for_symbol 提供 callers/callees（去重封顶 8）。
   - src/vector/chunk.rs:84-134（审计 #4）
   - src/vector/chunk.rs:137-166（审计 #4）
4. 子窗口切分在超大符号体内按 SUB_WINDOW_LINES 切、每窗重拼面包屑 breadcrumb。
   - src/vector/chunk.rs:177-179（审计 #4）
   - src/vector/chunk.rs:227-244（审计 #4）
5. 主入口 plan_chunks 编排三层：按文件分组的 KEPT_KINDS 过滤 + 文档行上提 → 主块或 sub_window_chunks 滑窗 → covered gaps 产 leftover → 磁盘兜底产 fallback；最后按 (file,line_start,symbol) 排序。
   - src/vector/chunk.rs:258-474（审计 #4）
   - src/vector/chunk.rs:280-340（审计 #4）
   - src/vector/chunk.rs:342-380（审计 #4）
   - src/vector/chunk.rs:407-468（审计 #4）
6. build_vector_index 调 symbols_from_codegraph → plan_chunks → 用 text_hash 增量复用 → 嵌入 → commit_build 原子落库。
   - src/vector/build.rs:8-11（审计 #11）
   - src/vector/build.rs:25-69（审计 #11）
7. 模块对外契约：pub use 暴露 Chunk、SymbolRow、plan_chunks。
   - src/vector/mod.rs:1-21（审计 #9）
   - src/vector/mod.rs:17（审计 #9）
8. compose_input 消费 Chunk：Raw 模式返回 chunk.text，Composite 模式 = header + 标识符层（去重封顶 20）。
   - src/vector/compose.rs:6（审计 #39）
   - src/vector/compose.rs:42-51（审计 #39）
9. VectorStore 用 chunk_key(file + symbol + line_start) 作为 chunk 唯一键、存 SQLite chunks 表 + 唯一索引。
   - src/vector/store.rs:7（审计 #41）
   - src/vector/store.rs:14-16（审计 #41）
10. RecallEngine 持 rows: Vec<(Chunk, Vec<f32>)>；recall 按文本查询返回 (Chunk, f32) 命中；format_recall_block 把 hits 渲染为「起步线索」格式。
   - src/vector/recall.rs:7（审计 #43）
   - src/vector/recall.rs:128-139（审计 #43）
   - src/vector/recall.rs:142-159（审计 #43）
11. CLI 在两条路径上触发 build_vector_index 并打印 chunk/嵌入/复用/GC 报告；run_task 阶段还通过 relations_for_symbol 取每个 hit 的 callers/callees 入图扩展种子。
   - src/cli.rs:167-180（审计 #45）
   - src/cli.rs:571-589（审计 #45）
   - src/cli.rs:630-643（审计 #45）

## 死胡同
- grep 关键字 "Chunk\b|chunk_key" / "plan_chunks|chunk::" 在词边界模式下 0 命中（grep 默认 plain 行为），改用宽松词 "chunk" 后获得完整调用链。
- 未读 chunk.rs:474 之后的测试实现细节（仅用于验证行为，不影响主流程理解）。
- 未读 src/vector/embed.rs（嵌入客户端，与切块逻辑正交）。
- 首次提交因引用了未读过的 compose/store/recall/cli 行被拒，read 后重提。

## 置信度
high

## 统计
turns=11 · tool_calls=19 · duration=50011ms · tokens=166943

