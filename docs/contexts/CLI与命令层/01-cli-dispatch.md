# CLI入口与命令分发（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
CLI 入口与命令分发是 codesleuth 的进程入口：main() 解析参数、生成 session_id、初始化结构化日志双轨（stderr + 会话文件），随后把控制权交给 Cli::run，按子命令分派到 Config / Index（向量） / run_task。给用户的价值是「一个二进制、按意图路由」：配置查看与写入、向量索引构建、只读侦察任务三类操作同走同一条入口与日志/审计身份。

## 证据列表
1. main() 在 src/main.rs 解析 CLI、生成 session_id、初始化日志后调用 Cli::run。
   - src/main.rs:5-12（审计 #2）
   - src/main.rs:6（审计 #2）
   - src/main.rs:8（审计 #2）
   - src/main.rs:9-10（审计 #2）
2. 日志双轨由 lib::init_tracing 装配：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同过滤同格式。
   - src/lib.rs:24-27（审计 #14）
   - src/lib.rs:39-58（审计 #14）
   - src/lib.rs:61-65（审计 #14）
3. Cli 是 clap 派生的命令行结构（任务、--repo、--vector、--fresh-index、--json、--out、-v、Config/Index 子命令等），是一级分派的承载体。
   - src/cli.rs:14-58（审计 #2）
   - src/cli.rs:60-96（审计 #2）
4. Cli::run 用 match self.command 做一级分派：Config → run_config；Index 且 --vector → run_index_vector；其它 Index → 诚实报错；None → run_task。
   - src/cli.rs:99-129（审计 #2）
5. run_task 内调 run_task_inner 装配 config、audit、fence、tools 注册、codegraph MCP、向量层、harness，再交给 agent.run。
   - src/cli.rs:184-192（审计 #2）
   - src/cli.rs:194-371（审计 #2）

## 死胡同
- 未读 src/cli.rs:76-805 中 ConfigAction 详细分支与 parses_* 辅助函数；本次只验证入口与一级分派，不影响四节结论。

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=26925ms · tokens=33794

