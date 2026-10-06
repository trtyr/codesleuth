# vector_search工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
vector_search 是 codesleuth 工具面里的一条只读工具，向调用方提供「自然语言问题 → 语义最相似代码位置指针」能力；CLI 在 setup_vector_layer 中将 VectorSearchTool 注册进 ToolRegistry，execute 校验 query/k 后调用 RecallEngine.recall 并经 format_recall_block 渲染输出；同一召回通道在 CLI 启动期被直接消费以注入系统提示。

## 证据列表
1. vector_search 工具向调用方提供自然语言到语义相似代码位置指针的检索能力，输入 query（必填）与 k（可选，默认 8，上限 10），返回的是位置指针，使用前必须 read 校验。
   - src/tools/vector_search.rs:26-46（审计 #2）
   - src/tools/vector_search.rs:30-35（审计 #2）
2. VectorSearchTool 结构体持有 Arc<RecallEngine> 与 default_k，通过 new(engine) 构造并实现 Tool trait。
   - src/tools/vector_search.rs:10-22（审计 #2）
   - src/tools/vector_search.rs:24-46（审计 #2）
3. execute 缺 query 时返 USER_INPUT 错误（带示例 hint），k 缺省为 8 并被 clamp 到 [1,10]；空结果返降级文案，否则由 RecallEngine::format_recall_block 渲染命中行块。
   - src/tools/vector_search.rs:48-63（审计 #2）
4. CLI 启动期在 setup_vector_layer 中构造 RecallEngine 并通过 registry.register(VectorSearchTool::new(Arc::clone(&recall))) 把本工具挂入 ToolRegistry。
   - src/cli.rs:601-605（审计 #12）
   - src/cli.rs:602-605（审计 #12）
5. RecallEngine 提供 pub async fn recall(&self, query: &str, k: usize) -> CsResult<Vec<(Chunk, f32)>>；模块按「HNSW k×3 超采（ef_search=64）→ 候选精确重算余弦 → 墓碑过滤取 top-K」三段式实现，<1000 向量走暴力（暖启动零负担）。
   - src/vector/recall.rs:1-15（审计 #16）
   - src/vector/recall.rs:128（审计 #16）
6. ToolRegistry 通过 register/get/schemas 暴露工具，调用方按 name 字符串（如 "vector_search"）检索并通过 Tool::execute 触发。
   - src/tools/mod.rs:24-58（审计 #14）
7. 同一 RecallEngine 召回通道在 CLI 启动期被直接消费：hits 经 format_recall_block 注入到系统提示，失败则 best-effort 写审计并降级；命中可作为 repomap 种子。
   - src/cli.rs:606-621（审计 #12）

## 死胡同
- 未深入读取 RecallEngine::recall 内部三段式实现细节，仅读签名 src/vector/recall.rs:128 与模块头注释 1-4 行（推断为「HNSW 超采→余弦重算→墓碑过滤」）
- 未读 Tool trait 顶部定义 src/tools/mod.rs:1-19 行（已通过 grep 锁定 register/get/schemas 行为，足够支撑本简略文档）
- 未读 src/cli.rs:665 之后的 141 行以及 setup_vector_layer 完整前文（已有 540-665 范围证据覆盖注册/召回注入关键路径）

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33272ms · tokens=41076

