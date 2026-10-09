# Code Review — codesleuth

- 审查时间：2026-10-09 12:34 · 功能点 28（切片来源：contexts） · 总耗时 1575s

## 总览

| 层 | 检查项 | 发现 | 置信度 | 状态 |
| --- | --- | --- | --- | --- |
| 微观 | [CLI入口与命令分发](features/cli-dispatch.md) | 5 | high | ✓ |
| 微观 | [配置管理子命令](features/config-cmd.md) | 8 | medium | ✓ |
| 微观 | [向量索引子命令](features/index-cmd.md) | 7 | high | ✓ |
| 微观 | [LLM provider抽象](features/llm-provider.md) | 5 | high | ✓ |
| 微观 | [配置加载链](features/config-loading.md) | 6 | high | ✓ |
| 微观 | [上下文压缩](features/context-compaction.md) | 5 | high | ✓ |
| 微观 | [报告生成](features/report.md) | 7 | medium | ✓ |
| 微观 | [Harness主循环](features/harness.md) | 11 | medium | ✓ |
| 微观 | [System Prompt](features/system-prompt.md) | 7 | medium | ✓ |
| 微观 | [工具注册表](features/tool-registry.md) | 8 | high | ✓ |
| 微观 | [read工具](features/read-tool.md) | 7 | high | ✓ |
| 微观 | [模糊搜索工具](features/fuzzy-search.md) | 9 | high | ✓ |
| 微观 | [vector_search工具](features/vector-search-tool.md) | 6 | medium | ✓ |
| 微观 | [向量嵌入](features/vector-embed.md) | 6 | high | ✓ |
| 微观 | [向量切块](features/vector-chunk.md) | 7 | high | ✓ |
| 微观 | [向量存储](features/vector-store.md) | 6 | medium | ✓ |
| 微观 | [向量召回引擎](features/vector-recall.md) | 6 | high | ✓ |
| 微观 | [向量组装](features/vector-compose.md) | 6 | high | ✓ |
| 微观 | [repo map预算注入](features/repo-map.md) | 6 | high | ✓ |
| 微观 | [Fence路径围栏](features/fence.md) | 8 | high | ✓ |
| 微观 | [bootlock跨进程锁](features/bootlock.md) | 5 | high | ✓ |
| 微观 | [writeguard](features/writeguard.md) | 6 | high | ✓ |
| 微观 | [审计日志](features/audit.md) | 7 | high | ✓ |
| 微观 | [证据库](features/evidence.md) | 10 | high | ✓ |
| 微观 | [结构图工具集](features/graph-tools.md) | 9 | high | ✓ |
| 微观 | [MCP客户端](features/mcp-client.md) | 10 | high | ✓ |
| 微观 | [错误码体系](features/error-codes.md) | 7 | high | ✓ |
| 微观 | [日志双轨](features/logging.md) | 7 | medium | ✓ |
| 宏观 | [aifriendly](global/aifriendly.md) | 8 | high | ✓ |
| 宏观 | [config](global/config.md) | 12 | high | ✓ |
| 宏观 | [coupling](global/coupling.md) | 8 | medium | ✓ |
| 宏观 | [errors](global/errors.md) | 9 | high | ✓ |
| 宏观 | [logging](global/logging.md) | 9 | high | ✓ |
| 宏观 | [overdesign](global/overdesign.md) | 9 | high | ✓ |

## 说明

- 微观 = 功能点调用链通读（bug 悬赏 + 死代码）；宏观 = 系统级体系横审
- 未审/失败的检查项重跑 `review.py --repo ... --out /tmp/pi/01a11ae1-f1ec-7462-bb76-9a84c4adf976/cr-test` 自动补审
