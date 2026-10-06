# writeguard（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
codesleuth 的 writeguard 模块提供「零写入自证」能力：任务前对目标仓库做 per-file 指纹快照，任务后再快照一次并 diff，codesleuth 工具面未触及的差异作为「不可归因变更」如实上报审计行，不冒领不瞒报；`.codegraph`/`.codesleuth`/构建产物等目录在快照期被 SKIP_DIRS 豁免。

## 证据列表
1. 模块提供 snapshot(root) 与 diff(before, after) 两入口；Manifest = BTreeMap<relpath, sha256>，Change { path, kind: Added|Removed|Modified }。
   - src/writeguard.rs:27-39（审计 #2）
   - src/writeguard.rs:104-142（审计 #2）
2. walk 递归遍历根目录：跳过 symlink（防成环/出仓）、跳过 SKIP_DIRS（.git/.codegraph/.codesleuth/target/node_modules/dist/__pycache__/.venv）；>1MB 文件用「大小+首 64KB」指纹而非全量哈希，避免资产仓快照分钟级阻塞。
   - src/writeguard.rs:11-24（审计 #2）
   - src/writeguard.rs:41-102（审计 #2）
3. 不可归因变更如实上报：cli.rs 在任务起止两次 snapshot，diff 后经 audit::append_line 写入 kind=write_check 行，包含 files_snapshotted / unattributed_changes / samples（前 5 条示例）。
   - src/cli.rs:252-253（审计 #9）
   - src/cli.rs:375-393（审计 #9）
4. 模块对外暴露为 pub mod writeguard；与 vector/chunk.rs、bootlock.rs 通过「.codesleuth/.codegraph 是工具元数据、写面豁免」约定对齐（D011 豁免面）。
   - src/lib.rs:22（审计 #29）
   - src/vector/chunk.rs:34-43（审计 #31）
   - src/bootlock.rs:13（审计 #33）
5. 测试覆盖：无变化空 diff、改/增/删三类 Change 检测、symlink 不入 manifest、.codegraph 豁免面变更归零、write_check 审计行 seq 单调不重。
   - src/writeguard.rs:144-256（审计 #2）

## 死胡同
- callers("writeguard") 未返回直接反向边；改用 grep "writeguard" 在 src/cli.rs、src/lib.rs、src/vector/store.rs、src/vector/chunk.rs、src/bootlock.rs 中定位到模块挂载与豁免约定引用。

## 置信度
high

## 统计
turns=9 · tool_calls=12 · duration=46222ms · tokens=96724

