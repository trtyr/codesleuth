<div align="center">

# 🔍 codesleuth

**进代码库找东西，带证据回来。**

只读侦察 Agent · 每条结论可回溯到 `file:line` · 对目标仓库**零源码写入**

[![tests](https://img.shields.io/badge/tests-90_passing-brightgreen)](scripts/ci.sh)
[![eval](https://img.shields.io/badge/eval-45%2F45_hardgate-brightgreen)](scripts/eval/run_eval.py)
[![rust](https://img.shields.io/badge/rust-1.88%2B-orange)](Cargo.toml)
[![license](https://img.shields.io/badge/license-MIT-blue)](#license)

```bash
codesleuth "这个仓库的重试逻辑在哪？给出 file:line 证据" --repo ./your-project
```

</div>

---

## 🤔 它是什么

一个 Rust 实现的**只读侦察 Agent CLI**。你扔一个问题进去，它自己看结构图、搜文件、读源码、验证证据，还你一份结构化报告——**结论、证据（file:line + 审计号）、走过的死胡同、置信度**。

它对目标仓库**一个字节都不改**：工具面物理无写能力、路径围栏拒 symlink 逃逸、运行前后逐文件比对自证零写入。

## ⚙️ 为何不同

|            | 直接扔给 LLM | grep 脚本  | codesleuth                                       |
| ---------- | ------------ | ---------- | ------------------------------------------------ |
| 大仓库     | 上下文装不下 | 人肉拼线索 | 结构图开局，一次拿到调用路径 + 影响面            |
| 答案可信度 | 看模型心情   | 无语义理解 | 引用了没真读过的文件？报告直接拒收重写           |
| 可审计性   | 黑盒         | 无记录     | 全量审计流水，报告 ↔ 审计双向回溯                |
| 长任务     | 窗口爆掉     | —          | 确定性上下文压缩，`recall` 续读原文，不重不漏    |
| 可靠性     | 可能空转烧钱 | —          | 空转结构性禁止：重复调用拒绝、零增量转向、熔断   |

## 📦 安装

```bash
cargo install codesleuth

# 结构图工具（可选；未安装时自动降级，其余照常）
npm i -g @colbymchenry/codegraph
```

<details>
<summary>从源码安装</summary>

```bash
git clone https://github.com/trtyr/codesleuth && cd codesleuth
cargo install --path .
```
</details>

## 🚀 快速上手

一切配置住在一个地方：**`~/.codesleuth/config.toml`**

```toml
[llm]
base_url = "https://api.openai.com/v1"   # 任意 OpenAI 兼容端点
api_key = "sk-..."
model = "gpt-4o-mini"

[vector]
embed_model = "Qwen/Qwen3-Embedding-8B"  # 语义召回（--vector）用
embed_dims = 1024
embed_mode = "composite"                 # raw | composite
repomap_budget = 24000                   # 任务导航图预算（字符）
# base_url = "https://embedding-provider.example/v1"   # 嵌入独立供应商（缺省跟随 [llm]）
# api_key = "sk-..."                                   # 嵌入独立密钥（缺省跟随 [llm].api_key）

[behavior]
thinking_on = false                      # 检索型任务默认关思考

# 多模型档位（D017）：不同任务用不同模型，--profile 运行时选用
[llm.profiles.scout]                     # 例：只读侦察专用
model = "some-fast-model"
model_context_tokens = 131072            # 可选；base_url / api_key 同理可覆盖，缺席跟随主 [llm]

[llm.profiles.review]                    # 例：review 专用
model = "some-strong-model"
```

`--vector` 相关参数全在上面 `[vector]` 段；不配也能跑——不建索引时工具自动降级，`codesleuth index <path> --vector` 时用默认值。

然后问你的第一个问题：

```bash
codesleuth "鉴权中间件在哪，被哪些路由使用？" --repo /path/to/project
```

就这样。它自己看图、搜索、读码、验证，报告自动落盘并告诉你路径。

<details>
<summary><b>更多用法</b></summary>

```bash
# 输出格式 --output-format：report=人类模板（默认）· raw=仅模型正文（交付通道，不套报告壳）· json=结构化报告
# 机器可读 JSON（schema 版本化，供上游 agent 消费）
codesleuth "错误处理有哪些模式？" --repo . --output-format json --out report.json

# 上游编排场景：raw 交付 + 输出契约（回答必须含指定标记，缺失补一轮仍缺则判 CS2005）
codesleuth "整理功能清单" --repo . --output-format raw --require "<!--FEATURE-INDEX-->"

# 多模型档位：--profile 选用 [llm.profiles.<名字>]；--model/--base-url 旗标仍可再压
codesleuth "这个模块的测试覆盖如何？" --repo . --profile review

# 聚焦子目录
codesleuth "错误处理有哪些模式？" --repo . --focus "crates/**"

# 索引管理
codesleuth index /path/to/project --rebuild
codesleuth index /path/to/project --vector      # 语义召回：向量化全仓 + 任务相关导航图
```

`--vector` 说明：首建按块嵌入（一次性，之后 text_hash 增量复用 + 失效块自动清理，改一行只重嵌一块）；索引存在 `<project>/.codesleuth/`，跟项目走。`[vector]` 段不配置时全部走默认值。

</details>

<details>
<summary><b>🧰 agent 工具面</b></summary>

| 工具                                       | 层     | 作用                                                            |
| ------------------------------------------ | ------ | --------------------------------------------------------------- |
| `explore`                                  | 结构图 | 相关符号源码 + 调用路径 + 影响面，一次返回                      |
| `callers` / `callees` / `impact` / `files` | 结构图 | 反向边 / 正向边 / blast radius / 文件清单                       |
| `find_files`                               | 模糊   | frecency 模糊路径搜索                                           |
| `grep`                                     | 模糊   | plain / regex / fuzzy 三态内容检索                              |
| `read`                                     | 强读   | 行号 + 锚点（内容变更锚点失效）、分页续读、二进制识别、编码消毒 |
| `vector_search`                            | 向量   | 语义召回（带相似度），结果须过读层才能成为证据                  |
| `submit_report`                            | 收敛   | 结构化报告提交，**证据必须来自会话内真实读取**，否则拒收        |

</details>

## 🔒 隐私

目标仓库的代码片段只发送到**你配置的 LLM 端点**；除此之外零遥测、零外发。审计日志只存本地 `~/.codesleuth/`。

## 🧪 开发

```bash
bash scripts/ci.sh          # fmt + clippy -D warnings + test
python3 scripts/eval/run_eval.py --help   # 正式 eval：多语言 fixture × 硬门 + judge 判分
```

## License

MIT
