# 微观 · System Prompt（system-prompt）

> 置信度 medium · 7 条发现 · 538882 tokens

System Prompt 链路整体实现扎实：SYSTEM_PROMPT 常量由 Harness::run 恒定注入首条 system 消息，任务派生数据（召回块/导航图）正确走 first_user_suffix 附加在 user 消息上而非 system（D005 分层被严格遵守），压缩时 compact() 保留 head=2（system+task）保证 prompt 恒在。发现 1 个 P2 真实缺陷：CLI 的 --focus 旗标（README 文档化、clap 定义、测试断言均存在）在 run_task_inner 中从未被消费——声明了限定检索范围却完全被忽略，工具面照常全库检索，属"文档承诺 vs 实现不一致"的半成品。另有若干 P3：report.rs validate() 滥用 INDEX_BUILD_FAILED（exit 5）承载报告校验错误；llm.rs 退避注释与实现不符；降级 prose 报告构造路径未经 validate()。死代码维度：无确定死代码；最值得清理的是 errors.rs CsError::retryable()（生产零引用，仅测试用）。bug 密度最高文件：src/cli.rs。

1. P2 · --focus 旗标完全未接线：clap 定义 pub focus: Vec<String>（cli.rs:28-30），README:113 把 --focus "crates/**" 作为正式用法文档化，cli.rs 测试 parses_full_surface（822-849）断言 focus.len()==2；但 run_task_inner（228-515）全程未读 self.focus——Fence/FuzzyEngine/ReadTool/导航图装配均不接收过滤。触发：用户按 README 用 --focus。错误行为：全库检索、范围限定静默失效；应有：过滤生效或旗标报错。上游防线不成立：无 warn/错误，用户无从得知。（src/cli.rs:28-30、src/cli.rs:228-515、README.md:112-113）
2. P3 · Report::validate() 错误段位误用：非法置信度/空 answer 返回 INDEX_BUILD_FAILED（CS4011），按段位映射 exit code 5（索引），实为报告校验问题；当前 harness 用 map_err 转 String（harness.rs:659）丢了码位，影响有限，但直接消费该公共 API 会得误导退出码。（src/report.rs:59-74、src/errors.rs:13-22）
3. P3 · 降级 prose 路径绕过 validate()：degraded_prose 构造后直接返回（harness.rs:242-253），strip_task_echo 剥空任务回显后可产出空 answer 报告并写盘；非降级路径 answer 为空会被拒，两路径不对称。（src/harness.rs:212-253、src/report.rs:46-57）
4. P3 · llm.rs 退避注释与实现不符：注释称「500ms / 1s / 2s / 4s」，实现 500u64 << attempt.min(4)，attempt=1 即 1s——500ms 档从未出现，行为自洽但文档失真。（src/llm.rs:245-249）
5. 干净段核验：SYSTEM_PROMPT 注入链（harness.rs:92-106 → llm.rs to_request_message System 映射 → compact head=2 恒保 system+task）三方一致；submit_report/recall 内置 schema 与 SYSTEM_PROMPT 第 23 行 JSON 形状一致；prompt.rs 锚点测试覆盖契约词。审法：沿首条消息产生点到 LLM 序列化点全程跟踪，含压缩后再注入路径。（src/harness.rs:92-106、src/context.rs:57-82、src/harness.rs:487-521）
6. 干净段核验：审计 seq 锁内取号+写盘单调性（audit.rs:144-162）、recall 读回排序过滤、evidence observe/cite_seq 键匹配与 harness 消费点（recall 工具、build_report）契约一致；recall from 缺省 0/to 缺省 from 只产生空结果，无越界风险。（src/audit.rs:144-162、src/harness.rs:344-360、src/evidence.rs:44-55）
7. 死代码盘点：errors.rs 预留常量已注释说明（4012/4013/5099）不算残余；CsError::retryable()（errors.rs:93-96）生产零引用、仅测试 builder_fluency 使用，列「疑似死」（公共 API），最值得先清理；audit.rs Ledger 砍除有诚实注释，属已裁决说明而非死代码。（src/errors.rs:93-96、src/errors.rs:142-146、src/audit.rs:166-168）
