# files 文件关联查询（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
files 是只读工具面里的 codegraph 结构图工具之一：列出已索引的仓库文件结构，支持可选 glob 过滤（如 src/**），帮助 Agent 在做结构/影响面分析前先看仓库文件地图。入口为 FilesTool（src/tools/graph.rs），调用链一句话：FilesTool::execute 取 filter 参数后调 CodegraphEngine::call("files")，后者附上 projectPath 并经 MCP client 调用外部 codegraph_files 工具，结果加「[codegraph files]」前缀返回。仅当仓库存在非空符号索引时才与 explore/callers/callees/impact 一起注册，否则提示改用 find_files/grep/read。

## 证据列表
1. files 工具由 FilesTool 实现，描述为「列出已索引的文件结构（可按 glob 过滤）」，唯一可选参数 filter（glob）。
   - src/tools/graph.rs:229-248（审计 #2）
2. 运作链：FilesTool::execute 收集 filter 后调 CodegraphEngine::call("files")，call 自动附 projectPath 并经 MCP client 调用 codegraph_files 外部工具，结果以 [codegraph files] 前缀返回。
   - src/tools/graph.rs:257-264（审计 #2）
   - src/tools/graph.rs:104-110（审计 #2）
3. 注册条件：cli.rs 在 codegraph 符号索引非空时才注册 FilesTool 与 Explore/Callers/Callees/Impact；否则给地形提示改用 find_files/grep/read。FilesTool::new 接收共享 Arc<CodegraphEngine>。
   - src/cli.rs:282-295（审计 #9）
   - src/tools/graph.rs:230-237（审计 #2）
4. 下游依赖为外部 codegraph 二进制；graph.rs 测试覆盖二进制缺失时返回结构化错误（INDEX_NOT_AVAILABLE）。
   - src/tools/graph.rs:267-278（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=3 · tool_calls=4 · duration=9648ms · tokens=13185
