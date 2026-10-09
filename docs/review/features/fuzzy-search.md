# 微观 · 模糊搜索工具（fuzzy-search）

> 置信度 high · 9 条发现 · 457315 tokens

模糊搜索链路（find_files/grep → FuzzyEngine → ToolRegistry → cli.rs 装配 → harness 分发）整体质量较好：参数判空、limit clamp、调用去重、围栏定界均到位；但存在 1 个 P2 与数个 P3 契约性缺陷，集中在 src/tools/fuzzy.rs。最值得先修：grep 的 mode 枚举未校验（schema 承诺 enum 但实现对任意非法值静默降级为 PlainText，LLM 传 "RegExp" 等会以错语义搜索且零错误反馈）。另有 CLI 层一处确定死代码：--focus 旗标收集后零消费。链路清单中 vector/*、audit、report、llm、logs、context 等为 codegraph 闭包漂移混入，与该功能点无调用关系，仅接口面核对后跳过内部。

1. P2：grep 的 mode 参数 schema 声明 enum [plain|regex|fuzzy]，但实现对任何非法值（如 "RegExp"）静默按 PlainText 处理，LLM 传错时以纯文本搜索正则串、几乎必零命中且无错误反馈；上游防线不成立——harness 只校验参数是合法 JSON，不按 schema 校验枚举。（src/tools/fuzzy.rs:139、src/tools/fuzzy.rs:152-157、src/harness.rs:382-394）
2. P3：grep 的 file_offset 硬编码为 0 且 parameters() 未暴露该参数，但截断提示「还有更多文件未扫（file_offset=N）」给出调用方无法执行的续读指令（对比 find_files 的「可提高 limit」可执行），诱导 LLM 原样重调并撞 harness 去重拒绝。（src/tools/fuzzy.rs:170、src/tools/fuzzy.rs:200-204、src/tools/fuzzy.rs:133-143）
3. P3 疑似：find_files 的 query 经 QueryParser 解析后直接 fuzzy_search，对「全为否定约束（如 !node_modules）无正向项」等会从 fff 返回错误的 query，错误未按链路惯例包装为 USER_INPUT+hint（对比缺 query 参数时的规整处理）。（src/tools/fuzzy.rs:86-92、src/tools/fuzzy.rs:74-79）
4. P3 疑似：grep 设置 time_budget_ms=5000 但 enforce_time_budget=false，预算写而不启，模块注释承诺的降级前提未落实；且同步阻塞的 picker.grep/fuzzy_search 调用直接在 async fn 内执行（无 spawn_blocking），大仓库上无界占用 tokio worker。（src/tools/fuzzy.rs:173-174、src/tools/fuzzy.rs:112、src/tools/fuzzy.rs:181）
5. 确定死代码：CLI 的 --focus 旗标（pub focus: Vec<String>）经 clap 注册收集用户输入，但全仓 grep self.focus/.focus 0 命中、run_task_inner 全程未读——收集后零消费，属「写一半的功能」（FuzzySearchOptions/GrepSearchOptions 均未接 glob 过滤），建议移除或落实。（src/cli.rs:28-30、src/cli.rs:228-514）
6. 干净段：ToolRegistry get/schemas 与 harness 分发契约一致（未知工具拒绝并计入 no_progress）；调用去重（同工具+同参）先于执行，重复调用计入打转计数——与 prompt.rs「禁止重复调用」纪律闭环。（src/tools/mod.rs:38-53、src/harness.rs:395-411、src/harness.rs:364-379）
7. 干净段：Fence resolve 两道关卡（词汇层归一预检 + canonicalize 复检）实现正确；FuzzyEngine::new 仅取 fence.root() 作 FilePicker base_path 属安全用法（find_files/grep 参数面不含路径，无需走 resolve 守门；read 工具单独持 Arc<Fence> 在 read.rs:86 守门）。（src/fence.rs:30-55、src/tools/fuzzy.rs:23-37、src/cli.rs:279-284）
8. 干净段：find_files/grep 的 limit clamp(1,100)/(1,200) 边界（0/1/上限）代入正确；grep 输出 rel[m.file_index] 索引映射与 fff 契约（files 与 matches 同批返回）一致。（src/tools/fuzzy.rs:80-84、src/tools/fuzzy.rs:158-162、src/tools/fuzzy.rs:187-198）
9. 三方对质补充：replay 契约锁定 [find_files, grep, read, submit_report] 序列；adversarial 只读允许清单包含 find_files/grep 且 schema 无写语义——两测试与实现一致。（tests/replay.rs:91-101、tests/adversarial.rs:20-29、tests/adversarial.rs:116-139）
