# 微观 · 错误码体系（error-codes）

> 置信度 high · 7 条发现 · 298753 tokens

错误码体系主链（errors.rs 定义 → 各业务模块产生 CsError → cli 三入口 report_error + exit_code 退出 → main.rs std::process::exit）实现整体正确：段位→exit code 映射集中且与测试一致，三入口 Err 分支都统一走 report_error。但发现一处真实分类缺陷：生产路径 OpenAiProvider::chat 把所有 HTTP 5xx 归为 CS2002 LLM_RATE_LIMITED（llm.rs:262-267），而体系里专设的 CS2003 LLM_SERVER 在生产代码中零引用、仅存活于 #[cfg(test)] 的 classify_llm_error（llm.rs:315-332）——签名/文档承诺 5xx→LLM_SERVER 与生产行为不一致，且速率限制误报会误导用户。死代码维：CS2003 是本链唯一确定死（生产零引用）的常量，其余常量均有生产使用；errors.rs:50/56 的预留段位注释是有意登记非残余。bug 维：P0-P2 为 0，P3 一项；密度最高文件 src/llm.rs。最值得先修：chat 中 5xx 分支改用 LLM_SERVER（或删除 LLM_SERVER 并同步文档），恢复码位与语义一致。

1. 维A·P3：生产分类把 HTTP 5xx 与 429 一并归为 CS2002 LLM_RATE_LIMITED（retryable=true），而错误码体系专设的 CS2003 LLM_SERVER 仅被 #[cfg(test)] 的 classify_llm_error 引用，生产路径从未使用——5xx 网关故障被误报为『速率受限』，码位与语义错位（触发条件：任何上游 5xx；错误行为 CS2002 rate-limit 误报，应报 CS2003 LLM_SERVER。上游防线不成立：exit code 同为 3、消息含原始 status，掩盖了误分类，但 retryable/监控语义失真）（src/llm.rs:260-273、src/errors.rs:36-42）
2. 维A 三方对质确认：classify_llm_error（含 5xx→LLM_SERVER 分支）位于 #[cfg(test)]，是被测试保护但不在生产链路上的『影子分类器』——测试 error_classification 通过不代表生产行为一致，恰构成上一条 bug 的证据（src/llm.rs:312-332、src/llm.rs:420-437）
3. 段位→exit code 映射集中且正确：1000-1009→1 用法、1010-1099→2 配置、CS2xxx→3、CS3xxx→4、CS4xxx→5、其它（含 CS5xxx 与越界码）→6；单测逐段断言一致。任务线索中『CS1xxx→1』不精确，实际 CS1xxx 细分两档（文档详稿已自行纠正）（src/errors.rs:13-22、src/errors.rs:125-133）
4. 三顶层入口（Cli::run 对 Index 分支、run_task、run_config 的 Get/Set）Err 分支统一 report_error(&e) + e.exit_code()，main 以 std::process::exit(code) 收口——错误出口无遗漏分支；ConfigAction::Path 无错误路径返回 0（src/cli.rs:125-148、src/cli.rs:218-226、src/cli.rs:543-562）
5. report_error 输出契约（主行 Display『CSxxxx: msg』+ 根因行 + hint 行）与终局错误统一 with_retryable(false) 的约定在链路中一致执行；LLM 重试环 429/5xx/网络错误在 attempts 内重试、超限后终局返回，逻辑自洽（src/errors.rs:109-117、src/llm.rs:282-287、src/llm.rs:303-306）
6. INDEX_LOCKED（CS4016）判负路径在 cli 两处显式 return Err（codegraph 引导、向量层装配），不走降级 catch-all；bootlock 三态语义（Won/超时→CS4016/等待获得→复用）与 D020 注释、测试一致——共享交叉点审查通过（src/cli.rs:299-312、src/cli.rs:355-366、src/bootlock.rs:101-121）
7. 维B：CS2003 LLM_SERVER 为本链唯一确定死的公共常量（生产零引用，仅 test 模块使用；不在序列化/配置/注册表中）；INDEX_TIMEOUT 有生产使用，errors.rs:50/56 的 4012/4013/5099 预留注释系有意登记（P005 R6.2），不算残余物（src/errors.rs:47-56）
