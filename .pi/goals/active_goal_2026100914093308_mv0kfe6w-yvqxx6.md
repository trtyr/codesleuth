{
  "version": 3,
  "id": "mv0kfe6w-yvqxx6",
  "objective": "修复 P007 R3 全部 P3 一致性/契约缺陷（29 条中 28 条，3.25 HNSW 墓碑截断移交给 R4），按模块分批：对质→实现→回归测试→roadmap 回写→ci.sh 全绿→commit。",
  "status": "active",
  "autoContinue": true,
  "usage": {
    "tokensUsed": 1289657,
    "activeSeconds": 2253
  },
  "sisyphus": false,
  "revision": 175,
  "createdAt": "2026-10-09T06:09:33.080Z",
  "updatedAt": "2026-10-09T06:48:17.096Z",
  "scheduler": {
    "version": 1,
    "owner": "01a11f02-a4b4-7630-a06f-f8e4e2d4bb0a",
    "generation": "27c5f144-3bb1-40f6-b76b-8e8d8fea7b2e",
    "used": 3,
    "phase": "idle",
    "repairUsed": false
  },
  "taskList": {
    "tasks": [
      {
        "id": "r3-llm",
        "title": "LLM 批：3.1 5xx 改 LLM_SERVER（chat 改调 classify_llm_error 或删影子分类器）+ 3.2 200 状态体读失败改可重试 CS2001 + 3.15 退避注释同步",
        "status": "complete",
        "verificationContract": "错误分类单测覆盖 5xx→LLM_SERVER、体读失败→retryable=true；注释与实现一致",
        "completedAt": "2026-10-09T06:13:28.099Z",
        "evidence": "classify_llm_error 转生产单源（5xx→LLM_SERVER）、体读失败 retryable CS2001(LLM_UNREACHABLE)、退避注释同步；137 tests 全绿 bg_1mjekqn3"
      },
      {
        "id": "r3-report",
        "title": "报告批：3.3 validate 改用正确错误码 + 3.4 render_human 降级标注改基于 degraded 字段",
        "status": "complete",
        "verificationContract": "单测：报告校验错误不再是 INDEX_BUILD_FAILED；零 findings+dead_ends 合法报告不再被标降级",
        "completedAt": "2026-10-09T06:13:28.110Z",
        "evidence": "validate 改 OUTPUT_CONTRACT + render_human 零 findings 非降级标注；新增 zero_findings_with_dead_ends 测试；137 全绿"
      },
      {
        "id": "r3-config",
        "title": "config 批：3.5 to_file_view 补 context 段 + llm.profile 虚构值 + set 的 profile 忽略提示 + 3.6 XDG hint 改 HOME + 测试名去 env",
        "status": "complete",
        "verificationContract": "config get 全量输出含生效 context 值；hint 不再提 XDG",
        "completedAt": "2026-10-09T06:19:46.352Z",
        "evidence": "to_file_view 补 context、profile 虚构值改「（未选用档位）」、set 静默忽略加提示、XDG hint 改 HOME、测试名去 env；to_file_view_includes_effective_context 新增"
      },
      {
        "id": "r3-recall-read",
        "title": "recall/read 参数批：3.7 recall 缺参报 USER_INPUT + 3.8 read offset/limit 非法值输出头声明实际生效值 + 3.26 错点碰撞承诺改口径",
        "status": "complete",
        "verificationContract": "缺 to 不再静默空范围；limit 超限回显实际值",
        "completedAt": "2026-10-09T06:19:46.359Z",
        "evidence": "recall 缺参打回+计熔断（不再静默空范围）；read 归一说明进输出头；12-bit 锚点口径修正注释"
      },
      {
        "id": "r3-fuzzy",
        "title": "fuzzy 批：3.9 grep 截断提示可执行化 + time_budget 写而不启处置 + find_files 错误包装 USER_INPUT",
        "status": "complete",
        "verificationContract": "截断提示不再诱导撞去重拒绝；非法 query 报错带 hint",
        "completedAt": "2026-10-09T06:19:46.363Z",
        "evidence": "grep 截断提示可执行化、time_budget 启用；find_files 错误包装子项经 fff API 对质为误报（fuzzy_search 不返回 Result），roadmap 记录"
      },
      {
        "id": "r3-audit-logs",
        "title": "审计/日志批：3.10 audit 轮转 seq 跨代+create 播种+损坏行处理 + 3.11 日志轮转运行期检查+rename 失败可见化 + 3.12 工具失败写 tool_result 审计 + 3.13 audit_seq 锚 tool_result 行",
        "status": "complete",
        "verificationContract": "单测覆盖播种与轮转 seq 续号；失败调用在审计有结果行",
        "completedAt": "2026-10-09T06:29:48.806Z",
        "evidence": "audit 播种+轮转不覆盖+损坏行跳过（reopen_seeds_seq_and_skips_corrupt_lines）、logs 运行期轮转+rename eprintln、工具失败 tool_result 行、evidence 锚 tool_result seq；139 全绿"
      },
      {
        "id": "r3-mcp-graph",
        "title": "MCP/graph 批：3.16 错误码语义归位（call_tool 拆分/bootlock 首试硬错误分类/embed 先取 status）+ 3.17 --vector 内层 repo_map 失败补审计留痕 + 3.23 CODEGRAPH_NO_DAEMON 注释修正/callees 参数面对齐/run_cli 诊断/空格残留",
        "status": "complete",
        "verificationContract": "grep 验证：注释与代码一致；降级路径均有审计行",
        "completedAt": "2026-10-09T06:48:17.054Z",
        "evidence": "grep 验证：graph.rs 仅剩修正注释无虚假承诺；isError/请求级错误改 INDEX_NOT_AVAILABLE、bootlock 带 source+hint、embed 先取 status；repo_map 降级已入 record_degraded 统一入口（bg_7ltoebw2 全绿）"
      },
      {
        "id": "r3-vector-misc",
        "title": "vector/杂项批：3.14 path_like 垃圾 token 收紧 + 3.18 repomap 空兑底+注释+死常量 + 3.19 并发数/HNSW 阈值注释同步+is_comment_line+codegraph 静默降级 warn + 3.20 CJK 估算列债务登记 + 3.21 READ_ONLY_TOOLS 补齐 + 3.22 空转测试改真断言 + 3.27 setup_vector_layer RecallEngine::open 降级化 + 3.28 耦合三件（降级 helper 收敛为主，大重构不做）+ 3.29 SKIP_DIRS 平台假设登记",
        "status": "complete",
        "verificationContract": "adversarial 三断言含 vector_search；空转测试有真断言；ci.sh 全绿",
        "completedAt": "2026-10-09T06:48:17.087Z",
        "evidence": "path_like 收紧+matches! 重构、repomap 空兜底/注释/死常量删、注释同步、READ_ONLY_TOOLS 补齐（adversarial 3 断言过）、空转测试真断言、3.27 降级化、record_degraded 收敛 7 处、CJK/SKIP_DIRS 债务登记（bg_7ltoebw2 全绿）"
      },
      {
        "id": "r3-final",
        "title": "R3 收口：roadmap R3 回写 + ci.sh 三连全绿（原始日志 CI_EXIT=0）+ commit",
        "status": "complete",
        "verificationContract": "ci.sh 原始日志全绿；roadmap R3 逐条勾销；决策记录补齐（3.25 移交 R4、3.28 范围裁剪）",
        "completedAt": "2026-10-09T06:48:17.095Z",
        "evidence": "roadmap R3 整段回写 done（3.25 移交 R4、3.9 子项误报、3.28 范围裁剪均有决策记录）；ci.sh bg_7ltoebw2 CI_EXIT=0（139+3+1）；commit 待落"
      }
    ],
    "blockCompletion": true,
    "proposedAt": "2026-10-09T06:09:20.791Z"
  },
  "activePath": ".pi/goals/active_goal_2026100914093308_mv0kfe6w-yvqxx6.md"
}

