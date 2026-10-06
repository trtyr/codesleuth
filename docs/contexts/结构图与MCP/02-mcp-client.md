# MCP客户端（结构图与MCP）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
MCP 客户端是 codesleuth 访问 codegraph 结构图后端的 stdio 通道：spawn `codegraph serve --mcp` 子进程后，以 newline-delimited JSON-RPC 完成 initialize 握手并对外暴露 `initialize / list_tools / call_tool` 三个方法，由 `CodegraphEngine` 独占持有、`Drop` 时回收 server 进程。

## 证据列表
1. McpClient 持子进程与 stdin/reader，封装 child + stdin + reader + next_id + timeout 五个字段。
   - src/mcp.rs:19-25（审计 #2）
2. `McpClient::spawn` 用 tokio Command 拉起 server 并接管 stdin/stdout，stderr 重定向到 null。
   - src/mcp.rs:68-102（审计 #2）
3. `initialize` 发 `initialize` 请求并随后 `notifications/initialized` 通知完成握手。
   - src/mcp.rs:104-114（审计 #2）
4. `call_tool` 走 `tools/call` 方法，结果经 `extract_tool_text` 拼接 text 内容，空内容 / isError 转为 CsError。
   - src/mcp.rs:128-133（审计 #2）
   - src/mcp.rs:38-53（审计 #2）
5. 底层 `request` 自增 `next_id` 后写一行 JSON-RPC（`build_request` 构造），在 reader 上循环 read_line 直到出现匹配 id 的响应，期间非法行与服务端通知一律跳过；带 90s 超时。
   - src/mcp.rs:152-190（审计 #2）
   - src/mcp.rs:16-35（审计 #2）
6. `impl Drop for McpClient` 在析构时 `try_lock` child 并 `start_kill`，完成 server 进程回收。
   - src/mcp.rs:198-204（审计 #2）
7. 唯一消费侧 `CodegraphEngine` 在 `start_with_bin` 中负责先 `codegraph init / index --force` 引导索引，再 `McpClient::spawn(bin, ["serve","--mcp"], root)` 并调 `client.initialize()`；会话内 `call` 透传到 `client.call_tool`，自动附 `projectPath`。
   - src/tools/graph.rs:19-22（审计 #4）
   - src/tools/graph.rs:75-102（审计 #4）
   - src/tools/graph.rs:104-110（审计 #4）
8. 错误统一归类为 `INDEX_NOT_AVAILABLE / INDEX_BUILD_FAILED / INDEX_TIMEOUT`（段位 CS4xxx），exit code 由段位决定。
   - src/errors.rs:46-49（审计 #4）
   - src/errors.rs:9-23（审计 #4）

## 死胡同
- 未深入 `src/tools/graph.rs` 中 `cg_symbol_tool!` 宏族展开细节（与本功能点关系为下游消费，链路已通过 `engine.call → client.call_tool` 覆盖）
- 未读 `src/vector/embed.rs` 中对 `McpClient::spawn` 的另一调用点（explore 提示存在 1 个调用，但与"MCP 客户端"主路径无关，未深入）

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=25455ms · tokens=41855

