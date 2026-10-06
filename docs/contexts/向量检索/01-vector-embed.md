# 向量嵌入（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量嵌入（src/vector/embed.rs）提供 EmbedClient：把代码块文本送进 Qwen3 嵌入服务的 OpenAI 兼容 /embeddings 接口，默认输出 1024 维（Matryoshka 降维，注释中允许实验上调到 4096），batch_size 默认 64、EMBED_CONCURRENCY=4（tokio Semaphore 限并发），查询侧通过 QUERY_INSTRUCT 拼指令前缀（instruct_query），文档侧不拼。响应 index 必须严格 0-based 对齐，错位即拒绝以防静默投毒（validate_index_alignment）。上游由 CLI 在 vector build 与 run 引导处从 cfg.vector 构造；下游被 build_vector_index 写 SQLite、RecallEngine::recall 做三段式召回、VectorSearchTool 对外暴露为 vector_search 工具。

## 证据列表
1. 向量嵌入模块基于 Qwen3 Embedding（OpenAI 兼容 /embeddings），默认 1024 维 Matryoshka 降维，batch_size 默认 64，EMBED_CONCURRENCY=4，查询侧加 Instruct 前缀
   - src/vector/embed.rs:1-7（审计 #2）
   - src/vector/embed.rs:12-16（审计 #2）
   - src/vector/embed.rs:42-49（审计 #2）
   - src/vector/embed.rs:52-64（审计 #2）
2. EmbedClient::embed 流程：去重（按 hash）→ split_batches 切批 → Semaphore(EMBED_CONCURRENCY=4) + JoinSet 并发 → 失败 2s 兜底重试一次 → 按批序号还原顺序回填
   - src/vector/embed.rs:33-39（审计 #2）
   - src/vector/embed.rs:71-146（审计 #2）
3. 单批请求经 embed_one 调用 /embeddings，build_request 固定 model/input/dimensions/encoding_format=float；响应 index 必须严格 0-based 对齐，validate_index_alignment 错位即拒绝以防静默投毒
   - src/vector/embed.rs:22-30（审计 #2）
   - src/vector/embed.rs:148-203（审计 #2）
   - src/vector/embed.rs:206-220（审计 #2）
4. instruct_query 给查询文本拼 QUERY_INSTRUCT 前缀（文档侧不拼），recall 链路上嵌入前调用
   - src/vector/embed.rs:11-20（审计 #2）
   - src/vector/recall.rs:127-139（审计 #4）
5. 上游：CLI 在 vector build（src/cli.rs:153）与 run 引导（src/cli.rs:557）两处构造 EmbedClient，参数从 cfg.vector.embed_model / embed_dims 取，base_url/api_key 缺省跟随 [llm]；embed 文本由 compose_input 按 EmbedMode::Raw / Composite 组装
   - src/cli.rs:140-172（审计 #19）
   - src/cli.rs:545-579（审计 #19）
6. 下游：build_vector_index 调 embed.embed 拿 Vec<Vec<f32>> 落 SQLite；RecallEngine::recall 单条嵌入做 HNSW/暴力召回；VectorSearchTool::execute 通过 engine.recall 对外暴露为 vector_search 工具
   - src/vector/build.rs:24-105（审计 #6）
   - src/vector/recall.rs:24-53（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #28）
   - src/vector/mod.rs:1-21（审计 #23）

## 死胡同
- 用 mode=regex 检索 "vector::embed|use.*embed::" 返回 0 命中（grep 语法不支持此模式）
- grep "recall_by_vector|\\.recall\\(" 返回 0 命中（regex 模式未启用）

## 置信度
high

## 统计
turns=8 · tool_calls=13 · duration=30246ms · tokens=100819

