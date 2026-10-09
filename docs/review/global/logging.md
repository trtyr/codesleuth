# 宏观 · logging

> 置信度 high · 9 条发现 · 255121 tokens

总评定级：部分覆盖（有统一设施但覆盖有明显盲区）。该项目拥有一个真正统一的日志设施：tracing + tracing-subscriber 单点初始化（lib.rs:27 init_tracing），stderr 人读轨 + ~/.codesleuth/logs/<session_id>.log 归档轨双轨同过滤同格式，session span 携带 session_id 结构化串线（有测试锁定），stdout 在 --json 模式下严格只放报告。日志与错误系统是「两条平行但有意设计的线」：错误走 CsError/report_error 的 stderr 三行结构化输出，不经 tracing——保证 stdout/stderr 纪律，但意味着终局错误不进归档日志文件（除非同位置另有 tracing::error!，如 llm.rs:279 有、多数 report_error 消费点没有）。主要短板：①覆盖稀疏——全仓生产代码仅 14 文件约 35 处 tracing 调用，MCP 客户端全文件零日志（spawn/超时/Drop 回收全静默），harness 工具执行失败分支、writeguard 考前快照失败等均无日志；②默认级别 warn 使归档轨近乎空载，info 级的配置就绪/嵌入进度默认不落盘；③64MB 轮转只在 open 时检查一次且失败被静默吞掉，运行期无界；④存在少量绕过门面的裸 eprintln!（多为有意的人读进度通道）；⑤敏感信息面干净——api_key 从不进日志或错误消息。级别纪律总体良好（可重试 warn、终局 error、进度 info、细节 debug），仅 llm.rs:298 一处终局错误用 warn 与同函数其他终局的 error 错位。

1. 域1·统一设施成立：init_tracing 单点初始化 registry+EnvFilter+stderr/文件双 fmt layer，全仓唯一日志库 tracing，无第二套框架；错误侧另有 report_error 专用 stderr 通道，属有意分流（src/lib.rs:27-66、src/logs.rs:13-41）
2. 域4·结构化串线有保障：session span 注入 session_id 且有测试断言 session_id="abc123" 落入日志行；cli.rs:257-263 配置就绪日志用键值参数化（repo/model/base_url/profile）而非字符串插值（src/logs.rs:105-145、src/cli.rs:255-263）
3. 域2·级别纪律总体一致：可重试失败→warn、终局→error、进度→info；一处错位：llm.rs:298 网络错误终局用 warn 而同函数其余终局用 error（src/llm.rs:274-299、src/vector/build.rs:69-78）
4. 域6·级别治理：verbose 0/1/2→warn/info/debug，EnvFilter 限 codesleuth 目标；默认 warn 使归档轨几乎只收 warn/error，info 级配置就绪/嵌入进度默认不落盘，与「留作归档」目的存在张力；无运行期/按模块动态调整（src/lib.rs:31-37）
5. 域6·轮转缺陷：64MB 阈值仅 open 时检查一次，运行期无大小控制；轮转失败 let _ = 静默吞掉且发生在 tracing 初始化前，注释声称的『warn 由调用方日志可见』不成立（src/logs.rs:22-29）
6. 域3·覆盖盲区：mcp.rs 全文件零 tracing 调用——spawn 失败仅转 CsError、90s 超时（request 循环）、Drop 回收全部无日志留痕；线上排查这些路径只能靠错误码反推（src/mcp.rs:68-102、src/mcp.rs:152-199）
7. 域1·裸输出盘点：生产代码绕过门面的 eprintln! 共 8 处——cli.rs:194/205（向量索引提示与 BuildReport 摘要）、443-459（零写入自证人读输出）、507（收尾提示）；属有意的进度/结果通道而非日志，但 cli.rs:205 混入警告形文案；stdout println! 仅限报告输出三模式，符合 stdout 纪律（lib.rs:26）（src/cli.rs:503-511、src/cli.rs:194-205、src/cli.rs:443-459）
8. 域6·错误↔日志衔接缺口：CsError 终局走 report_error 三行 eprintln 到 stderr，不经 tracing——错误不进归档日志文件；llm.rs:279/285 在返回前另有 tracing::error! 留档，但这是各处自发行为而非机制保证（src/errors.rs:108-117、src/llm.rs:279-286）
9. 域5·敏感信息面干净：api_key 仅用于 bearer_auth 构造（llm.rs:253），全仓日志与错误消息不含密钥；配置日志只打 base_url/model/profile；LLM 错误 snippet 为网关响应体截 200 字符非请求体；未发现密钥/token 落日志（src/cli.rs:255-263、src/llm.rs:253-265）
