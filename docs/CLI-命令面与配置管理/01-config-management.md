# config 配置管理（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
config 命令提供 `codesleuth config get/set/path`，让用户查看生效配置、把 llm/vector/context/behavior/graph 等键写入全局配置文件 ~/.codesleuth/config.toml、或查看全局/项目配置文件路径，是 CLI 配置管理的用户入口。

## 证据列表
1. 功能价值：config get 可无键输出全部生效配置（转成 TOML）或按键读取；set 将键值写入全局配置；path 打印全局/项目配置文件位置
   - src/cli.rs:474-505（审计 #2）
   - src/cli.rs:661-669（审计 #2）
2. 入口：clap 解析出 Command::Config 后调用 run_config(action)，分派到 config::global_config_path/project_config_path、config_get、config_set
   - src/cli.rs:103-104（审计 #2）
   - src/cli.rs:474-505（审计 #2）
3. set 链路：config_set 取 global_config_path，读现有文件或 FileConfig::default，经键表驱动的 apply_set 校验写入，再序列化 TOML 落盘并自动建目录
   - src/cli.rs:671-692（审计 #2）
   - src/config.rs:418-422（审计 #4）
4. get 链路：config_get 调 config::load 分层加载（默认值←全局←项目←CLI 覆盖），经 to_file_view 展示或 resolved_get 按键取值，读写共用 config_key_table 键表
   - src/config.rs:173-201（审计 #4）
   - src/cli.rs:661-669（审计 #2）
   - src/config.rs:254-262（审计 #4）
   - src/config.rs:424-428（审计 #4）
5. 配置文件定位：全局固定 ~/.codesleuth/config.toml，项目优先 .codesleuth/config.toml 并兼容旧 codesleuth.toml
   - src/config.rs:142-159（审计 #4）
6. 与其他模块交互：下游 config::load 是全 CLI（run 任务等）取生效配置的统一入口，llm/vector/context/behavior/graph 各段供向量、上下文压缩、思考开关等功能消费；上游依赖 clap 的 Command::Config 解析
   - src/cli.rs:96-104（审计 #2）
   - src/config.rs:173-201（审计 #4）
   - src/config.rs:203-243（审计 #4）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=21517ms · tokens=41059

