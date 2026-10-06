# 结构图工具集（结构图与MCP）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
结构图工具集通过本地 codegraph CLI 的 MCP 接口（`codegraph serve --mcp`）向上层 agent 暴露五个 Tool：explore / callers / callees / impact / files。CodegraphEngine 在 cli.rs 启动时完成 init（或 force index）→ spawn MCP 子进程 → initialize 握手，并独占进程（`CODEGRAPH_NO_DAEMON=1`，Drop 回收）；五个 Tool 的 execute 把参数归一化后统一走 CodegraphEngine::call（自动附 projectPath）→ McpClient::call_tool，回填 `[codegraph <tool>] <label]\n<out>`。索引为空或非代码仓时按 QA FINDING-009 诚实降级，不注册图工具并给 agent 地形提示。

## 证据列表
1. 模块定位：结图层以 MCP 方式接入 codegraph，生命周期归 codesleuth，每会话独占进程。
   - src/tools/graph.rs:1-5（审计 #2）
2. CodegraphEngine::start 入口负责索引引导（init/force，可执行名由配置链传入）后 spawn `codegraph serve --mcp` 并完成 initialize 握手。
   - src/tools/graph.rs:69-102（审计 #2）
3. CodegraphEngine::call 统一为 MCP 调用，自动追加 projectPath 并把工具名加 `codegraph_` 前缀。
   - src/tools/graph.rs:104-110（审计 #2）
4. cg_symbol_tool! 宏生成 CallersTool / CalleesTool / ImpactTool 三个反向边/正向边/影响面工具，共享参数 schema（symbol 必填，limit/depth 可选）。
   - src/tools/graph.rs:130-183（审计 #2）
5. ExploreTool 接受自然语言 query（结构图一把梭），FilesTool 接受可选 glob filter，分别独立实现 Tool trait。
   - src/tools/graph.rs:185-265（审计 #2）
6. 上游注册点：cli.rs 在索引非空时把 5 个工具装入 registry；非代码仓/索引空时按 QA FINDING-009 不注册并给出地形提示，引导 agent 用 find_files/grep/read。
   - src/cli.rs:255-301（审计 #15）
7. 下游依赖：McpClient::spawn 起子进程与 JSON-RPC 收发，call_tool 执行 tools/call；Drop 回收 server 进程。
   - src/mcp.rs:128-133（审计 #4）
8. 真机验收：tests/graph_live.rs 默认 ignore，演示 init→spawn→5 工具真实调用→Drop 全链路。
   - tests/graph_live.rs:1-48（审计 #4）

## 死胡同
- grep 用复合字面量模式首次 0 命中（被字面 OR 与字符类吞掉），改用正则重试后在 src/cli.rs 命中 5 行注册点。
- src/vector/chunk.rs 出现 codegraph sqlite 适配器（symbols_from_codegraph），与结构图工具集不直接相关，未纳入 findings。

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=30019ms · tokens=58176

