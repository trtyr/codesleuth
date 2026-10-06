# find_files 文件查找（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
find_files 是一个基于 fff-search 的模糊文件名查找工具（frecency 排序），输入 query（文件名/路径片段，支持约束语法）和可选 limit（默认 20，上限 100），帮助调用方按名字快速定位仓库内文件。入口是 FileFinderTool（src/tools/fuzzy.rs:41-110），CLI 启动时创建会话级 FuzzyEngine 并注册该工具（src/cli.rs:242-244）；执行时经 QueryParser 解析 query 后调用共享 FuzzyEngine 中 FilePicker.fuzzy_search 完成检索（src/tools/fuzzy.rs:86-92）。它与 GrepTool（内容检索）共享同一引擎、同属「读层+模糊层」检索工具面，上游依赖 crate::errors、crate::fence::Fence 与 fff-search crate；集成测试（tests/adversarial.rs、tests/replay.rs、tests/layered_live.rs）也将它与 GrepTool 注册在同一 ToolRegistry。

## 证据列表
1. FileFinderTool 是名为 find_files 的 Tool，按名字模糊找文件（frecency 排序），参数为 query（必填）与 limit（默认20，上限100）
   - src/tools/fuzzy.rs:40-71（审计 #2）
2. execute 解析 query/limit，QueryParser 解析后调用 engine.picker.fuzzy_search，输出编号路径列表，超限时提示可提高 limit
   - src/tools/fuzzy.rs:73-108（审计 #2）
3. CLI 启动时创建 FuzzyEngine 并把 FileFinderTool 与 GrepTool 注册进 ToolRegistry（层进 v0：读层+模糊层）
   - src/cli.rs:238-244（审计 #10）
4. FuzzyEngine 为会话级引擎，watch=false 同步收集文件，封装 fff-search FilePicker；FileFinderTool 与 GrepTool 共享同一 Arc<FuzzyEngine>（41-43、113-115）
   - src/tools/fuzzy.rs:17-37（审计 #2）
5. 上游依赖：crate::errors 错误码、crate::fence::Fence（取规范化根目录定界 picker）、fff-search crate（fuzzy.rs:5-14），单测 find_files_fuzzy_matches_name 覆盖模糊命中
   - src/tools/fuzzy.rs:222-236（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=13219ms · tokens=22692
