# repo map 导航图（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
repo map 导航图：--repo-map 开启时，从 codegraph SQLite 读符号与边表度数，按种子→邻居→度数降序在默认 24000 字符预算内贪心生成结构图，经 wrap_repo_section 包成 [repo map] 段注入首条用户消息。有召回命中时生成任务导航图（build_task_map），无召回则全局图（build_repo_map），失败均非致命降级并留审计痕。

## 证据列表
1. 功能定位：codegraph 符号+边表度数生成预算化（默认 24000 字符）仓库导航图，注入 system prompt 尾部 [repo map] 段
   - src/vector/repomap.rs:1-12（审计 #2）
2. 两个生成器：build_task_map（召回种子优先→一跳邻居→度数降序，◈/◇ 标记）与 build_repo_map（纯度数降序），预算满即截断
   - src/vector/repomap.rs:67-135（审计 #2）
   - src/vector/repomap.rs:137-183（审计 #2）
3. 数据来源：repo_map_inputs 只读打开 codegraph SQLite，查 nodes（排除容器型节点）与 edges 累计度数
   - src/vector/repomap.rs:23-64（审计 #2）
4. 入口链：setup_vector_layer 先 recall 取命中，经 chunk::relations_for_symbol 取邻居，调 build_task_map 并 wrap_repo_section 组后缀
   - src/cli.rs:532-659（审计 #7）
5. 兜底路径：仅 --repo-map 无向量时直接 repo_map_inputs→build_repo_map 生成全局图
   - src/cli.rs:333-351（审计 #7）
6. 下游消费与降级：后缀经 Harness::with_first_user_suffix 注入首条消息；向量层失败非致命降级并记审计 degraded
   - src/cli.rs:361-365（审计 #7）
   - src/cli.rs:315-326（审计 #7）
7. 预算可配：vector.repomap_budget 默认 24_000
   - src/config.rs:119-126（审计 #21）

## 死胡同
无

## 置信度
high

## 统计
turns=8 · tool_calls=7 · duration=36026ms · tokens=71599

