# impact 影响面分析（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
impact 影响面分析是只读检索工具面中的一个图分析工具：给定符号名（可选 depth 1-5），通过 codegraph 结构图计算改该符号的传递闭包波及面（blast radius）。入口由宏 cg_symbol_tool! 定义为 ImpactTool，注册于 cli.rs，执行时经 CodegraphEngine 转发到外部 codegraph MCP server 的 codegraph_impact 工具。

## 证据列表
1. impact 工具定位：cg_symbol_tool! 宏定义 ImpactTool，名为 "impact"，描述为「改这个符号的影响面（传递闭包 blast radius）。args: {symbol, depth?}」
   - src/tools/graph.rs:179-183（审计 #7）
2. 调用链：ImpactTool::execute 校验 symbol 参数后调用 CodegraphEngine::call("impact", args)，后者自动附加 projectPath 并调用 McpClient 的 codegraph_impact 工具
   - src/tools/graph.rs:104-110（审计 #7）
   - src/tools/graph.rs:159-163（审计 #7）
3. 入口注册：cli.rs 仅在 .codegraph/codegraph.db 存在且符号索引非空时注册 ImpactTool（与 explore/callers/callees/files 一同）；否则不注册并输出地形提示
   - src/cli.rs:273-288（审计 #14）
4. 上游依赖：CodegraphEngine 负责生命周期——索引缺失时执行 codegraph init（300s 超时，走引导锁），随后 spawn `codegraph serve --mcp` 并完成 initialize 握手，会话内五工具复用
   - src/tools/graph.rs:1-5（审计 #7）
   - src/tools/graph.rs:85-90（审计 #7）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=7 · duration=20832ms · tokens=23203
