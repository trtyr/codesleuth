# 宏观 · overdesign

> 置信度 high · 9 条发现 · 656203 tokens

总评定级：**局部过度**。核心链路（分层配置、Harness 主循环、错误码段位、Tool trait、OutputFormat/契约旗标）的复杂度均能对应真实需求且有测试接线，不算过度设计成瘾。过度集中在两处：①向量召回层为「≥10 万块的巨型仓」投机构建了 HNSW 影子图 + 墓碑/压缩整套机制，而 MIN_HNSW_SIZE=100_000（用户拍板暖启动零负担）意味着现实所有仓库都走暴力路径，且 RecallEngine 以 Arc 不可变共享注入工具，四个 &mut self 的墓碑/压缩 API 生产上结构性不可达——这是仍在生长面上的投机通用化，应止血；②CLI 与存储层存在一批「写一半/被裁决移除后未清」的残余（--focus、--rebuild、descriptions 表、转发包装、with_batch_size、list_tools），属历史遗留，可低成本清理。删掉清单各项不会破坏任何生产行为，合计可减约 200+ 行与 HNSW 依赖的激活面。最快见效三项简化：删 HNSW 分支与墓碑族（recall.rs 缩为暴力+余弦约 60 行）；删 --focus/--rebuild 旗标并同步 README 示例；删 descriptions 表与其 GC 行。

1. 【域4·最重的过度设计】HNSW 影子图+三段式查询整套机制（HNSW_M/MAX_LAYER/EF 常量、hnsw_rs 依赖、OVERFETCH_FACTOR）只为 ≥MIN_HNSW_SIZE=100_000 块的巨型仓服务，而注释自述该阈值是 2026-10-05 用户硬要求「暖启动零负担」拍板——现实所有仓库全走暴力余弦路径，HNSW 分支近乎永久死路径；删除后 recall.rs 缩为「全量暴力+余弦」约 60 行，维护税（新人需理解三段式、墓碑、降级三层语义）大幅下降（src/vector/recall.rs:13-22、src/vector/recall.rs:55-97、src/tools/vector_search.rs:11-16）
2. 【域1·投机通用化】RecallEngine 的墓碑族 API（tombstone/tombstone_count/needs_compact/compact，均 &mut self）生产结构性不可达：工具层持 Arc<RecallEngine>（不可变共享），全仓无 mut 路径；GC 死块在下一次全量构建时由 remove_stale_on 在 SQLite 层处理，会话内墓碑机制是为「未来增量删除」预留而未来未到。删掉后连带 needs_compact/compact 测试一起清（src/vector/recall.rs:68-97、src/tools/vector_search.rs:11-16）
3. 【域2·配置通胀】CLI --focus 参数（可多次收集 glob）全链路零消费，README:113 还作为官方示例文档化——纯「写一半」的旗标负债，用户以为缩小了范围实际全仓扫描；同段的 --rebuild 示例（README:116）同样只剩提示文案作用，删掉或接线二选一（src/cli.rs:28-30、README.md:112-117）
4. 【域2】Index 子命令 --rebuild 旗标只剩文案作用：非 vector 路径统一走 index_structure_error(rebuild) 打印提示，无任何行为差异；注释自认「实际入口：run --fresh-index」。删掉旗标与该分支世界更简单（src/cli.rs:89-91、src/cli.rs:140-144）
5. 【域4·已移除功能的残余】descriptions 表：schema 建表 + GC 每次 commit_build 都执行永远 0 行的 DELETE（store.rs:134），全仓无任何 INSERT——描述层已裁决移除，留下只删不写的半截兼容层。删表+删 GC 行为无任何可观测变化（src/vector/store.rs:71-75、src/vector/store.rs:134-135）
6. 【域3·纯转发包装】VectorStore::remove_stale 与 upsert_chunk 是 _on 内核的一行转发包装，生产全走 commit_build 内的 _on 版本，包装仅测试调用——间接跳数+1 的仪式层（src/vector/store.rs:108-110、src/vector/store.rs:170-172）
7. 【域2·可定制但零定制】EmbedClient::with_batch_size 构建器全仓零调用，batch_size 构造期硬编码 64 后再无变更通道——为不存在的批大小调参需求预留（src/vector/embed.rs:62-69）
8. 【域4】McpClient::list_tools 初始化握手后零调用（工具集在 cli.rs 静态注册，从不向 server 查询）——协议完整性 API，删掉不影响任何行为；配套的「initialize 返回 usage guidance Phase 4 可用」注释同样是为未到的 Phase 4 预留（src/mcp.rs:104-126）
9. 【判为正当·非过度】LlmProvider trait 虽只有一个生产实现，但 tests/adversarial.rs 与 replay.rs 各持 Scripted 实现做确定性回放，抽象被测试真实二次使用；profiles 档位系统（D017）有 CLI --profile、逐层合并、未知名报错的完整接线与测试；OutputFormat 三态与 --require/--require-line 契约（OutputRequirement 双语义：harness.rs:730-752，两旗标分别经 with_required_markers/with_required_line_markers 消费且 describe/satisfied_by 均在 missing_markers→contract_error 链上使用）均有消费链，不算投机通用化（src/config.rs:207-258、src/harness.rs:728-761）
