# vector_search 语义召回（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
vector_search 是仓库里的语义向量检索工具：接受自然语言问题（无需包含代码符号名），返回语义最相似的代码位置指针，适合概念型查询与跨词汇表述（src/tools/vector_search.rs:30-34）。入口为 VectorSearchTool（实现通用 Tool trait，注册名 "vector_search"），核心逻辑在 RecallEngine；由 CLI 装配层 setup_vector_layer 在向量层初始化时注册（src/cli.rs:597）。运作链路：execute → RecallEngine::recall（查询经指令前缀嵌入成向量）→ recall_by_vector 三段式召回（小库精确暴力 / 大库 HNSW 超采 + 余弦精算 + 墓碑过滤取 top-K）。上游依赖嵌入服务 EmbedClient、向量库 VectorStore 与 hnsw_rs 图索引；下游供 Agent 工具注册表（ToolRegistry）消费，返回结果提示使用者必须 read 原文验证。

## 证据列表
1. 工具描述与参数：输入自然语言 query，可选 k（默认 8 上限 10），返回语义最相似代码位置指针，证据需 read 原文
   - src/tools/vector_search.rs:30-46（审计 #2）
2. 入口结构 VectorSearchTool 实现通用 Tool trait，名字为 vector_search；execute 校验参数后调用 engine.recall 并格式化输出，空结果时引导改用 grep/find_files
   - src/tools/vector_search.rs:10-21（审计 #2）
   - src/tools/vector_search.rs:48-63（审计 #2）
3. 召回引擎 RecallEngine：SQLite 真相 + 会话内 HNSW 影子图 + 三段式查询；recall 将查询经 instruct_query 前缀嵌入，recall_by_vector 在无 HNSW（≤10 万块）时全量暴力余弦，有 HNSW 时 k×3 超采后精算并墓碑过滤
   - src/vector/recall.rs:1-4（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/vector/recall.rs:99-125（审计 #4）
4. 装配入口：CLI 的 setup_vector_layer 补建向量索引、打开 VectorStore，并把 VectorSearchTool 注册进 ToolRegistry；构建失败弹性降级不致命
   - src/cli.rs:532-540（审计 #10）
   - src/cli.rs:589-597（审计 #10）
5. 上下游：依赖 vector::embed::EmbedClient、vector::store::{VectorStore, cosine}、hnsw_rs；作为 Box<dyn Tool> 注册进 ToolRegistry 供 Agent 调用
   - src/vector/recall.rs:6-10（审计 #4）
   - src/vector/recall.rs:26-29（审计 #4）
   - src/tools/vector_search.rs:109-114（审计 #2）

## 死胡同
- grep "VectorSearchTool::new|setup_vector_layer" 组合词零命中（grep 对多词 plain 模式不支持），拆成单符号后命中

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=18381ms · tokens=31170

