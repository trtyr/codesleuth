# 微观 · 向量嵌入（vector-embed）

> 置信度 high · 6 条发现 · 412742 tokens

向量嵌入链路（embed→compose/chunk→build→store→recall→vector_search→CLI）防御设计良好，但发现 2 个 P2 级真实缺陷：①RecallEngine::open 对 model/dim 失配只降级 HNSW、暴力路径照常跨模型/跨维度（zip 截断）检索，构建失败降级复用旧索引时静默产出无语义的召回；②embed_one 错误分支对含中文的 JSON 响应做字节级 200 截断可 panic。另有 2 个 P3（embedding 标量 null 被静默丢缺产出短维向量并落库；GC 键解析失败回退 line_start=0 方向性错误）。死代码：with_batch_size、remove_stale 包装、descriptions 表为确定死/残余；tombstone/compact 族为已登记预留入口。最值得先修：embed.rs:173 的 UTF-8 截断 panic（一行修复）。

1. P2-1：模型/维度失配静默跨模型检索——model_ok/dim_ok 只用于 HNSW 门槛，暴力路径用新客户端查询向量对旧模型/旧维度行向量做 zip 截断余弦；构建失败降级复用旧索引（cli.rs:637-643）即触发（src/vector/recall.rs:36-51、src/vector/store.rs:289-300、src/cli.rs:636-659）
2. P2-2：embed_one 错误分支对 serde_json 序列化文本做字节级 200 截断（&snippet[..min(200)]），多字节 UTF-8（中文错误体常见）截断即 panic（src/vector/embed.rs:166-177）
3. P3-1：embed_one filter_map 内层静默丢缺非数值标量（null），条数校验抓不住短维向量，upsert 按 vector.len() 落库 dim；疑似级，验证=构造含 null 的 data 单测（src/vector/embed.rs:182-199、src/vector/store.rs:175-196）
4. P3-2：remove_stale_on 键解析失败回退 line_start=0 执行删除（与 parts.len()!=3→continue 自相矛盾）；因 line_start 恒≥1 实际删不到行，但方向错误（src/vector/store.rs:113-139）
5. 死代码盘点：with_batch_size 零调用（bin crate 无外部 API）；remove_stale 包装仅测试调用（生产走 commit_build→remove_stale_on）；descriptions 表只删不写为残余；tombstone/needs_compact/compact 仅测试调用，文档登记为预留入口（非死代码）（src/vector/embed.rs:66-69、src/vector/store.rs:105-110、src/vector/store.rs:71-75）
6. 审过且干净的段：split_batches/validate_index_alignment 边界（空/1-based/跳号，含单测）；embed 去重槽位回填一致性（空输入/全重复/批失败传播）；chunk 切块行号钳制与切片区间；commit_build 单事务原子性；vector_search k clamp；run_index_vector 双路径锁（src/vector/embed.rs:226-264、src/vector/embed.rs:77-145、src/vector/chunk.rs:280-340）
