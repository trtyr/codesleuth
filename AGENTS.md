# AGENTS.md — codesleuth

给 AI coding agent 的项目指引（人也可读）。

## 项目是什么

只读代码侦察 Agent harness CLI（Rust）：三族只读工具（结构图 / 模糊 / 强读）+ agent loop，对目标代码库零写入，产出证据可回溯的侦察报告。

## 硬性约束（违反 = 产品 bug）

状态目录：~/.codesleuth/（config.toml + reports/audit/ledger/eval）；项目级落 <project>/.codesleuth/（索引）。
2. **docs/plantree/ 不进 git**（本地规划态，`.gitignore` 已排除，勿移除该规则）。
3. **密钥不进任何文件**：API key 住 `~/.codesleuth/config.toml`（用户自担，env 层已移除）；测试密钥在 engram credentials（`newapi/codesleuth-test`），不写入仓库。
4. **上下文压缩必须确定性**（零 LLM 调用），不做散文式摘要（D008）。
5. **方案变更先回写 plan tree**（decisions/ 或 topics/）再动代码——「为什么这么设计」的唯一来源在那里。

## 常用命令

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
bash scripts/ci.sh   # 三连全绿门
python3 -m unittest discover -s scripts/eval -t scripts/eval   # eval 判分离线单测
```

正式 eval（真网关，密钥读 ~/.codesleuth/config.toml）：

```bash
python3 scripts/eval/run_eval.py \
  --suite tests/golden/fixture-rs.json=tests/fixtures/fixture-rs \
  --suite tests/golden/fixture-ts.json=tests/fixtures/fixture-ts \
  --suite tests/golden/fixture-py.json=tests/fixtures/fixture-py
```

向量混合检索（P003）：`codesleuth index <path> --vector` 建索引；run 时 `--vector --repo-map` 启用暖启动 + repo map。

## 模块地图与权威方案

- 本仓公开口径：事实权威 = README + 代码与模块头注释（带设计决策编号 D0xx，注释即决策摘要）。
- 方案原文（solution-map / decisions D001-D014 / eval 判分规格）在作者本地规划态 `docs/plantree/`（.gitignore 排除，**不随仓分发**）——仓内代理按代码与注释自洽理解，勿按图索骥外部路径。

## engram 档案索引（2026-10-07 docs-upload 落地）

- 本项目 = engram project `codesleuth`（id `01a11563-23b0-7eb1-86ac-26598386c52c`，11 分类）；
  codegraph `codesleuth`（head `d980b9ccddaf`，查询前看 freshness，stale 先 sync）
- 文档清单（title → category，doc_search/doc_get 直达）：
  - 00-overview → 总览
  - 01-cli-dispatch / 02-config-cmd / 03-index-cmd → CLI与命令层
  - 01-llm-provider / 02-config-loading → LLM与配置
  - 01-context-compaction / 02-report → 上下文与报告
  - 01-harness / 02-system-prompt → 侦察编排
  - 01-tool-registry / 02-read-tool / 03-fuzzy-search / 04-vector-search-tool → 只读工具面
  - 01-vector-embed / 02-vector-chunk / 03-vector-store / 04-vector-recall / 05-vector-compose / 06-repo-map → 向量检索
  - 01-fence / 02-bootlock / 03-writeguard / 04-audit / 05-evidence → 安全与防护
  - 01-graph-tools / 02-mcp-client → 结构图与MCP
  - 01-error-codes / 02-logging → 错误与日志
  - 开工记录 2026-10-07 → 历史（文档基线 cd0b0d5）
- 图清单（projects 文件区，file_get 直取）：`diagram-00-architecture.html`（全局架构）
  + `diagram-01` ~ `diagram-28`（逐篇配图，序号对应上方文档）
- 检索配方：架构怎么设计 → 00-overview + 侦察编排；一个工具怎么实现 → 只读工具面/向量检索；
  为什么这么定 → docs/plantree（本地）+ 模块头注释 D0xx；报错码什么意思 → 错误与日志
- 更新纪律：文档清单变化时同步更新本段（本段是快照，权威在 engram）

## 约定

- 错误一律走 `errors::CsError`（CS 码 + hint + retryable）；exit code 由码段决定（`CsCode::exit_code`）。
- 输出纪律：stderr 走过程，stdout 走结果；`--json` 时 stdout 仅 JSON。
- 测试：LLM 相关用录制回放；真网关测试标 `#[ignore]`；二进制/编码边界必有单测。
- 依赖原则：禁用需要非 Rust 工具链的 feature（如 fff-search 的 `zlob` 需要 zig）。
- eval 题库：每题必须带 `fact_ref`（事实出处 file:line），gold 只能从 fixture 源码事实导出，禁止凭记忆出题。
