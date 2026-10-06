# explore 结构图探索（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
explore 是 codesleuth 只读检索工具面中的结构图探索工具：一次调用返回与 query（符号名/文件名/自然语言）相关的符号源码、调用路径与影响面，专为「开局看图」设计。实现上它是 codegraph 外部 MCP 服务的薄封装：ExploreTool::execute 校验 query 后经 CodegraphEngine.call 转发为 MCP 工具 codegraph_explore，引擎在启动时通过 codegraph CLI init/serve 建立连接。CLI 启动时仅在 codegraph 存在且符号索引非空时才注册该工具（诚实工具面策略），否则以地形提示告知降级用 find_files/grep/read。

## 证据列表
1. explore 工具定位：一次性返回相关符号源码 + 调用路径 + 影响面，供开局看图/流程问题使用，参数为 query（符号名/文件/自然语言）
   - src/tools/graph.rs:185-204（审计 #2）
2. 实现：execute 校验非空 query 后经 engine.call("explore", {query}) 转发，输出加 [codegraph explore] 前缀
   - src/tools/graph.rs:214-226（审计 #2）
3. 转发层：CodegraphEngine::call 自动附 projectPath，调用 McpClient.call_tool("codegraph_explore")
   - src/tools/graph.rs:104-110（审计 #2）
4. 上游依赖：CodegraphEngine::start 负责索引引导（init/force，带引导锁）并 spawn codegraph serve --mcp 完成握手，会话内五工具复用（模块头注释 graph.rs:1-5）
   - src/tools/graph.rs:69-102（审计 #2）
5. 注册与准入：CLI 启动时仅在 codegraph 存在且符号索引非空（graph_symbols_nonempty）时注册 ExploreTool 及 callers/callees/impact/files；否则不注册并注入地形提示引导改用 find_files/grep/read
   - src/cli.rs:273-295（审计 #11）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=4 · duration=13264ms · tokens=29786
