# 向量存储（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量存储是向量检索域的唯一持久真相层：vectors.db（SQLite 单文件）持久化 chunk 及其嵌入向量，HNSW 图仅作为会话内影子索引。提供 schema 初始化、增量 upsert、text_hash 复用、GC 清理、原子构建事务、全量加载、cosine 工具与指纹路径等能力，由 build_vector_index 编排构建，由 RecallEngine 消费真相加载影子。

## 证据列表
1. vectors.db 是嵌入索引唯一持久真相，HNSW 仅会话内影子
   - src/vector/store.rs:1-3（审计 #2）
   - src/vector/recall.rs:1-4（审计 #13）
2. VectorStore 持 rusqlite Connection 并在 open 时建表（chunks/meta/descriptions）
   - src/vector/store.rs:41-43（审计 #2）
   - src/vector/store.rs:47-79（审计 #2）
3. commit_build 在单一事务内完成 GC + upsert + meta 写入，原子落库
   - src/vector/store.rs:198-232（审计 #2）
   - src/vector/build.rs:85-95（审计 #6）
4. build_vector_index 编排：plan_chunks → existing_hashes 增量复用 → compose_input + embed.embed → commit_build 落库
   - src/vector/build.rs:24-105（审计 #6）
5. RecallEngine.open 从 store.load_all 拉真相，仅当 ≥ MIN_HNSW_SIZE 时构建 HNSW 影子，否则走暴力
   - src/vector/recall.rs:34-66（审计 #13）
   - src/vector/recall.rs:19-20（审计 #13）
6. 索引路径 index_path(state_dir, fp)/vectors.db 与项目本地目录 .codesleuth/
   - src/vector/store.rs:303-312（审计 #2）
   - src/cli.rs:590-605（审计 #26）
7. VectorSearchTool 持有 Arc<RecallEngine>，execute 调 recall(query,k) 并用 format_recall_block 输出
   - src/tools/vector_search.rs:10-63（审计 #8）
8. 上游依赖 codegraph 符号表、EmbedClient、compose_input、bootlock；下游消费 VectorSearchTool 工具与 run_task 召回先行注入
   - src/vector/build.rs:7-14（审计 #6）
   - src/vector/build.rs:35-36（审计 #6）
   - src/cli.rs:162-172（审计 #26）
   - src/cli.rs:565-608（审计 #26）
   - src/cli.rs:602-605（审计 #26）
9. vector 模块公共导出 VectorStore / RecallEngine / build_vector_index 等
   - src/vector/mod.rs:1-21（审计 #4）

## 死胡同
- grep 全文搜 build_vector_index/VectorStore/RecallEngine/VectorSearchTool 未给出更多新调用点（仅返回 82 文件已读过的命中）
- 未深入 vector/chunk.rs、vector/embed.rs、vector/compose.rs 的实现细节（任务域限定为「向量存储」）

## 置信度
high

## 统计
turns=8 · tool_calls=14 · duration=29732ms · tokens=123453

