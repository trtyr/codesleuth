# 微观 · 向量索引子命令（index-cmd）

> 置信度 high · 7 条发现 · 414079 tokens

`index --vector` 链路核心事务与复用逻辑干净，但存在 2 个 P2 真实 panic 缺陷：plan_chunks 在 codegraph 记录符号但文件已清空时数组切片越界 panic（chunk.rs:285-307，上游 unwrap_or_default 只兜 db 缺失不兜陈旧）；embed 错误消息按字节截断 &snippet[..200] 可切在 UTF-8 多字节字符中间 panic（embed.rs:171-174）。另有 3 个 P3：OpponentFinished 复用不校验 meta 口径（cli.rs:187-196）、GC 键异常处理不一致（store.rs:113-127）、is_comment_line 对 '*' 开头代码行误判（chunk.rs:247-255）。死代码维：with_batch_size 确定死（零调用），remove_stale 包装与 tombstone 家族为预留/测试专用，descriptions 表为已移除描述层的 schema 残留。最值得先修：B1 空文件切片 panic。

1. P2：plan_chunks 对空文件+陈旧 codegraph 符号行会 panic——read_lines 空文件返回 Some(vec![])，start=max(1).min(1)=1、end=min(0).max(1)=1，lines[0..1] 越界；fallback 分支已修同类越界（418 行注释）而符号分支漏改；上游 build.rs unwrap_or_default 只兜 db 缺失不兜 db 陈旧（src/vector/chunk.rs:272-307、src/vector/chunk.rs:168-175、src/vector/chunk.rs:418-419）
2. P2：embed_one 错误消息 &snippet[..snippet.len().min(200)] 字节切片可切在多字节 UTF-8 字符中间 panic，且发生在错误路径上，embed() 的一次重试只覆盖 Err 不覆盖 panic，进程直接崩溃（src/vector/embed.rs:163-176、src/vector/embed.rs:103-114）
3. P3：run_index_vector 的 OpponentFinished 复用只检查 index_path 存在，不校验 model/dim/mode meta，配置漂移时静默复用异口径索引；对比正常路径 build.rs 有三键失效判定，语义不对称（src/cli.rs:187-196、src/vector/build.rs:41-47）
4. P3（疑似）：remove_stale_on 键畸形处理不一致——parts!=3 跳过，行号解析失败却按 0 继续执行 DELETE；因 gc_keys 恒产自 existing_hashes 良构格式而实际不可达，属防御代码内部不一致（src/vector/store.rs:113-127、src/vector/store.rs:143-166）
5. P3：is_comment_line 的 starts_with("*") 把以 * 开头的普通代码行（解引用/指针表达式）误判为注释，docstring 上提窗口（最多 20 行）会把此类行错误吸收进上一符号块（src/vector/chunk.rs:247-255）
6. 主路径干净段：commit_build 事务边界（GC+upsert+meta 同一 unchecked_transaction，网络调用在事务外）、embed 去重槽位回填（空/单条/重复代入均界内）、validate_index_alignment（1-based/乱序/空数组均拒绝）、fingerprint/index_path 在 cli 两个调用点拼接一致——逐行核对与代入验证（src/vector/store.rs:201-232、src/vector/build.rs:71-92、src/vector/embed.rs:72-146）
7. 死代码盘点：with_batch_size 全仓零引用（确定死，最值得先清）；remove_stale 公开包装仅测试调用；RecallEngine 的 tombstone/tombstone_count/needs_compact/compact 仅测试调用（预留入口，文档自认）；descriptions 表为已移除描述层的 schema 残留（建表+GC 空清，永不写入）（src/vector/embed.rs:66-69、src/vector/store.rs:108-110、src/vector/recall.rs:69-97）
