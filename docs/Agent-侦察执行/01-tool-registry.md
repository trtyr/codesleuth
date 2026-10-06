# 工具注册表（Agent 侦察执行）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
工具注册表（ToolRegistry）是 Agent 侦察执行域的只读工具中枢：它把九个只读检索/结构工具以统一 trait 集中注册、按名查找，并向 LLM 输出 JSON Schema 工具面，是「接口上就不存在写能力」的 D009 只读边界防线。入口为 src/tools/mod.rs 的 Tool trait 与 ToolRegistry；src/cli.rs 装配时按层注册 read/fuzzy/graph/vector_search 共九个工具，Harness 运行期经 schemas() 注入工具列表、经 get() 分发执行；下游与 LLM schema、fence 只读围栏、FuzzyEngine、codegraph MCP、RecallEngine 及审计模块交互。

## 证据列表
1. ToolRegistry 定位为只读边界防线，注释明确「接口上就不存在写能力」
   - src/tools/mod.rs:1-2（审计 #2）
2. 统一抽象：Tool trait（name/description/parameters/只读 execute）+ ToolRegistry（Vec<Box<dyn Tool>>）
   - src/tools/mod.rs:14-27（审计 #2）
3. 核心 API：register 注册、get 按名查找、schemas 输出 Vec<ToolSchema> 给 LLM
   - src/tools/mod.rs:34-54（审计 #2）
4. 装配：cli.rs 注册 read/fuzzy 两层 3 个工具 + codegraph 就绪且有符号时注册 5 个图工具 + 向量层注册 vector_search，共九个
   - src/cli.rs:240-244（审计 #18）
   - src/cli.rs:282-288（审计 #18）
   - src/cli.rs:597-599（审计 #18）
5. 诚实工具面：codegraph 无符号时图工具不注册，改为注入地形提示
   - src/cli.rs:273-292（审计 #18）
6. 运行期消费：Harness::new 持有 ToolRegistry，tool_names 用 schemas() 生成工具名列表注入提示
   - src/harness.rs:46-57（审计 #16）
   - src/harness.rs:631-635（审计 #16）
7. 上游依赖：工具实现依赖 Fence 只读围栏、FuzzyEngine、CodegraphEngine（codegraph MCP）
   - src/cli.rs:239（审计 #18）
   - src/cli.rs:242（审计 #18）
   - src/cli.rs:252-253（审计 #18）
8. vector_search 工具依赖 vector::RecallEngine（嵌入+召回存储）
   - src/cli.rs:596-599（审计 #18）

## 死胡同
无

## 置信度
high

## 统计
turns=12 · tool_calls=14 · duration=30948ms · tokens=67065
