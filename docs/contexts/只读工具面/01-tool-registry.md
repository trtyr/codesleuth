# 工具注册表（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
工具注册表（ToolRegistry）是只读工具面的统一寻址与模式导出器，物理上无写能力（接口根上就不存在写方法）。它以 `Vec<Box<dyn Tool>>` 持有 `Tool` 实现，提供 `register` 注入、`get(name)` 寻址、`schemas()` 导出 `ToolSchema` 列表；由 `cli` 在启动时装配 `ReadTool`/`FileFinderTool`/`GrepTool`，`Harness` 持有 `tools: ToolRegistry` 字段并通过 `tool_names`+`schemas` 枚举、按 `name` 取出 `Tool` 后 `execute`（只读执行）。

## 证据列表
1. `ToolRegistry` 是 `Vec<Box<dyn Tool>>` 的薄包装结构体，本身只持有工具列表，无任何写能力方法。
   - src/tools/mod.rs:24-27（审计 #2）
2. `Tool` trait 定义了只读工具的契约：`name` / `description` / `parameters` / 异步只读 `execute(args) -> CsResult<String>`，接口层就排除了写能力。
   - src/tools/mod.rs:14-22（审计 #2）
3. `register` 入口把 `Box<dyn Tool>` 推入内部 `Vec`，无去重、无校验、无写盘/落库副作用。
   - src/tools/mod.rs:34-36（审计 #2）
4. `get(name)` 按 `t.name() == name` 线性查找并以 `&dyn Tool` 返回，供调用方触发 `execute`。
   - src/tools/mod.rs:38-43（审计 #2）
5. `schemas()` 将内部工具映射成 `ToolSchema { name, description, parameters }` 列表，用于把工具清单暴露给上层（Harness/LLM）。
   - src/tools/mod.rs:45-54（审计 #2）
6. 模块注释明确标注「D009 只读边界的根本防线：接口上就不存在写能力」，证明只读定位是设计意图而非偶然。
   - src/tools/mod.rs:1（审计 #2）
7. 单元测试 `registry_roundtrip` 覆盖 `new`/`register`/`is_empty`/`get`/`schemas` 全流程，是行为规范的最小可信源。
   - src/tools/mod.rs:83-94（审计 #2）
8. 向量搜索工具自带 `registers_into_registry` 集成测试，验证「能成功注册并被按名查到」。
   - src/tools/vector_search.rs:108-114（审计 #4）
9. CLI 启动时 `ToolRegistry::new()` 后依次 `register` 注入 `ReadTool` / `FileFinderTool` / `GrepTool`，是注册表的上游装配点。
   - src/cli.rs:246-250（审计 #14）
10. `Harness` 持有 `tools: ToolRegistry` 字段，把注册表作为运行时按名分发的来源。
   - src/harness.rs:37（审计 #16）
   - src/harness.rs:49（审计 #16）
11. `Harness::run_with` 显式把 `ToolRegistry` 作为入参传入，并经 `Harness::new` 灌入字段，验证注册表是 Harness 的核心依赖。
   - src/harness.rs:803-818（审计 #16）
12. `tool_names(&ToolRegistry)` 通过 `reg.schemas()` 收集工具名清单，是 Harness 枚举工具的标准用法。
   - src/harness.rs:668-672（审计 #16）
13. 对抗测试 `build_registry` 自行按 `ToolRegistry::new` + `register(ReadTool/FileFinderTool/GrepTool)` 模式构造夹具驱动 Harness。
   - tests/adversarial.rs:31-39（审计 #25）
14. 回放测试 `replay.rs` 沿用同一装配模式：`ToolRegistry::new()` + 三个 `register`。
   - tests/replay.rs:65-69（审计 #27）
15. 分层联测 `layered_live.rs` 同样以 `ToolRegistry::new()` + 三个 `register` 模式构造夹具。
   - tests/layered_live.rs:29-33（审计 #29）

## 死胡同
- 未展开实现细节（题目红线 A），未读取 `fuzzy.rs` / `graph.rs` / `read.rs` 中 `Tool` 实现内部；只验证了注册表抽象层与装配/消费面。
- grep "register(" 在 82 文件中 0 命中，改用 grep "ToolRegistry" 7 文件命中完成定位，未影响结论。

## 置信度
high

## 统计
turns=7 · tool_calls=11 · duration=41269ms · tokens=49610

