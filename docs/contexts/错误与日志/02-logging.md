# 日志双轨（错误与日志）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
「日志双轨」功能在 init_tracing 中通过 tracing registry 同时挂 stderr_layer 与 file_layer 实现：stderr 给人实时看、~/.codesleuth/logs/<session>.log 留作归档，两轨共用同一 EnvFilter 与同一格式，--json 模式下 stdout 仍只承载报告 JSON；归档由 SessionLog 单代轮转（64MB 超限 → .log.old），并在 session span 内自动携带 session_id 完成结构化串线。

## 证据列表
1. 功能注释明确定义日志双轨：stderr 人读 + ~/.codesleuth/logs/<session>.log 归档，同过滤同格式；--json 下 stdout 严禁日志
   - src/lib.rs:24-26（审计 #2）
   - src/logs.rs:1-3（审计 #4）
2. init_tracing 由 main 启动时调用，传入 verbose 与 session_id；session_id 来自 audit::new_session_id 以保证日志与审计同一身份
   - src/main.rs:3, 7-9（审计 #14）
3. init_tracing 组装 EnvFilter（按 verbose 选 warn/info/debug）、stderr_layer、file_layer，挂在同一 registry，文件层共用同一过滤器
   - src/lib.rs:27-66（审计 #2）
4. 文件层路径来自 global_state_dir() + logs/<session_id>.log，打开失败仅 eprintln 警告并降级为仅 stderr，不阻断
   - src/lib.rs:43-58（审计 #2）
5. SessionLog 单代 64MB 轮转：超限 rename 到 .log.old，append 打开；提供 MakeWriter 实现供 tracing 使用
   - src/logs.rs:10-45, 64-70（审计 #4）
6. 测试验证 session span 内事件自动带 session_id 串线到输出行（结构化串线）
   - src/logs.rs:105-143（审计 #4）

## 死胡同
- 未深入 errors.rs 的 report_error（与日志双轨主链无直接耦合，不影响结论）
- 未读 src/audit 等无关模块

## 置信度
high

## 统计
turns=6 · tool_calls=4 · duration=27363ms · tokens=39519

