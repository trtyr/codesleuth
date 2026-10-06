# 错误码体系（错误与日志）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
错误码体系（CSxxxx）以段位决定 exit code（CS1xxx→1 用法、CS2xxx→3 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其它→6），由 src/errors.rs 统一定义 CsCode/CsError/report_error；各业务模块用 CsError 返回错误，cli 三个顶层入口（run/run_task/run_config）捕获后调用 report_error 打 stderr 并按 exit_code 退出。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- 未深入展开 LLM/classify_llm_error 与 cli/index_structure_error 的内部错误分类细节（仅做上下游引用层确认）
- 未读取 cli.rs 三个入口的完整实现，只确认其调用 report_error 与 exit_code

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=20599ms · tokens=17295

