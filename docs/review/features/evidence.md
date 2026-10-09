# 微观 · 证据库（evidence）

> 置信度 high · 10 条发现 · 402956 tokens

证据库链路（evidence.rs → harness.rs 写入/校验 → read/fence/audit 接口）发现 6 项缺陷：P1×1（observe 对所有工具输出 indiscriminate 入库，模型凭 grep/find_files 输出中出现的路径即可通过「真实读过」evidence 校验，诚实性契约被结构性绕过）、P2×3（read 的二进制/空文件/offset 越界三种「诚实声明」Ok 输出也算存证成功；audit_seq 锚到 tool_call 行而非 tool_result 行导致 recall 互查半盲；append_line 与 record 取号机制不互锁的疑似并发撞号）、P3×2（path_like 垃圾 token 污染 observed_paths 与打回提示并可充当 evidence.file；Evidence.lines 完全不校验只锚文件级）。最优先修复是把 evidence 入库收敛到「真实读到内容」的通道。死代码维度：确定死 0 项，疑似 1 项（fence.rs:35 不可达的 current_dir 分支），最值得清理该项。链路最干净的一段是 evidence.rs 本体的复制粘贴对（info_keys vs observe 切词逻辑）与 read.rs 分页边界。

1. B1(P1): harness 对所有工具 Ok 输出 indiscriminately observe 入库；grep/find_files 等输出中出现的路径 token 进入 EvidenceStore 后 cite_seq 判 Some 即放行，模型从未 read 过的文件可通过 submit_report 的「真实读过」evidence 校验（工具 schema 文案 harness.rs:491 明写『必须引用本会话真实读过的文件』），防线被结构性绕过（src/harness.rs:426-436、src/harness.rs:567-579、src/evidence.rs:44-51）
2. B2(P2): read 工具的三种「诚实声明」Ok 返回（二进制不倾倒/空文件/offset 越界）均未读出内容，但 harness 仍对它们 observe_exact(path) 存证，模型可用越界 read 把未见过内容的文件变成合法 evidence（src/tools/read.rs:96-118、src/harness.rs:437-441）
3. B3(P2): evidence.audit_seq 存的是 tool_call 行的 seq（只含 args），recall 该锚点回读不到被引文件内容，报告↔审计互查锚点半盲（src/harness.rs:414-417、src/audit.rs:114-133、src/report.rs:10-16）
4. B4(P3): path_like 启发式把 3.14/1.2.3 等垃圾 token 入库，observe 切词产生含空格路径的碎片（测试自证），污染 observed_paths 计数、零 findings 打回提示，且垃圾 token 可被 cite_seq 命中充当 evidence.file（src/evidence.rs:22-24、src/harness.rs:616-627、src/harness.rs:1456-1461）
5. B5(P3): Evidence.lines 是任意字符串且从不对照实际读取范围，锚点防张冠李戴能力止于文件级（src/harness.rs:559-563、src/report.rs:12-15）
6. B6(P3 疑似): append_line 用「读全文取 max+1」取号与 Audit::record 锁内 fetch_add 不互锁，并发时 seq 撞号破坏 JSONL 单调不变量（当前调用面为会话后 write_check，窗口小）（src/audit.rs:21-29、src/audit.rs:144-147）
7. 干净面：evidence.rs 内 info_keys 与 observe 的复制粘贴切词逻辑逐 token 一致无漏改；or_insert 首次语义两入口一致；build_report 的 uncited/零 findings/bare finding 三道打回分支覆盖完整（src/evidence.rs:8-24、src/evidence.rs:35-55、src/harness.rs:601-643）
8. 干净面：read.rs 分页边界（offset-1 skip、end-offset+1 take、空文件/越界/截断续读 offset）代入 0/1/超界心算无误（src/tools/read.rs:108-145）
9. 疑似不可达代码：fence.resolve 的 current_dir 分支不可达（root 已 canonicalize 为绝对，join 结果恒绝对）；这也是死代码维度唯一发现，确定死代码为 0（src/fence.rs:30-56、src/fence.rs:12-21）
10. 接口面核对：CLI 装配（fence Arc 共享 ReadTool/FuzzyEngine 顺序）、harness 消费 ChatResponse（resp.text Option 以 unwrap_or_default 消费）、mcp call_tool 返回 String 契约均一致，无接口面缺陷（src/cli.rs:279-284、src/harness.rs:264-320）
