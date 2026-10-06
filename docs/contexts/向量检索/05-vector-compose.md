# 向量组装（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量组装（src/vector/compose.rs）提供 compose_input(chunk, mode) 将每个代码 chunk 序列化为嵌入输入；EmbedMode 含 Raw（返回 chunk.text 全文）与 Composite（白拿层 header + 标识符层去重封顶 20，默认模式）两种。模式由 CLI 从 cfg.vector.embed_mode 解析（"raw"→Raw，否则→Composite），由 build_vector_index 在嵌入循环中调用，结果送入 EmbedClient::embed 并以 mode 字符串写入索引 meta 参与增量复用与作废判定。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- grep 默认 plain 模式对 compose_input/EmbedMode 返回 0 命中（0/82 文件），改用 regex 后才定位；非实质性死胡同。
- identifiers_of 仅在 compose.rs 内部被引用（grep 1/82 文件），未在其他模块复用，故下游仅写其作为 compose_input Composite 分支的内部依赖。

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=31890ms · tokens=50248

