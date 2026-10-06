# grep 内容检索（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
grep 是仓库内内容检索工具，支持 plain（默认）/regex/fuzzy 三种匹配模式，供调用方以 {pattern, mode?, limit?} 在仓库范围内拿到 "file:line: content" 形式的命中结果，是层进检索协议中「顺藤摸点」的定位手段。

## 证据列表
1. 功能价值：grep 在仓库内做内容检索，支持 plain/regex/fuzzy 三态模式，返回 file:line:content 命中，是层进检索协议的定位手段（fuzzy.rs 模块注释明确 find_files/grep 的分工）。
   - src/tools/fuzzy.rs:112-131（审计 #2）
2. 入口与关键文件：GrepTool 定义于 src/tools/fuzzy.rs:113（impl Tool name="grep"，execute 在 145-207）；CLI 启动时在 src/cli.rs:242-244 创建 FuzzyEngine 并注册 GrepTool；单元测试 grep_plain_finds_lines 位于 src/tools/fuzzy.rs:238-248。
   - src/tools/fuzzy.rs:124-143（审计 #2）
   - src/tools/fuzzy.rs:145-181（审计 #2）
   - src/cli.rs:242-244（审计 #10）
   - src/tools/fuzzy.rs:17-37（审计 #2）
   - src/tools/fuzzy.rs:153-157（审计 #2）
   - src/tools/fuzzy.rs:164-181（审计 #2）
   - src/tools/fuzzy.rs:238-248（审计 #2）
3. 运作方式：调用方传 {pattern, mode?, limit?} → GrepTool::execute 将 mode 字符串映射为 fff_search 的 GrepMode（fuzzy.rs:153-157），构建 GrepSearchOptions（smart_case、limit clamp 1-200 等）后调用共享 FuzzyEngine 内 FilePicker.grep（fuzzy.rs:181），再格式化为 "file:line: content" 输出（fuzzy.rs:183-206）。
   - src/tools/fuzzy.rs:145-207（审计 #2）
   - src/cli.rs:242-244（审计 #10）
4. 模块交互：上游依赖 fff_search crate（FilePicker/GrepMode/GrepSearchOptions/QueryParser，fuzzy.rs:9-12）、crate::errors 错误码与 crate::fence::Fence（fuzzy.rs:5-6，FuzzyEngine::new 用 Fence 取规范化根目录，fuzzy.rs:23-27）；与 ReadTool、FileFinderTool 共享同一 Arc<FuzzyEngine> 注册进 ToolRegistry（cli.rs:241-244）；下游被 tests/adversarial.rs:37、tests/replay.rs:69、tests/layered_live.rs:33 注册消费，零命中降级由 fff 处理（fuzzy.rs:112）。
   - src/tools/fuzzy.rs:9-15（审计 #2）
   - src/tools/fuzzy.rs:5-7（审计 #2）
   - src/tools/fuzzy.rs:114（审计 #2）
   - src/cli.rs:238-244（审计 #10）
   - src/tools/fuzzy.rs:187-193（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16744ms · tokens=28399
