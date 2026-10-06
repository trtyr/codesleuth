# callers 调用方查询（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
callers 是 codesleuth 只读检索工具面中的调用方查询工具：给定符号名，返回「谁调用了它」（codegraph 结构图反向边），帮助 Agent 快速定位符号的上游引用与影响入口。定义于 src/tools/graph.rs:169-173（宏 cg_symbol_tool! 生成 CallersTool，参数 {symbol, limit?}），执行时先经 symbol_args 校验参数（graph.rs:113-128），再由 CodegraphEngine::call 附加 projectPath 并转发给外部 codegraph MCP 服务（graph.rs:104-110，实际查询由 codegraph 完成）。注册在 cli.rs:284-287，仅当 codegraph 存在且符号索引非空时才注册，否则输出地形提示（cli.rs:290-293）。上游依赖：CodegraphEngine 生命周期（start 中 codegraph init/serve，graph.rs:71-102）与 McpClient；下游：与同族 callees/impact/explore 共用同一宏与结图层，作为 Tool 注册进 registry 供 Agent 调用。

# callers 调用方查询（检索工具面 · 只读）

## 1. 是干什么的
给定符号名，返回「谁调用了它」——结构图反向边，供 Agent 快速定位符号的上游引用（src/tools/graph.rs:172 工具描述）。

## 2. 入口与关键文件
- src/tools/graph.rs:169-173 —— CallersTool 定义（宏生成，args: {symbol, limit?}）
- src/tools/graph.rs:159-164 —— execute：校验参数并转发查询
- src/tools/graph.rs:104-110 —— CodegraphEngine::call：附 projectPath 调 MCP
- src/cli.rs:284-287 —— 工具注册（索引非空才注册）

## 3. 怎么运作
CallersTool::execute → symbol_args 校验 symbol → engine.call("callers", args) → McpClient::call_tool("codegraph_callers")，实际反查由外部 codegraph 服务完成，结果格式化为 `[codegraph callers] {symbol}` 返回（graph.rs:162-163）。

## 4. 交互
上游依赖 CodegraphEngine（负责 codegraph init/serve 生命周期，graph.rs:71-102）与 McpClient；与同族 callees/impact/explore 共用宏与结图层（graph.rs:174-183），注册进 cli.rs 的 Tool registry 供 Agent 会话使用。

## 证据列表
1. callers 工具由宏 cg_symbol_tool! 定义为 CallersTool，描述为「谁调用了这个符号（结构图反向边）」，参数 {symbol, limit?}
   - src/tools/graph.rs:169-173（审计 #2）
2. 执行链：execute 经 symbol_args 校验 symbol/limit 后调用 engine.call(tool, a) 转发给 codegraph
   - src/tools/graph.rs:159-164（审计 #2）
   - src/tools/graph.rs:113-128（审计 #2）
3. CodegraphEngine::call 自动附 projectPath，调用 McpClient::call_tool("codegraph_callers")，实际查询由外部 codegraph MCP 服务完成
   - src/tools/graph.rs:104-110（审计 #2）
   - src/tools/graph.rs:89-90（审计 #2）
4. 注册：仅当 codegraph 存在且符号索引非空时注册 CallersTool 等图工具，否则输出地形提示
   - src/cli.rs:282-293（审计 #10）
5. 同族 callees/impact/explore 共用同一宏与结图层；CodegraphEngine 负责 codegraph init/serve 生命周期
   - src/tools/graph.rs:174-183（审计 #2）
   - src/tools/graph.rs:71-102（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=16241ms · tokens=21709
