# MCP 进程生命周期回收（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
MCP 进程生命周期回收（D011）让 codesleuth 以一次性会话方式启动外部 codegraph MCP 子进程使用其图查询工具，并在会话结束时由 Rust 的 Drop 语义自动 kill 子进程，保证无残留进程。入口是 `McpClient::spawn`（src/mcp.rs:68）与 `Drop for McpClient`（src/mcp.rs:198-205）；调用链为 `cli.rs` → `CodegraphEngine::start`（src/tools/graph.rs:71）→ `McpClient::spawn` + `initialize` 握手，会话期间通过 `CodegraphEngine::call` → `McpClient::call_tool` 做 JSON-RPC 请求，engine 被丢弃时 Drop 触发 `child.start_kill()` 回收。上游依赖 `crate::errors`、`crate::bootlock`（引导锁）与 tokio 进程/IO；下游消费方是 `src/tools/graph.rs` 的 CodegraphEngine 及其暴露给上层的 graph 工具，另有真机验收测试 `tests/graph_live.rs:12`。

## 证据列表
1. 该功能是最小 MCP over stdio 客户端：spawn server → newline-delimited JSON-RPC（initialize/tools_list/tools_call）→ Drop 回收，模块头即声明此定位
   - src/mcp.rs:1-4（审计 #2）
2. 入口与回收点：McpClient::spawn 以 piped stdin/stdout 启动子进程；Drop 实现调用 child.start_kill() 保证会话结束即回收 server 进程
   - src/mcp.rs:68-102（审计 #2）
   - src/mcp.rs:198-205（审计 #2）
3. 调用链：CodegraphEngine::start 经 bootlock 引导锁后 spawn McpClient 并握手；call 方法封装 tools/call 并自动附 projectPath
   - src/tools/graph.rs:71-102（审计 #19）
   - src/tools/graph.rs:105-110（审计 #19）
4. 上游调用方为 CLI 入口，从配置链传入可执行名（cfg.graph.bin）创建 CodegraphEngine 并置于 Arc 共享
   - src/cli.rs:249-257（审计 #24）
5. 真机验收测试覆盖完整生命周期：start → 五工具调用 → drop 回收（对应验收 D011）
   - tests/graph_live.rs:1-5（审计 #4）
   - tests/graph_live.rs:47-48（审计 #4）
6. 交互模块：依赖 crate::errors 错误体系（INDEX_NOT_AVAILABLE 等）与 crate::bootlock 仓库级引导锁（索引 init 串行化）
   - src/mcp.rs:6（审计 #2）
   - src/tools/graph.rs:78-88（审计 #19）

## 死胡同
无

## 置信度
high

## 统计
turns=9 · tool_calls=8 · duration=27903ms · tokens=61491

