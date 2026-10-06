# 向量召回引擎（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量召回引擎（src/vector/recall.rs 中的 RecallEngine）为上层工具与 CLI 任务编排提供「自然语言/向量 → top-K 余弦相似度代码位置」能力；采用「≤10 万向量走暴力、否则 HNSW 三段式（k×3 超采 → 余弦精确复算 → 墓碑过滤取 top-K）」策略，由 VectorSearchTool 与 cli::setup_vector_layer 共同消费。

## 证据列表
1. RecallEngine 结构定义在 src/vector/recall.rs:25-30，持 rows（真相快照）、alive（活跃集合）与可选 HNSW 影子图。
   - src/vector/recall.rs:25-30（审计 #2）
2. 三段式策略与常量定义明确：HNSW_EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2。
   - src/vector/recall.rs:13-22（审计 #2）
3. RecallEngine::open 是构造入口，模型/维度不符时跳过 HNSW、降级为暴力；满足 MIN_HNSW_SIZE 才调用 build_hnsw。
   - src/vector/recall.rs:34-66（审计 #2）
4. recall_by_vector 实现 HNSW 三段式（k×OVERFETCH_FACTOR 超采 → cosine 精确复算 → alive 墓碑过滤 → 截断 top-K）与无 HNSW 时的暴力回退两套分支。
   - src/vector/recall.rs:100-125（审计 #2）
5. recall 异步入口负责 query 嵌入（instruct_query 指令前缀）并把 row 索引映射回 Chunk。
   - src/vector/recall.rs:128-139（审计 #2）
6. 墓碑与压缩：tombstone 维护 alive 集合；compact 在 needs_compact 为真时丢死行、重编号、必要时重建 HNSW。
   - src/vector/recall.rs:69-97（审计 #2）
7. format_recall_block 注入固定提示头「[语义召回 · 起步线索]」、带「可能无关」与「必须 read 原文」免责。
   - src/vector/recall.rs:141-159（审计 #2）
8. VectorSearchTool 持有 Arc<RecallEngine>，在 execute 中调用 engine.recall 并用 RecallEngine::format_recall_block 渲染。
   - src/tools/vector_search.rs:10-22（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #4）
9. CLI 侧 setup_vector_layer 用 VectorStore 构造 Arc<RecallEngine>，注入 VectorSearchTool，并直接消费 recall + format_recall_block 当任务提示种子。
   - src/cli.rs:602-621（审计 #17）

## 死胡同
- grep 用 plain 模式对 RecallEngine::open 检索返回 0 命中，改用 regex 模式后正常返回（已规避）。
- callers 查询 RecallEngine::tombstone 仅返回 2 个测试调用方（recall.rs:219、245），未发现生产调用方；据此判断 compact 路径在当前仓库内为预留入口。
- 未读取 src/vector/chunk.rs / src/vector/embed.rs / src/vector/store.rs 内部实现细节，仅依赖 recall.rs 中的引用与 import 关系，外部模块的细节作为隐含依赖未深入。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=37969ms · tokens=54582

