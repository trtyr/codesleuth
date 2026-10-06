# 模糊搜索工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
对外暴露 `find_files`（frecency 模糊路径搜索）与 `grep`（plain/regex/fuzzy 三态内容检索）两个只读工具，共用 `FuzzyEngine` 持有的 `FilePicker` 会话级索引，由 `src/cli.rs` 装配进 `ToolRegistry` 后供 LLM 通过 tool_calls 调用。

## 证据列表
1. 模块顶部声明 fff-search 封装与 zlob 禁用策略
   - src/tools/fuzzy.rs:1-3（审计 #2）
2. FuzzyEngine 持有 FilePicker，new 阶段用 Fence 取根 + 同步 collect_files
   - src/tools/fuzzy.rs:18-38（审计 #2）
3. FileFinderTool 实现 Tool trait，name="find_files"，执行走 QueryParser + FuzzySearchOptions + picker.fuzzy_search
   - src/tools/fuzzy.rs:41-110（审计 #2）
4. GrepTool 实现 Tool trait，name="grep"，mode 映射 GrepMode::PlainText/Regex/Fuzzy，执行走 picker.grep
   - src/tools/fuzzy.rs:113-208（审计 #2）
5. fff-search crate 提供 FilePicker / QueryParser / FuzzySearchOptions / GrepSearchOptions / GrepMode
   - src/tools/fuzzy.rs:9-12（审计 #2）
6. cli.rs 在层进 v0 阶段构建 Arc<FuzzyEngine> 并把两个工具注册到 ToolRegistry
   - src/cli.rs:244-250（审计 #23）
7. system prompt 把 find_files / grep 列为契约锚点
   - src/prompt.rs:42-43（审计 #36）
8. Tool trait 定义于 tools/mod.rs，子模块 fuzzy 导出两个工具与引擎
   - src/tools/mod.rs:1-22（审计 #8）
9. replay 测试与 fixture 证明 find_files / grep 被 LLM 实际作为工具调用
   - tests/replay.rs:91（审计 #45）
   - tests/fixtures/replay/fixture-rs-replay.json:6-11（审计 #47）
10. 单测覆盖模糊命中、grep 内容命中、缺参报错三条路径
   - src/tools/fuzzy.rs:222-258（审计 #2）

## 死胡同
- grep "FuzzyEngine::new|FileFinderTool::new|GrepTool::new" 复合正则 0 命中（符号名带括号时索引切分失败）
- grep "fff_search" 仅 1 命中，依赖项声明位置已在 fuzzy.rs:9 直接定位，无需扩展
- explore 返回的 vector_search/recall.rs 内容与本任务（路径+内容检索）非同一关注面，未纳入引用

## 置信度
high

## 统计
turns=13 · tool_calls=17 · duration=45899ms · tokens=159561

