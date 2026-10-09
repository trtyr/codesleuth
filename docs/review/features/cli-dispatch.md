# 微观 · CLI入口与命令分发（cli-dispatch）

> 置信度 high · 5 条发现 · 757843 tokens

CLI 入口与命令分发链总体质量高：session_id 统一身份、审计取号在锁内、引导锁三态语义清晰、降级路径全部留痕。但发现一个确定的 P2 正确性缺陷：`--focus` CLI 参数（含 README 官方示例）在全仓库零消费——声明「限定检索范围」却从未传入 FuzzyEngine/Fence/grep，用户以为缩小了范围实际全仓扫描。另有若干 P3：`config get` 忽略 --model/--base-url 覆盖（与「打印生效配置」承诺不一致）；`run_index_vector` 的 OpponentFinished+产物缺失路径在无锁状态下构建，存在并发窗口（代码注释自认）。死代码维度：确定死 2 项（--focus 参数、--rebuild 旗标仅剩文案作用）。bug 密度最高文件：src/cli.rs。最值得先修：--focus 要么接通到 picker/fence 限定，要么删除参数与 README 示例。

1. P2 bug（确定）：`--focus` 参数全链路零消费。Cli 定义 `pub focus: Vec<String>`（可多次），README:113 官方示例 `--focus "crates/**"`，但全仓 grep 无任何读取点——run_task_inner 不读该字段，FuzzyEngine::new 与 Fence 均不接收范围参数。用户以为限定了检索范围，实际全仓扫描，且无任何「未生效」提示。上游防线不成立：clap 只解析，无下游消费即无防线。（src/cli.rs:28-30、README.md:112-113）
2. P3 bug（确定，契约不一致）：`config get` 声称打印「生效配置」，但 run_config 只透传 profile，忽略 --model/--base-url；而 run_task_inner 的 CliOverrides 含全套覆盖。`codesleuth --model m config get llm.model` 显示配置文件值而非实际生效值。（src/cli.rs:724-728、src/cli.rs:250-255）
3. P3 bug（疑似，并发窗口）：run_index_vector 的 OpponentFinished 且产物不存在的罕见路径在完全不持锁状态下构建（`_ => None` 分支无 guard），此时第三进程可同时拿到 Won 并行构建同一 index_dir。注释自认「增量事务保护仍有效」——该保护在 store 层（清单外，仅接口面，未验证）。验证方法：三进程并发 index --vector 观察产物完整性与重复嵌入。（src/cli.rs:197-201、src/cli.rs:647-649）
4. 审过且干净的部分（说明方法）：①退出码链：所有错误路径统一 report_error + exit_code()，段位映射与 cli.rs 文档头三方对质一致；②session_id 全程数据流：main 生成→init_tracing 文件层→session span→Audit 文件名→reports/{id}.md/.json，同一身份无分叉；③审计 seq 单调：record 锁内取号，write_check 在 harness drop 后以文件最大 seq+1 续号，不与内存计数冲突；④Runtime/cg 生命周期：agent 先 drop、cg 后 drop，MCP 回收次序正确；⑤fence 双层检查（词汇归一+canonicalize 复检）与 ReadTool 消费契约（offset>total 诚实回显、limit clamp）代入空/零/一均成立；⑥writeguard symlink 跳过与 >1MB 指纹化边界无差一；⑦config 合并顺序 global→project→CLI、profiles 项目层胜出、apply_profile 先于 merge_cli，与文档优先级一致。（src/errors.rs:13-22、src/cli.rs:117-148、src/audit.rs:144-147）
5. 死代码盘点：确定死 2 项——①`--focus` 参数（见 bug#1）；②`Command::Index --rebuild` 旗标仅在 index_structure_error 的 hint 文案分支出现，无任何行为作用。疑似死 0：预留错误码常量（INDEX_STALE/INDEX_CORRUPT/ENGINE_NOT_WIRED）已被注释声明预留隔离；audit.rs:166-168 账本注释是「写一半功能」的裁决说明，不算死代码。最值得先清理：--focus（连 README 一起，否则持续误导）。（src/cli.rs:86-95、src/cli.rs:518-531、src/errors.rs:50-56）
