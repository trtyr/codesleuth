# callees 被调查询（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
callees 是 codesleuth 只读检索工具面中的结图层查询工具：给定符号名，返回该符号调用了谁（codegraph 结构图正向边），帮助 Agent 理解代码流向。入口由宏 cg_symbol_tool! 生成 CalleesTool，execute 校验 symbol 后转发 CodegraphEngine::call，再经 McpClient 以 MCP tools/call 调外部 codegraph server 的 codegraph_callees 工具；仅当 codegraph 索引存在且符号非空时才注册。

## 证据列表
1. 功能：callees 查询「这个符号调用了谁」（结构图正向边），参数 {symbol}，是五图工具（explore/callers/callees/impact/files）之一
   - src/tools/graph.rs:174-178（审计 #2）
2. 入口：CalleesTool 由宏 cg_symbol_tool! 生成，实现统一 Tool trait（name/description/parameters/execute）
   - src/tools/graph.rs:130-167（审计 #2）
3. 校验：execute 经 symbol_args 要求 symbol 必填（非空），透传可选 limit/depth
   - src/tools/graph.rs:113-128（审计 #2）
4. 调用链：CalleesTool.execute → CodegraphEngine.call("callees") 自动附加 projectPath → McpClient.call_tool("codegraph_callees")（MCP over stdio JSON-RPC）调外部 codegraph server
   - src/tools/graph.rs:104-110（审计 #2）
   - docs/检索工具面-只读/09-mcp-codegraph.md:6（审计 #16）
5. 注册与门控：cli.rs 仅在 codegraph 存在且 .codegraph/codegraph.db 符号索引非空时注册 CalleesTool 等五工具，否则注入地形提示改用 find_files/grep/read
   - src/cli.rs:275-294（审计 #2）
6. 交互：上游依赖 src/mcp.rs 的 McpClient 与 CodegraphEngine（init/serve 生命周期）；与同族 callers/impact/explore 共用同一宏与结图层；vector 召回层（chunk.rs relations_for_symbol）也查询 callees 关系作为关系上下文
   - src/vector/chunk.rs:83-88（审计 #14）
   - docs/检索工具面-只读/09-mcp-codegraph.md:12（审计 #16）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=34438ms · tokens=31722