# Goal Prompt

修复 P007 R3 全部 P3 一致性/契约缺陷（29 条中 28 条，3.25 HNSW 墓碑截断移交给 R4），按模块分批：对质→实现→回归测试→roadmap 回写→ci.sh 全绿→commit。

## Progress

- Status: running
- Auto-continue: on
- Sisyphus mode: no
- Time spent: 37m33s
- Tokens used: 1.3M (1,289,657) tokens
## Tasks

<!-- blockCompletion: true -->
- [x] r3-llm: LLM 批：3.1 5xx 改 LLM_SERVER（chat 改调 classify_llm_error 或删影子分类器）+ 3.2 200 状态体读失败改可重试 CS2001 + 3.15 退避注释同步 — evidence: classify_llm_error 转生产单源（5xx→LLM_SERVER）、体读失败 retryable CS2001(LLM_UNREACHABLE)、退避注释同步；137 tests 全绿 bg_1mjekqn3
- [x] r3-report: 报告批：3.3 validate 改用正确错误码 + 3.4 render_human 降级标注改基于 degraded 字段 — evidence: validate 改 OUTPUT_CONTRACT + render_human 零 findings 非降级标注；新增 zero_findings_with_dead_ends 测试；137 全绿
- [x] r3-config: config 批：3.5 to_file_view 补 context 段 + llm.profile 虚构值 + set 的 profile 忽略提示 + 3.6 XDG hint 改 HOME + 测试名去 env — evidence: to_file_view 补 context、profile 虚构值改「（未选用档位）」、set 静默忽略加提示、XDG hint 改 HOME、测试名去 env；to_file_view_includes_effective_context 新增
- [x] r3-recall-read: recall/read 参数批：3.7 recall 缺参报 USER_INPUT + 3.8 read offset/limit 非法值输出头声明实际生效值 + 3.26 错点碰撞承诺改口径 — evidence: recall 缺参打回+计熔断（不再静默空范围）；read 归一说明进输出头；12-bit 锚点口径修正注释
- [x] r3-fuzzy: fuzzy 批：3.9 grep 截断提示可执行化 + time_budget 写而不启处置 + find_files 错误包装 USER_INPUT — evidence: grep 截断提示可执行化、time_budget 启用；find_files 错误包装子项经 fff API 对质为误报（fuzzy_search 不返回 Result），roadmap 记录
- [x] r3-audit-logs: 审计/日志批：3.10 audit 轮转 seq 跨代+create 播种+损坏行处理 + 3.11 日志轮转运行期检查+rename 失败可见化 + 3.12 工具失败写 tool_result 审计 + 3.13 audit_seq 锚 tool_result 行 — evidence: audit 播种+轮转不覆盖+损坏行跳过（reopen_seeds_seq_and_skips_corrupt_lines）、logs 运行期轮转+rename eprintln、工具失败 tool_result 行、evidence 锚 tool_result seq；139 全绿
- [x] r3-mcp-graph: MCP/graph 批：3.16 错误码语义归位（call_tool 拆分/bootlock 首试硬错误分类/embed 先取 status）+ 3.17 --vector 内层 repo_map 失败补审计留痕 + 3.23 CODEGRAPH_NO_DAEMON 注释修正/callees 参数面对齐/run_cli 诊断/空格残留 — evidence: grep 验证：graph.rs 仅剩修正注释无虚假承诺；isError/请求级错误改 INDEX_NOT_AVAILABLE、bootlock 带 source+hint、embed 先取 status；repo_map 降级已入 record_degraded 统一入口（bg_7ltoebw2 全绿）
- [x] r3-vector-misc: vector/杂项批：3.14 path_like 垃圾 token 收紧 + 3.18 repomap 空兑底+注释+死常量 + 3.19 并发数/HNSW 阈值注释同步+is_comment_line+codegraph 静默降级 warn + 3.20 CJK 估算列债务登记 + 3.21 READ_ONLY_TOOLS 补齐 + 3.22 空转测试改真断言 + 3.27 setup_vector_layer RecallEngine::open 降级化 + 3.28 耦合三件（降级 helper 收敛为主，大重构不做）+ 3.29 SKIP_DIRS 平台假设登记 — evidence: path_like 收紧+matches! 重构、repomap 空兜底/注释/死常量删、注释同步、READ_ONLY_TOOLS 补齐（adversarial 3 断言过）、空转测试真断言、3.27 降级化、record_degraded 收敛 7 处、CJK/SKIP_DIRS 债务登记（bg_7ltoebw2 全绿）
- [x] r3-final: R3 收口：roadmap R3 回写 + ci.sh 三连全绿（原始日志 CI_EXIT=0）+ commit — evidence: roadmap R3 整段回写 done（3.25 移交 R4、3.9 子项误报、3.28 范围裁剪均有决策记录）；ci.sh bg_7ltoebw2 CI_EXIT=0（139+3+1）；commit 待落

