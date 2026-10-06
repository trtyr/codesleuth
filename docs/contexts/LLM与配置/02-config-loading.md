# 配置加载链（LLM与配置）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
配置加载链按"CLI 参数 > 项目配置 > 全局配置 > 默认值"优先级，把多源 TOML 配置合并成运行时 Config（src/config.rs:191-223），环境变量层已整体移除（src/config.rs:1-3）。入口是 config::load，先解析全局/项目可选路径后委托纯函数 load_layered：先 Config::default，再用 merge_file 依次覆盖全局、项目层，再 apply_profile 应用 --profile 档位，最后 merge_cli 写入 CliOverrides。

## 证据列表
1. 配置加载链的优先级与"环境变量层已移除"在文件头文档注释中明文规定
   - src/config.rs:1-3（审计 #2）
2. load 是公开入口，解析全局与项目两处可选路径后委托 load_layered
   - src/config.rs:191-203（审计 #2）
3. load_layered 是纯函数加载链：默认 ← 全局 ← 项目，再 apply_profile、merge_cli
   - src/config.rs:207-223（审计 #2）
4. apply_profile 先于 merge_cli 执行，--model/--base-url 旗标永远是最后覆写者
   - src/config.rs:225-259（审计 #2）
5. merge_file 逐字段以 Option 形式覆盖，缺席字段不覆盖（FileConfig 全部 Option）
   - src/config.rs:9-22（审计 #2）
   - src/config.rs:261-299（审计 #2）
6. 全局配置固定位于 ~/.codesleuth/config.toml，项目配置为 .codesleuth/config.toml 并兼容旧 ./codesleuth.toml
   - src/config.rs:160-176（审计 #2）
7. CliOverrides 与 clap 解耦，仅承载 base_url/model/profile 三个可选字段
   - src/config.rs:150-157（审计 #2）
8. CLI 多处通过 config::load 注入 CliOverrides 触发加载链
   - src/cli.rs:140（审计 #9）
   - src/cli.rs:221（审计 #9）
   - src/cli.rs:668（审计 #9）
9. run_config（config 子命令入口）展示全局/项目路径，并提供 get/set
   - src/cli.rs:480-509（审计 #9）
10. 测试用例直接覆盖了"CLI > 项目 > 全局"与"缺文件回落默认"两个核心契约
   - src/config.rs:622-643（审计 #2）
   - src/config.rs:681-685（审计 #2）

## 死胡同
- 未读 src/config.rs:300-599 之间的 merge_file 收尾、merge_cli 实现细节（本任务为概览级侦察，无需展开）
- scripts/eval/run_eval.py:110-118 仅作为向量召回线索，未 read 原文确认
- 未在 src/cli.rs 中确认 CliOverrides 各字段的 clap 绑定行号（仅确认 run_config 位于 src/cli.rs:480）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=30702ms · tokens=47925

