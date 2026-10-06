# 配置管理子命令（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
codesleuth 的 `config` 子命令由 `Cli::run` 派发到 `run_config`，按 `ConfigAction::{Get, Set, Path}` 三个动作分别查看生效配置（单键或全量）、写入全局 `config.toml`、以及打印全局/项目配置文件路径。实现上 `config_get`/`config_set` 走 `config::load` / `config::global_config_path` / `config::apply_set` / `config::resolved_get` / `config::to_file_view` 等下层函数完成读取与持久化，对外不修改其它状态。

## 证据列表
1. Cli::run 顶层派发 Command::Config 分支到 run_config，传入 action 与 profile。
   - src/cli.rs:107-108（审计 #2）
2. ConfigAction 枚举定义三个子动作：Get { key }、Set { key, value }、Path。
   - src/cli.rs:80-96（审计 #2）
3. run_config 按动作分流：Path 打印全局与项目配置路径；Get 调 config_get；Set 调 config_set 并打印写入路径。
   - src/cli.rs:480-511（审计 #2）
4. config_get 内部调 config::load 合并配置，无键走 to_file_view 序列化全量，有键走 resolved_get 读单值。
   - src/cli.rs:667-678（审计 #2）
5. config_set 内部取 global_config_path、parse_file 已有文件（或默认）、apply_set 写入键、然后 toml::to_string_pretty 写回磁盘。
   - src/cli.rs:680-701（审计 #2）
6. config::load 实现「全局文件 + 项目文件 + CLI 覆盖」的分层合并。
   - src/config.rs:190-204（审计 #2）
7. global_config_path 固定指向 ~/.codesleuth/config.toml。
   - src/config.rs:159-162（审计 #2）
8. apply_set / resolved_get 共用同一张键表（find_key），未知键报 CS1012。
   - src/config.rs:487-497（审计 #2）
9. to_file_view 把运行时 Config 投影回 FileConfig，用于 `config get` 全量展示。
   - src/config.rs:499-524（审计 #2）
10. parses_config_subcommands 测试覆盖了 config get|path 与 index --rebuild 的子命令解析正确性。
   - src/cli.rs:775-796（审计 #2）

## 死胡同
- grep "ConfigAction|run_config|Command::Config" 第一次返回 0 命中（管道被工具吞掉），改用分别 grep 与 explore 拿到全部位置。
- 未读 src/config.rs 中 parse_file / FileConfig / merge_file / merge_cli 内部实现，因属实现细节且与本概览无关。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33333ms · tokens=65967

