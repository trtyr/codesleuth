# MCP codegraph 客户端（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
MCP codegraph 客户端是 codesleuth 只读检索工具面中的结图层数据源：它 spawn 外部 codegraph 服务（MCP over stdio、newline-delimited JSON-RPC），为 agent 的 explore/callers/callees/impact/files 五个结构检索工具提供符号调用图查询能力，会话结束由 Drop 回收子进程。

## 证据列表
1. 价值：通过 spawn 外部 codegraph MCP 服务（stdio JSON-RPC）获得结构图查询能力，供 explore/callers/callees/impact/files 五个图工具使用
   - src/mcp.rs:1-4（审计 #2）
   - src/tools/graph.rs:1-5（审计 #4）
2. 入口与关键文件：src/mcp.rs 定义 McpClient（spawn/initialize/call_tool/Drop 回收）；src/tools/graph.rs 定义 CodegraphEngine 及图工具（ExploreTool/CallersTool 等）；src/cli.rs 负责启动与注册
   - src/mcp.rs:18-25,68,104-114,128-133,198-205（审计 #2）
   - src/tools/graph.rs:18-22,71-102,104-110,169-183,186-188（审计 #4）
3. 运作链：cli.rs 调 CodegraphEngine::start（索引缺失跑 codegraph init、force 则重建，经 bootlock 串行化，然后 spawn `codegraph serve --mcp` 并 initialize 握手）→ agent 调用图工具 execute → engine.call（自动附 projectPath，工具名加 codegraph_ 前缀）→ McpClient.call_tool 发 JSON-RPC
   - src/cli.rs:252-257（审计 #10）
   - src/tools/graph.rs:75-101,105-110（审计 #4）
   - src/mcp.rs:128-133（审计 #2）
4. 模块交互：上游依赖 bootlock 引导锁（D014）与外部 codegraph CLI 二进制（cfg.graph.bin 传入）；下游注册进 ToolRegistry 供 agent 检索（无符号索引时不注册并提示降级用 find_files/grep/read；启动失败非致命降级并审计留痕）
   - src/cli.rs:273-296（审计 #10）
   - src/tools/graph.rs:78-88（审计 #4）

## 死胡同
- grep "CodegraphEngine::start"（plain，含双冒号）零命中，改用 "CodegraphEngine" 命中 src/cli.rs

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=17816ms · tokens=51917
