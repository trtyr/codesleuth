# 嵌入供应商独立配置（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
「嵌入供应商独立配置」让向量语义检索的嵌入服务脱离主 LLM 单独选供应商：通过 config 的 [vector] 段（VectorConfig）独立指定嵌入 base_url、api_key、模型、维度与 raw/composite 模式，缺省时自动回退跟随主 [llm] 段，解除 chat 与 embedding 必须同一家供应商的限制。装配时 resolve_embed_endpoint 做 [vector] 优先、[llm] 兜底的解析，再用解析结果构造 EmbedClient 供索引构建使用。

## 证据列表
1. VectorConfig 收编嵌入专用 base_url/api_key（None=跟随 [llm]）及 embed_model/embed_dims/embed_mode 等参数
   - src/config.rs:78-90（审计 #4）
2. resolve_embed_endpoint 实现 [vector] 优先、[llm] 兜底的端点解析
   - src/cli.rs:518-530（审计 #6）
3. setup_vector_layer 调用 resolve_embed_endpoint 后用结果构造 EmbedClient
   - src/cli.rs:545-556（审计 #6）
4. EmbedClient 持有独立 base_url/api_key/model/dimensions，经 OpenAI 兼容 /embeddings 调用嵌入服务
   - src/vector/embed.rs:41-52（审计 #11）
5. 既有初稿文档与本结论一致（结论与证据链可互证）
   - docs/向量语义检索/04-embedding-config.md:5-7（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=8716ms · tokens=13403
