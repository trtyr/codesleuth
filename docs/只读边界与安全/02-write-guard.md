# 零写入自证（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
零写入自证：任务前后对目标仓库做 per-file manifest 快照比对，证明 codesleuth 全程只读；凡差异（除 .codegraph/.codesleuth 等工具元数据豁免目录）一律如实归因为「不可归因变更」，写入审计并打印 stderr 摘要，给调用方可审计的零写入保证。

## 证据列表
1. 功能价值：证明 codesleuth 在目标仓内的可写面为空集，任何差异如实上报（不可归因变更），不冒领不瞒报
   - src/writeguard.rs:1-4（审计 #2）
2. 关键文件 src/writeguard.rs：snapshot 生成 BTreeMap<路径,sha256> manifest，diff 检测增/删/改三类变更
   - src/writeguard.rs:26-39（审计 #2）
   - src/writeguard.rs:117-142（审计 #2）
3. 调用链：cli.rs 任务前 snapshot 考前快照 → agent.run 任务运行 → 任务后再 snapshot + diff，写审计 write_check 行并输出 stderr 摘要
   - src/cli.rs:246-247（审计 #7）
   - src/cli.rs:369-389（审计 #7）
4. 交互：快照豁免 SKIP_DIRS（.git/.codegraph/.codesleuth/target 等），与 vector::store 的项目索引目录 .codesleuth、bootlock 锁文件共用同一豁免约定（D011 工具元数据）；errors 模块提供 REPO_NOT_READABLE
   - src/writeguard.rs:11-20（审计 #2）
   - src/vector/store.rs:305-307（审计 #12）
   - src/bootlock.rs:12-13（审计 #17）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=5 · duration=26444ms · tokens=55536

