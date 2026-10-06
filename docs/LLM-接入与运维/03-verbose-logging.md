# -v 日志详细度（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
-v 旗标是 CLI 的日志详细度计数器（-v=info，-vv=debug，默认 warn），入口解析后在 main 中传给 init_tracing 建立 stderr + 会话日志文件的双轨 tracing 过滤，让用户/运维按需增减日志噪音并保留归档。

## 证据列表
1. -v 是 clap 计数旗标 verbose: u8，注释写明「-v info / -vv debug」
   - src/cli.rs:50-52（审计 #12）
2. init_tracing 将计数映射为过滤级别：0→warn、1→info、≥2→debug，构造 EnvFilter=codesleuth=<level>
   - src/lib.rs:27-37（审计 #2）
3. 日志双轨输出：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同一过滤器共用；--json 模式下 stdout 只出报告 JSON
   - src/lib.rs:24-26, 39-65（审计 #2）
4. main 解析 Cli 后调用 init_tracing(cli.verbose, &session_id)，session_id 由 audit::new_session_id 生成，审计与日志共用身份
   - src/main.rs:5-10（审计 #14）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=17058ms · tokens=15176

