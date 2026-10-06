# 向量索引子命令（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量索引子命令由 CLI 的 `Index --vector` 旗标触发：`Cli::run` 分发到 `run_index_vector`（src/cli.rs:109-115, 132-182），后者加载配置、解析嵌入端点、获取仓库级引导锁，并 `block_on` 调用 `vector::build_vector_index`（src/vector/build.rs:24-105）完成 chunk 规划、复用/失效判定、网络嵌入、原子提交与 GC，最终通过 `eprintln!` 打印 `BuildReport` 统计。

## 证据列表
1. Command::Index 子命令定义包含 `--vector` 旗标，是该功能的 CLI 入口定义。
   - src/cli.rs:67-77（审计 #2）
2. Cli::run 在 Command::Index 分支上当 `vector: true` 时调用 `Self::run_index_vector`，否则走 index_structure_error 报错路径。
   - src/cli.rs:109-124（审计 #2）
3. run_index_vector 负责加载配置、解析嵌入端点、构造 EmbedClient、获取 bootlock 引导锁，并以 tokio runtime block_on 驱动 vector::build_vector_index，最后 eprintln! 输出 BuildReport。
   - src/cli.rs:132-182（审计 #2）
4. build_vector_index 是核心实现：基于 codegraph.db 拉符号、规划 chunk、依据 model/dim/mode 判定 stale、按 text_hash 复用、通过 EmbedClient 嵌入，最后由 VectorStore::commit_build 在单事务内完成 GC 删除 + upsert + meta 写入。
   - src/vector/build.rs:24-105（审计 #4）

## 死胡同
- grep "run_index_vector|Index\s*\{|Command::Index" 命中了 .pi/goals/ 下的目标账本 JSON 文件，与代码无关，未采用
- 未深入展开 setup_vector_layer / index_structure_hint_is_honest 等次要符号，超出概览级侦察范围

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=23387ms · tokens=28101

