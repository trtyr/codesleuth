# 微观 · 工具注册表（tool-registry）

> 置信度 high · 8 条发现 · 994672 tokens

ToolRegistry 链路整体质量高：注册/寻址/导出三方签名一致（各工具 description 声明的默认值与上限与 parameters、execute 中的 clamp 三方对质全部吻合），只读边界（Trait 无写方法 + Fence 双层校验 + writeguard 快照）成立。未发现 P0/P1 正确性缺陷。真实问题集中在三处：①tests/adversarial.rs 的只读允许清单 READ_ONLY_TOOLS 漏列 vector_search（及内置 submit_report/recall），使 D009「全部注册工具名 ⊆ 允许清单」这条产品命门断言对生产工具面（--vector 时含 vector_search）失效——防线声明与实际覆盖脱节；②RecallEngine 的 tombstone/needs_compact/compact/tombstone_count 一整套墓碑-压缩 API 生产零调用（仅测试引用），是写一半的残余功能；③若干 P3 级缺陷（GrepTool 对 fff 返回 file_index 的无检查索引可能 panic、vector_search 空库测试断言恒真为空转测试、审计轮转检查仅在 create 时一次性生效等）。最值得先修：补齐 READ_ONLY_TOOLS 让对抗断言重新覆盖真实工具面。

1. 【P2·三方对质失配】adversarial 断言①的只读允许清单 READ_ONLY_TOOLS 只有 9 个名字（read/find_files/grep/explore/callers/callees/impact/files），漏列生产工具面在 --vector 下注册的 vector_search（cli.rs:660 注册）；且 harness 内置 submit_report/recall 也不在清单。当前测试因自建 registry 未含 vector_search 而侥幸通过，但「工具面无写能力」这条产品命门断言已不再覆盖真实工具面——一旦未来注册任何新工具，该防线静默失守。（tests/adversarial.rs:20-29、src/cli.rs:660-662）
2. 【P3·API 误用疑似】GrepTool::execute 以 rel[m.file_index] 直接索引（rel 由 result.files 构建，m 来自 result.matches），对 fff-search 返回的 file_index 无边界检查；若第三方库返回越界索引将 panic 并击穿整个会话。上游防线只能依赖 fff 内部不变量，仓库内无法验证——验证方法：构造 file_index > files.len() 的 mock。（src/tools/fuzzy.rs:187-199）
3. 【死代码·写一半功能】RecallEngine 的墓碑/压缩机制（tombstone/tombstone_count/needs_compact/compact）生产零调用（callers 仅命中自身测试），cli.rs 装配为只读 Arc<RecallEngine> 后无任何写路径触发墓碑；needs_compact 零引用。整段是与主程序断连的孤儿 API，仅在测试簇内部互引。（src/vector/recall.rs:68-97、src/cli.rs:659-662）
4. 【P3·测试空转】vector_search 的 empty_index_reports_gracefully_without_panic 断言为 out.is_err() || out.is_ok()——恒真表达式，零检验力，属伪装成回归测试的空转。（src/tools/vector_search.rs:95-106）
5. 【P3·状态生命周期】审计轮转检查只在 Audit::create 打开前做一次；会话中途超 64MB 不会轮转（继续追加，无界增长），且 session_id 含纳秒时间戳（audit.rs:54-60）几乎不重复，轮转分支实际不可达——注释声称的「单代轮转保护」在生产中近乎死路径。（src/audit.rs:77-102、src/audit.rs:54-60）
6. 【P3·三方对质小失配】recall 内置工具 schema 声明 required:[from,to]，但 execute 中 from 缺省 0、to 缺省 from（harness.rs:344-345）——缺参不报 USER_INPUT 而是静默回读 [0,0] 一行，与 submit_report 对非法参数严格拒绝的风格不一致。（src/harness.rs:344-345、src/harness.rs:509-520）
7. 【已审干净项】ToolRegistry 核心（register/get/schemas/is_empty）与各工具 description↔parameters↔execute 三方对质全一致：read 默认200/上限2000/1-based offset、find_files 20/100、grep 50/200、vector_search 8/10，clamp 语义正确；边界代入（空 query 被 filter 拒、k=999 clamp 到 10、offset>total 诚实反馈、空文件/二进制/lossy UTF-8 分支）均读到正确处理代码；Fence 双层校验（词汇归一预检 + canonicalize 复检）与 symlink/绝对路径逃逸测试对质通过；harness 对 registry 的消费（schemas 每轮重导出、get 按名寻址、未知工具拒绝并列名）无缺陷。（src/tools/mod.rs:34-54、src/tools/read.rs:50-84、src/tools/fuzzy.rs:57-84）
8. 【P3·残余物】cli.rs 地形提示字符串中「不可用。」与「请用」之间有约 50 个连续空格的排版残留（疑似拼接/对齐残留）。（src/cli.rs:330-332）
