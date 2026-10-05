# AGENTS.md — codesleuth

给 AI coding agent 的项目指引（人也可读）。

## 项目是什么

只读代码侦察 Agent harness CLI（Rust）：三族只读工具（结构图 / 模糊 / 强读）+ agent loop，对目标代码库零写入，产出证据可回溯的侦察报告。

## 硬性约束（违反 = 产品 bug）

状态目录：~/.codesleuth/（config.toml + reports/audit/ledger/eval）；项目级落 <project>/.codesleuth/（索引）。
2. **docs/plantree/ 不进 git**（本地规划态，`.gitignore` 已排除，勿移除该规则）。
3. **密钥不进任何文件**：API key 只从环境变量读取；测试密钥在 engram credentials（`newapi/codesleuth-test`），不写入仓库。
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

- 目标模块地图：`docs/plantree/baseline/module-map.md`
- 权威方案（从这里读起）：`docs/plantree/plans/001-read-only-agent-harness/topics/solution-map.md`
- 为什么这么设计：`docs/plantree/plans/001-read-only-agent-harness/decisions/`（D001-D013）
- eval 体系：`docs/plantree/plans/002-eval-system/`（判分规格一页纸：topics/scoring-spec.md）
- 依赖核验结论：`docs/plantree/plans/001-read-only-agent-harness/topics/research/dependency-findings.md`

## 约定

- 错误一律走 `errors::CsError`（CS 码 + hint + retryable）；exit code 由码段决定（`CsCode::exit_code`）。
- 输出纪律：stderr 走过程，stdout 走结果；`--json` 时 stdout 仅 JSON。
- 测试：LLM 相关用录制回放；真网关测试标 `#[ignore]`；二进制/编码边界必有单测。
- 依赖原则：禁用需要非 Rust 工具链的 feature（如 fff-search 的 `zlob` 需要 zig）。
- eval 题库：每题必须带 `fact_ref`（事实出处 file:line），gold 只能从 fixture 源码事实导出，禁止凭记忆出题。
