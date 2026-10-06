# 多语言 fixture 评测（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
多语言 fixture 评测是 scripts/eval 下的正式评测系统：以 Rust/TS/Python 三个 fixture 仓为被测对象，对 codesleuth 二进制的侦察报告做「硬门规则校验 + LLM judge 三维打分」，产出 scorecard 并支持 baseline 回归比对，作为质量门禁。入口为 run_eval.py（--suite 可重复指定 GOLDEN=FIXTURE_DIR），每题子进程调用被测二进制（run_one_question），经 hardgate 硬门、judge（三维 1-5 分、temperature=0）判分，最后 scorecard 汇总并和 baseline 对比、支持退出码供 CI 使用。上游依赖 golden 题库与 fixtures、~/.codesleuth/config.toml 的 LLM 配置；下游消费为 scorecard/baseline 机制与 README/AGENTS 文档中的 CI 工作流。

## 证据列表
1. 评测以多语言 fixture 仓为对象，验证 codesleuth 报告质量：题库含 fixture-rs/ts/py（AGENTS.md 示例一次跑三个 suite；fixture-rs.json 有 16 题，考察文件定位、事实核对与死代码诚实回答）
   - scripts/eval/run_eval.py:1-14（审计 #2）
   - tests/golden/fixture-rs.json:1-20（审计 #7）
   - AGENTS.md:30-35（审计 #15）
2. 入口是 scripts/eval/run_eval.py 的 main，--suite 重复传入 GOLDEN=FIXTURE_DIR 对；每题通过 run_one_question 子进程运行被测二进制 codesleuth-bin 并解析 JSON 报告与审计路径
   - scripts/eval/run_eval.py:121-133（审计 #2）
   - scripts/eval/run_eval.py:34-52（审计 #2）
3. 调用链：main → eval_suite（run_eval.py:55）→ 每题先 hardgate.check_hard_gate 硬门（run_eval.py:75），通过后调 judge.judge_report 三维打分（run_eval.py:83-85；judge 维度为 evidence_grounded/depth/honesty，1-5 分，temperature=0，失败降级为 judge_degraded）
   - scripts/eval/run_eval.py:55-107（审计 #2）
   - scripts/eval/judge.py:1-5（审计 #4）
4. 判分结果汇入 scorecard.build_scorecard，与 load_baseline 对比并保存 scorecard，--set-baseline 可设基线（run_eval.py:176-193）；README.md:130 将其标注为「正式 eval：多语言 fixture × 硬门 + judge 判分」
   - scripts/eval/run_eval.py:176-193（审计 #2）
   - README.md:130（审计 #10）
5. 上游依赖：golden 题库 JSON、tests/fixtures/ 各语言 fixture 目录，以及 ~/.codesleuth/config.toml 的 [llm] api_key/base_url/model（judge 端点配置）
   - scripts/eval/run_eval.py:110-118（审计 #2）
   - scripts/eval/run_eval.py:154-165（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=18769ms · tokens=35703

