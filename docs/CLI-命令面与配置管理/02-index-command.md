# 向量索引构建命令（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
`index <path> --vector` 命令为指定目标仓库手动预构建向量语义索引：基于 codegraph 结构库切分符号 chunk，调用嵌入服务生成向量并增量落库（复用未变块、GC 清理死块），供运行时语义检索复用。

## 证据列表
1. 命令入口 run_index_vector：规范化仓库路径、加载配置并解析嵌入端点（独立 [vector] 供应商，缺省跟随 [llm]），最后打印构建报告。
   - src/cli.rs:128-178（审计 #2）
2. 入口在 tokio runtime 上调用 vector::build_vector_index 完成实际构建，且与运行时引导共用 bootlock 引导锁（竞争败者退出）。
   - src/cli.rs:155-168（审计 #2）
3. 构建核心：从 .codegraph/codegraph.db 取符号并 plan_chunks，校验模型/维度/模式变更导致旧向量作废，仅嵌入变更块，GC 删除失效块后 commit_build 单事务原子落库。
   - src/vector/build.rs:25-105（审计 #4）
4. 索引存储于仓库内 .codesleuth/indexes/<fingerprint>/vectors.db，由 project_index_dir/index_path 决定路径。
   - src/vector/store.rs:303-312（审计 #7）

## 死胡同
- 未确认 CLI 层 index 子命令 clap 定义的具体行（未读 run_task/clap 解析段）

## 置信度
high

## 统计
turns=3 · tool_calls=3 · duration=11094ms · tokens=12541
