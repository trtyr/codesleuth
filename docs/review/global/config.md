# 宏观 · config

> 置信度 high · 12 条发现 · 455919 tokens

总评定级：**需要学习但顺理成章，且大半个身子已经跨进「开箱即用」**。这是一套刻意做过易用性治理的配置面：优先级链正是黄金三层「CLI 旗标 > 项目配置 > 全局配置 > 内置默认值」（唯一受控扩展是 D017 档位表）；环境变量层已被整体裁决移除（config.rs:3），api_key 明文住配置文件、resolve_api_key 只认文件直配，运行期 env 仅剩向 codegraph 子进程注入的 CODEGRAPH_TELEMETRY=0/DO_NOT_TRACK=1 两个自设变量，不污染用户 shell——域1近乎满分。零配置体验好：必填仅 api_key 一个，其余全有写死的合理默认（OpenAI 端点、60% 压缩阈值、向量不配自动降级），缺 key/缺 task/缺 repo 均报「缺什么+用法行」的可自纠错误。AI 友好度是亮点：--output-format json|raw|report 三态、Report 带 REPORT_SCHEMA_VERSION=1 版本化 schema、stdout/stderr 纪律干净。但仍有五处「承诺 vs 实现」接缝伤：①README:113 官方示例 --focus 全链路零消费；②README:116 `index --rebuild` 实为诚实报错的假旗标，文档未更新；③config get 全量打印丢 context 段、llm.profile 无档位返回虚构 "default"；④config set 非原子写+静默丢未知键/注释；⑤CONFIG_MISSING hint 误导用户检查从不参与的 XDG_CONFIG_HOME。最快见效三项：删除或接通 --focus（连带 README）、config set 改临时文件+rename 原子写、to_file_view 补齐 context 段。

1. 优先级链为黄金三层+档位：CLI > 项目(.codesleuth/config.toml，兼容 codesleuth.toml) > 全局(~/.codesleuth/config.toml) > 默认值；环境变量层已整体移除，模块头明确声明「一个配置文件管一切」（src/config.rs:1-3、src/config.rs:207-222）
2. 密钥走配置文件而非 env：api_key 明文直配于 [llm]，resolve_api_key 只认文件直配、空白视为未配置、值不落日志；缺失时 CS1011 + 精确 hint 指到 ~/.codesleuth/config.toml（src/config.rs:528-538、src/config.rs:115-118）
3. 运行期环境变量仅剩对 codegraph 子进程注入的 CODEGRAPH_TELEMETRY=0 / DO_NOT_TRACK=1（隐私关停用），非用户配置通道，无 env 滥用（src/tools/graph.rs:29-33）
4. 零配置体验：必填仅 api_key 一个；默认值是写死的合理实践（OpenAI 端点、gpt-4o-mini、压缩阈值 60%、向量层不配则自动降级不致命）；缺 task/--repo 报 CS1001 且 hint 给出完整用法行——报错可自纠三件套（码+人话+修法）齐备（src/config.rs:121-148、src/cli.rs:228-240、src/cli.rs:599-602）
5. 参数面克制：顶层约 14 个旗标 + config/index 两个子命令，全部有 /// doc 帮助注释；命名风格统一 kebab-case；--output-format report|raw|json 取代旧 --json，交互模式一致（src/cli.rs:22-66、src/cli.rs:99-114、src/cli.rs:518-531）
6. 机器可读输出与退出码契约：Report JSON 带 REPORT_SCHEMA_VERSION=1 版本化 schema；错误码 CSxxxx 段位→exit code 集中映射（1 用法/2 配置/3 LLM/4 目标库/5 索引/6 内部）并有测试锁定；report_error 输出主行+根因行+hint 行，结构稳定可程序化消费（src/errors.rs:8-23、src/errors.rs:108-117）
7. stdout/stderr 分离干净：json 模式 stdout 仅报告 JSON，日志双轨（stderr + ~/.codesleuth/logs/<session>.log）默认 warn 级；运行回显贴心——报告/审计落盘路径、turns/tool calls 统计、零写入自证结论均打在 stderr（src/lib.rs:24-41、src/cli.rs:501-513）
8. 【域6 硬伤】README 官方示例 --focus "crates/**" 是欺骗：clap 定义了 pub focus: Vec<String>，但 run_task_inner 全程未读该字段，工具面照常全库检索；同段 `index --rebuild` 也是假旗标（仅触发诚实报错 hint），文档仍当正式用法示例（README.md:56-122、README.md:112-117、src/cli.rs:28-30）
9. 【域4/6 不一致】config get 全量打印丢失 context 段（to_file_view 硬编码 FileContext::default()），与「打印生效配置」承诺不符；llm.profile 无档位时 get 返回虚构值 "default"，误导脚本消费方（src/config.rs:495-523、src/config.rs:358-360）
10. 【域1 破坏性】config set 全量回写非原子（fs::write 直接覆盖全局 config.toml，中途崩溃留半截 TOML 致后续所有 load 报 CS1012）；且 FileConfig 无 deny_unknown_fields，读改写回静默丢弃用户手工注释与自定义段（src/cli.rs:553-557、src/config.rs:179-188、src/config.rs:11-22）
11. 【域4 误导 hint】CONFIG_MISSING 语境提示检查 XDG_CONFIG_HOME，但 global_config_path 只用 dirs::home_dir()，XDG 从不参与——hint 把用户引向无效方向；另测试名 precedence_cli_beats_env_... 仍含已移除的 env 层，属名称级漂移（src/config.rs:159-167、src/config.rs:622-643）
12. 【域3/6 正面】配置键表驱动一处声明三处共用（apply_set/resolved_get/未知键 hint），未知键报 CS1012 且 hint 列全部键名；有全键 set→写盘→load→get 闭环回归测试——扩展面治理到位，加键成本一行（src/config.rs:312-320、src/config.rs:331-379、src/config.rs:545-580）
