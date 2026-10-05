"""eval runner（P002 E2/E3）：跑题 → 硬门 → judge → scorecard → 回归退出。

用法：
  python3 scripts/eval/run_eval.py <golden.json> <fixture_dir> [options]

options:
  --no-judge          跳过 judge 层（离线可用，scorecard 标注 judge=null）
  --set-baseline F    跑完后把 F（或本次）设为 baseline
  --codesleuth-bin P  被测二进制（默认 ./target/debug/codesleuth）
  --judge-model M     judge 模型（默认取 ~/.codesleuth/config.toml 的 model）
  --base-url U        judge 端点（默认取 config.toml 的 base_url）

密钥/端点/模型：一律读 ~/.codesleuth/config.toml（环境变量层已移除）。
"""

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hardgate  # noqa: E402
import judge  # noqa: E402
import scorecard  # noqa: E402

AUDIT_LINE = re.compile(r"^#\s*审计:\s*(.+)$", re.M)
QUESTION_TIMEOUT = 3600  # 大仓首次 --vector 会话内建索引可达数十分钟（P003 E3-R1 教训）


def run_one_question(codesleuth_bin: str, question: str, fixture_dir: str, extra_args: list | None = None) -> tuple:
    """跑一题。返回 (report | None, audit_path | None, stderr_text, returncode, duration_ms)。"""
    started = time.monotonic()
    proc = subprocess.run(
        [codesleuth_bin, question, "--repo", fixture_dir, "--json", *(extra_args or [])],
        capture_output=True,
        text=True,
        timeout=QUESTION_TIMEOUT,
    )
    duration_ms = int((time.monotonic() - started) * 1000)
    report = None
    if proc.returncode == 0 and proc.stdout.strip():
        try:
            report = json.loads(proc.stdout)
        except ValueError:
            report = None
    m = AUDIT_LINE.search(proc.stderr or "")
    audit_path = m.group(1).strip() if m else None
    return report, audit_path, proc.stderr or "", proc.returncode, duration_ms


def eval_suite(codesleuth_bin, golden, fixture_dir, use_judge, judge_cfg, extra_args: list | None = None) -> dict:
    """跑完一套题，返回 suite 统计（judge_cfg None 表示离线模式）。"""
    questions = golden["questions"]
    hard_pass = 0
    judge_scores = {d: [] for d in judge.JUDGE_DIMENSIONS}
    judge_degraded = 0
    tokens = 0
    duration_ms = 0
    failures_log = []

    for idx, q in enumerate(questions, 1):
        question = q["q"]
        report, audit_path, stderr, code, dur = run_one_question(codesleuth_bin, question, fixture_dir, extra_args)
        duration_ms += dur
        if code != 0 or report is None:
            failures_log.append({"q": question, "failures": [f"进程退出码 {code}: {stderr[-200:]}"]})
            print(f"[{idx}/{len(questions)}] FAIL(进程) {question}")
            continue
        tokens += int((report.get("stats") or {}).get("total_tokens") or 0)
        audit_rows = hardgate.load_audit_rows(audit_path) if audit_path else {}
        ok, failures = hardgate.check_hard_gate(report, audit_rows, q)
        entry = {"q": question, "failures": failures}
        if not ok:
            failures_log.append(entry)
            print(f"[{idx}/{len(questions)}] FAIL(硬门) {question} :: {'; '.join(failures)}")
            continue
        hard_pass += 1
        if use_judge and judge_cfg:
            dims = judge.judge_report(
                judge_cfg["base_url"], judge_cfg["api_key"], judge_cfg["model"], question, report
            )
            if dims is None:
                judge_degraded += 1
            else:
                for d in judge.JUDGE_DIMENSIONS:
                    judge_scores[d].append(dims[d])
            print(f"[{idx}/{len(questions)}] PASS {question} :: judge={dims}")
        else:
            print(f"[{idx}/{len(questions)}] PASS {question}")

    suite_judge = {
        d: (round(sum(v) / len(v), 3) if v else None) for d, v in judge_scores.items()
    }
    return {
        "fixture": golden["fixture"],
        "questions": len(questions),
        "hard_gate_pass": hard_pass,
        "judge": suite_judge,
        "judge_degraded_count": judge_degraded,
        "tokens": tokens,
        "duration_ms": duration_ms,
        "_failures_log": failures_log,
    }


