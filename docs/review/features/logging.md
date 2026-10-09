# 微观 · 日志双轨（logging）

> 置信度 medium · 7 条发现 · 138518 tokens

「日志双轨」链路整体实现正确：init_tracing 在 registry 上挂共享 EnvFilter + stderr/file 两层 fmt::layer，SessionLog 用 Arc<Mutex<File>> 提供线程安全 MakeWriter，session span 串线经 cli.rs:119 进入后自动注入两轨，--json 下无任何日志走 stdout。未发现 P0/P1/P2 级正确性缺陷；有 3 条 P3 级问题（运行期无轮转导致 64MB 上限只在 open 时检查、轮转失败被静默吞掉且注释声称的 warn 不存在、SessionLog::path() 零调用）与 2 条疑似设计取舍（默认 warn 级使归档轨近乎空载、单代轮转因 session_id 每次唯一而近乎不可达）。死代码维度：确定死 0、疑似死 1（path()，pub API 入不可判名单）。

1. P3｜轮转只发生在 open 时刻，运行期永不轮转：SessionLog::open 在 23-29 行检查超限并 rename，之后（33-36 行）append 打开，Write impl（48-62 行）没有任何大小检查——一次长会话可无限超 64MB。触发条件：同一 session 文件持续追加超限；错误行为 vs 应有行为：文档/注释（logs.rs:2「超限 → .old」）暗示 64MB 上限被强制，实际只在下次 open（即下次碰巧复用同名文件）才轮转。上游防线不成立论证：Write 路径完全绕过大小检查，fmt layer 也不会调用 open。（src/logs.rs:22-36、src/logs.rs:48-62）
2. P3｜轮转失败被完全静默：logs.rs:28 `let _ = std::fs::rename(...)`，注释声称「warn 由调用方日志可见」，但此时 tracing 尚未初始化（init_tracing 在 lib.rs:46 调用 open，registry 在 61-65 行才 .init()），且代码根本没有发出任何 warn——错误分支零输出，注释与实现不符。触发条件：rename 失败（权限/.old 被占用等）；行为：无任何告警地继续追加超限文件。（src/logs.rs:26-29、src/lib.rs:43-58）
3. 疑似死代码：SessionLog::path()（logs.rs:43-45）在全部 141 个已索引文件中零调用（grep ".path()" 零命中，SessionLog 全部引用仅有 lib.rs:46 的 open 与测试）。因它是 pub API（lib crate），列入不可判名单，属疑似死。（src/logs.rs:43-45、src/main.rs:8-10）
4. P3 疑似（设计取舍）｜64MB 单代轮转在实际运行中近乎不可达：session_id = 纳秒时间+pid（audit.rs:54-60）每次运行唯一，logs/<session_id>.log 每次新建，几乎不可能超 64MB 再被 open 复用；轮转逻辑仅被手工构造文件的测试（logs.rs:76-89）触达。这使 LOG_ROTATE_BYTES 与轮转分支成为实际近乎死路（但实现正确）。验证方法：统计 ~/.codesleuth/logs/ 下是否存在任何 .old 文件。（src/audit.rs:53-60、src/lib.rs:31-37、src/logs.rs:76-103）
5. P3 疑似（设计取舍）｜「同过滤」使归档轨默认近乎空载：verbose=0 时 filter=warn（lib.rs:31-35），两轨共享（lib.rs:61-65），默认会话归档文件几乎只含 warn 级事件，与「留作归档」的目的存在张力——但这是「同过滤同格式」注释的显式承诺，非缺陷。（src/lib.rs:31-37、src/lib.rs:61-65）
6. 已审干净的部分及方法：init_tracing 全函数（lib.rs:27-66）逐行读——EnvFilter 构造的 fallback、两层 with_target(false)、file 层 with_ansi(false) 与打开失败 eprintln 降级不阻断、None layer 挂 registry 均正确；main→init_tracing→cli.run 顺序与 session span（cli.rs:119 entered 守卫覆盖 run 全程）正确；stderr writer 与 file writer 均无 stdout 写入，--json 纪律成立；SessionLog 的 Write 用 poisoned.into_inner() 恢复（logs.rs:52,59）避免 panic；with_extension("log.old") 在 <hex>-<pid>.log 上产出正确（无点号截断风险）。边界代入：空 buf、零字节文件（不轮转，>max 严格比较正确）、恰好 64MB（不轮转，符合文档「超限」语义）均通过心算。（src/lib.rs:27-66、src/cli.rs:117-148、src/main.rs:5-12）
7. errors.rs 链路面（report_error/exit_code）核对：report_error 只写 stderr（errors.rs:110-116），与 stdout 纪律一致；exit_code 段位映射与文档一致；audit::new_session_id 的 SystemTime::now().unwrap_or_default() 在时钟早于 epoch 时退化为 0-pid（可容忍）。无缺陷。（src/errors.rs:109-117、src/errors.rs:13-22、src/audit.rs:13-60）
