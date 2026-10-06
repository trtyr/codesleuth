# 主侦察命令（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
主侦察命令是 codesleuth 的核心 CLI 入口：用户传入自然语言任务与目标仓库，经配置加载、工具注册（read/grep/graph/vector）与 Agent harness 驱动 LLM 完成检索，最终输出带证据的结构化报告（stdout/--out/落盘 reports/）。调用链：main → Cli::parse → Cli::run → run_task_inner → harness::Harness::run。

## 证据列表
1. Cli 结构定义任务/repo/--json/--out/--fresh-index/--vector/--repo-map 等参数，为 clap Parser 入口
   - src/cli.rs:14-55（审计 #2）
2. main 解析 Cli、生成 session_id 并调用 cli.run
   - src/main.rs:5-11（审计 #12）
3. run_task_inner 校验 task/repo、加载配置、构造 LLM provider 与工具注册表（read/fuzzy），最后由 Harness::run 驱动 agent
   - src/cli.rs:190-244（审计 #2）
   - src/cli.rs:353-365（审计 #2）
4. 可选注册 codegraph 图工具（explore/callers/callees/impact/files）与向量召回层，均支持非致命降级并留痕审计
   - src/cli.rs:252-296（审计 #2）
   - src/cli.rs:299-328（审计 #2）
5. 任务结束后序列化结构化报告，按 --json/--out 输出并落盘 ~/.codesleuth/reports/{session}.md/.json
   - src/cli.rs:423-447（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=23427ms · tokens=22918