def read_global_config() -> dict:
    """读 ~/.codesleuth/config.toml 的 [llm]（tomllib，py3.11+；环境变量层已移除）。"""
    import tomllib
    from pathlib import Path
    p = Path.home() / ".codesleuth" / "config.toml"
    if not p.exists():
        return {}
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    return data.get("llm", {}) or {}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="codesleuth 正式 eval（P002）")
    ap.add_argument("--suite", action="append", required=True,
                    metavar="GOLDEN=FIXTURE_DIR",
                    help="题库=fixture 目录，可重复（如 tests/golden/fixture-rs.json=tests/fixtures/fixture-rs）")
    ap.add_argument("--no-judge", action="store_true")
    ap.add_argument("--set-baseline", metavar="FILE", help="把指定 scorecard（或缺省为本次）设为 baseline")
    ap.add_argument("--codesleuth-bin", default=os.path.join("target", "debug", "codesleuth"))
    ap.add_argument("--judge-model", default="")
    ap.add_argument("--base-url", default="")
    ap.add_argument("--extra-args", default="", help="透传给 codesleuth 子进程的额外参数（如 --vector）")
    ap.add_argument("--codesleuth-version", default="unknown")
    args = ap.parse_args(argv)

    goldens = []
    fixture_dirs = []
    for s in args.suite:
        if "=" not in s:
            print(f"--suite 格式应为 GOLDEN=FIXTURE_DIR: {s}")
            return scorecard.EXIT_USAGE
        g, f = s.split("=", 1)
        try:
            with open(g, "r", encoding="utf-8") as fh:
                goldens.append(json.load(fh))
        except OSError as e:
            print(f"题库读取失败 {g}: {e}")
            return scorecard.EXIT_USAGE
        fixture_dirs.append(f)

    use_judge = not args.no_judge
    cfg = read_global_config()
    extra_args = shlex.split(args.extra_args)
    judge_cfg = None
    if use_judge:
        api_key = cfg.get("api_key", "")
        if not api_key:
            print("judge 需要 ~/.codesleuth/config.toml 的 [llm] api_key（或使用 --no-judge）")
            return scorecard.EXIT_USAGE
        judge_cfg = {
            "base_url": args.base_url,
            "api_key": api_key,
            "model": args.judge_model or "gpt-4o-mini",
        }

    suites = []
    for golden, fixture_dir in zip(goldens, fixture_dirs):
        suites.append(
            eval_suite(args.codesleuth_bin, golden, fixture_dir, use_judge, judge_cfg, extra_args)
        )
    # 关键字传参防位错（审计发现：位置参数曾把版本号填进 judge_model 字段）
    sc = scorecard.build_scorecard(
        suites,
        model=cfg.get("model", "unknown"),
        judge_model=judge_cfg["model"] if judge_cfg else "none",
        codesleuth_version=args.codesleuth_version,
    )
    sc["_failures_log"] = [f for s in suites for f in s["_failures_log"]]

    baseline, source, ref = scorecard.load_baseline()
    if baseline is None:
        source = "auto-first"
        ref = "(本次)"
    sc = scorecard.attach_baseline(sc, baseline or sc, source, ref)
    path = scorecard.save_scorecard(sc)
    if args.set_baseline:
        target = args.set_baseline if args.set_baseline != "this" else path
        scorecard.set_baseline(target)
        print(f"baseline 已设为 {target}")

    t = sc["totals"]
    for s in suites:
        print(
            f"\n=== {s['fixture']}: 硬门 {s['hard_gate_pass']}/{s['questions']}"
            f" · judge={s['judge']} · tokens={s['tokens']} ==="
        )
    print(f"\n=== 总计: 硬门率 {t['hard_gate_rate']} · judge_avg={t['judge_avg']} ===")
    print(f"scorecard: {path}")
    if sc["regression"]["detected"]:
        print(f"!!! 回归：{'; '.join(sc['regression']['reasons'])}")
        return scorecard.EXIT_REGRESSION
    return scorecard.EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
