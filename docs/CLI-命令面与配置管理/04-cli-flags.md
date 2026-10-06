# 命令行旗标集（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
CLI 命令旗标集由 src/cli.rs 的 Cli 结构体（clap derive）集中定义（--repo/--focus/--json/--out/--model/--base-url/--fresh-index/--vector/--repo-map/-v），入口 main.rs 调 Cli::parse → Cli::run 分发：无子命令时进入 run_task_inner 逐旗标生效（--repo 校验 canonicalize、--fresh-index 传给 codegraph 启动、--vector 装配向量层、--repo-map 构建导航图注入首条消息、--json/--out 控制 stdout/落盘），index --vector 子命令走 run_index_vector 预建向量索引，config 子命令走 run_config。上游依赖 config::load（合并 --model/--base-url 覆盖）与 bootlock 引导锁，下游消费工具注册表、harness::Harness（驱动 LLM 侦察）、审计日志与报告持久化。

## 证据列表
1. 全部旗标在 Cli 结构体集中定义：--repo(--repo) --focus(可多次 glob) --json --out --model --base-url --fresh-index(强制重建索引) --vector(向量召回) --repo-map(repo map 注入) -v 日志级别
   - src/cli.rs:20-55（审计 #2）
2. 入口为 main.rs：Cli::parse → init_tracing(verbose) → cli.run(session_id)；Cli::run 按子命令分发到 run_config / run_index_vector / run_task
   - src/main.rs:5-11（审计 #4）
   - src/cli.rs:96-126（审计 #2）
3. run_task_inner 校验 task 与 --repo（缺失报 USER_INPUT 并给用法提示，canonicalize 报 REPO_NOT_FOUND），随后 config::load(overrides) 合并 --model/--base-url 覆盖
   - src/cli.rs:190-223（审计 #2）
4. --fresh-index 透传给 CodegraphEngine::start；--vector 触发 setup_vector_layer；仅 --repo-map 时用 vector::repomap 构建全局导航图注入首条消息
   - src/cli.rs:252-253（审计 #2）
   - src/cli.rs:299-311（审计 #2）
   - src/cli.rs:334-351（审计 #2）
5. --out 将报告写入指定文件（--json 时为 JSON 否则 Markdown），--json 控制 stdout 仅输出 JSON 报告；报告同时持久化到 ~/.codesleuth/reports/（cli.rs:428-436）
   - src/cli.rs:437-447（审计 #2）
6. index 子命令仅支持 --vector 走 run_index_vector（预建向量索引，先 acquire_guard 引导锁），非 vector 时 index_structure_error 诚实报错并提示 run --fresh-index
   - src/cli.rs:128-163（审计 #2）
   - src/cli.rs:459-472（审计 #2）
7. 旗标集下游消费：工具注册表（read/fuzzy/graph 五工具，vector 层按 --vector 注册）、harness::Harness::run 驱动 LLM 侦察
   - src/cli.rs:240-296（审计 #2）
   - src/cli.rs:353-365（审计 #2）
8. config 子命令（path/get/set）由 run_config 分发到 config 模块，属于配置管理面
   - src/cli.rs:474-505（审计 #2）

## 死胡同
- grep 'cli.focus' 无命中：--focus 的运行时消费点未定位，仅在 Cli 定义（src/cli.rs:26-28）与测试/README 中出现
- grep 'fn run\(|CliOverrides' 正则模式 0 命中，改用 plain 'run_task' 检索后定位

## 置信度
high

## 统计
turns=12 · tool_calls=11 · duration=42588ms · tokens=88164
