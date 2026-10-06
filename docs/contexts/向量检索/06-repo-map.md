# repo map预算注入（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
repo map 预算注入：从 codegraph SQLite 读符号表与边表度数，按默认 24_000 字符预算做度数中心度贪心装填，生成结构化导航文本并以 `[repo map]` 段包裹，作为首条 User 消息后缀一次性注入（harness.rs:83-88），让 agent 进门即可参照图结构、避免对图工具的盲查。实际实现按 D005 分层（harness.rs:67-68）走 user 消息而非 system——这是与 repomap.rs:1-4 头部注释/任务书措辞的差异点，以代码为准。

## 证据列表
1. 默认字符预算为 24_000（≈6k token），并为单符号行设置 200 字符上限以防长名吃光预算。
   - src/vector/repomap.rs:9-10（审计 #2）
   - src/vector/repomap.rs:11-12（审计 #2）
2. repo_map_inputs 从 codegraph SQLite 只读打开，SELECT 限定 kind 为 function/method/struct/class/interface/impl/trait 七类节点，并从 edges 表累加 source/target 两侧度数。
   - src/vector/repomap.rs:22-65（审计 #2）
3. build_task_map 与 build_repo_map 都是预算内贪心装填；build_repo_map 排序键为「度数降序 → 名称升序 → 文件路径升序」，build_task_map 优先召回命中种子、再邻居、最后按度数填充。
   - src/vector/repomap.rs:137-178（审计 #2）
   - src/vector/repomap.rs:67-135（审计 #2）
   - src/vector/repomap.rs:200-229（审计 #2）
4. wrap_repo_section 把地图包成 "[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。" 段，固定前缀与查证后缀由 unit test 验证。
   - src/vector/repomap.rs:180-183（审计 #2）
   - src/vector/repomap.rs:290-294（审计 #2）
5. 单元测试覆盖了三个核心不变量：度数降序排序、预算封顶后追加「其余未列入」溢出提示、小仓全图无溢出提示。
   - src/vector/repomap.rs:246-262（审计 #2）
   - src/vector/repomap.rs:264-279（审计 #2）
   - src/vector/repomap.rs:281-287（审计 #2）
6. 调用方在 src/cli.rs 的两条路径使用 repomap：向量层任务相关路径（setup_vector_layer 内部，625-653 行）走 build_task_map + wrap_repo_section；--repo-map 无向量路径（340-346 行）走 build_repo_map + wrap_repo_section。
   - src/cli.rs:340-346（审计 #15）
   - src/cli.rs:619-658（审计 #15）
   - src/cli.rs:540-545（审计 #15）
7. 主流程在 src/cli.rs:306 进入 setup_vector_layer，最终通过 src/cli.rs:368 的 with_first_user_suffix 把段交给 Harness。
   - src/cli.rs:304-314（审计 #15）
   - src/cli.rs:367-371（审计 #15）
8. 实际注入位置是首条 User 消息后缀而非 system：harness.rs:74-88 在 messages 构造完后取 last_mut 把 "\n\n" + suffix 追加到 User 消息 content；system 消息保持 crate::prompt::SYSTEM_PROMPT 恒定。
   - src/harness.rs:67-72（审计 #39）
   - src/harness.rs:74-88（审计 #39）
   - src/harness.rs:42-43（审计 #39）
9. 预算由 config.vector.repomap_budget 提供，默认 24_000，可被配置文件（FileConfig merge）覆写，并暴露在 config key 表中支持 CLI get/set；README 文档值也是 24000。
   - src/config.rs:99（审计 #51）
   - src/config.rs:137（审计 #51）
   - src/config.rs:274-275（审计 #51）
   - src/config.rs:437-444（审计 #51）
   - README.md:70（审计 #59）
10. 任务路径下 seeds 由召回 hits 转 symbols 组成，neighbors 由 vector::chunk::relations_for_symbol 返回的 callers/callees 合并去重得到（fallback/leftover 命中跳过）。
   - src/cli.rs:627-646（审计 #15）
11. 导航图构建失败走非致命降级：tracing::warn 记录后向 audit 写 "degraded" 事件并跳过注入，不阻断主流程。
   - src/cli.rs:348-356（审计 #15）
   - src/cli.rs:655-657（审计 #15）
12. 无符号索引的诚实工具面：cli.rs:282-285 在 codegraph 无符号时不注册图工具并把 first_suffix 留给地形提示，避免 agent 反复空查询图工具。
   - src/cli.rs:279-294（审计 #15）

## 死胡同
- grep 模式 "build_repo_map|build_task_map|repo_map_inputs|DEFAULT_BUDGET_CHARS" 用 plain 模式命中 0 条，换 regex 才命中——plain 模式的 | 不是逻辑或
- 未直接读到 build_task_map 在 setup_vector_layer 中被调用的完整 cfg/registry 装载上下文，但调用串联已通过 src/cli.rs:625-653 read 直接证实
- src/vector/chunk.rs 中 relations_for_symbol 的实现未读，凭 src/cli.rs:638 的调用点推断其返回 (callers, callees)

## 置信度
high

## 统计
turns=13 · tool_calls=23 · duration=74554ms · tokens=158678

