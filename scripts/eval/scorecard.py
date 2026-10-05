"""scorecard 构建 / baseline 对比 / 回归判定（P002 scoring-spec §四）。

回归规则：hard_gate_rate 下降（严格 < baseline），或 judge 任一维下降 > 0.5。
退出码：0 = 无回归；1 = 用法错误；2 = 网关不可用；5 = 检测到回归。
"""

import json
import os
import sys
import time

SCORECARD_VERSION = 1
JUDGE_DIMENSIONS = ("evidence_grounded", "depth", "honesty")
JUDGE_REGRESSION_THRESHOLD = 0.5

EXIT_OK = 0
EXIT_USAGE = 1
EXIT_GATEWAY = 2
EXIT_REGRESSION = 5


def resolve_state_dir() -> str:
    """全局家（2026-10-05 归位）：~/.codesleuth（与 Rust 侧 global_state_dir 对齐）。"""
    return os.path.expanduser("~/.codesleuth")


def eval_dir() -> str:
    return os.path.join(resolve_state_dir(), "eval")


def build_scorecard(suites: list, model: str, judge_model: str, codesleuth_version: str = "unknown") -> dict:
    """suites: [{fixture, questions, hard_gate_pass, judge:{dim|None}, judge_degraded_count, tokens, duration_ms}]"""
    total_q = sum(s["questions"] for s in suites)

    def avg(dim):
        vals = [s["judge"][dim] for s in suites if (s.get("judge") or {}).get(dim) is not None]
        return round(sum(vals) / len(vals), 3) if vals else None

    totals = {
        "questions": total_q,
        "hard_gate_rate": round(sum(s["hard_gate_pass"] for s in suites) / total_q, 4) if total_q else 0.0,
        "judge_avg": {d: avg(d) for d in JUDGE_DIMENSIONS},
    }
    return {
        "scorecard_version": SCORECARD_VERSION,
        "codesleuth_version": codesleuth_version,
        "model": model,
        "judge_model": judge_model,
        "ran_at": int(time.time()),
        "suites": suites,
        "totals": totals,
    }


def compare_to_baseline(scorecard: dict, baseline: dict) -> tuple:
    """返回 (regressed, delta, reasons)。升/平 → False；hard_gate_rate 下降或 judge 维降幅 > 阈值 → True。"""
    t = scorecard["totals"]
    b = baseline["totals"]
    reasons: list = []
    d_hard = round(t["hard_gate_rate"] - b["hard_gate_rate"], 4)
    delta = {"hard_gate_rate": d_hard, "judge_avg": {}}
    if d_hard < 0:
        reasons.append(f"hard_gate_rate 下降 {d_hard}")
    for dim in JUDGE_DIMENSIONS:
        cur = (t.get("judge_avg") or {}).get(dim)
        base = (b.get("judge_avg") or {}).get(dim)
        if cur is None or base is None:
            delta["judge_avg"][dim] = None
            continue
        d = round(cur - base, 3)
        delta["judge_avg"][dim] = d
        if d < -JUDGE_REGRESSION_THRESHOLD:
            reasons.append(f"judge.{dim} 下降 {-d}（> {JUDGE_REGRESSION_THRESHOLD}）")
    return (len(reasons) > 0, delta, reasons)


def attach_baseline(scorecard: dict, baseline: dict, source: str, ref: str) -> dict:
    regressed, delta, reasons = compare_to_baseline(scorecard, baseline)
    scorecard["baseline"] = {"source": source, "ref": ref, "delta": delta}
    scorecard["regression"] = {
        "detected": regressed,
        "rule": "hard_gate_rate 下降 或 judge 任一维下降 > 0.5",
        "reasons": reasons,
    }
    return scorecard


def save_scorecard(scorecard: dict) -> str:
    d = eval_dir()
    os.makedirs(d, exist_ok=True)
    path = os.path.join(d, f"scorecard-{scorecard['ran_at']}.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(scorecard, fh, ensure_ascii=False, indent=2)
    return path


def load_scorecard(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def baseline_pointer_path() -> str:
    return os.path.join(eval_dir(), "baseline.json")


def load_baseline() -> tuple:
    """返回 (baseline_dict | None, source, ref)。指针文件存 {"ref": "<scorecard 路径>"}。"""
    ptr = baseline_pointer_path()
    if not os.path.exists(ptr):
        return None, "none", ""
    with open(ptr, "r", encoding="utf-8") as fh:
        ref = json.load(fh).get("ref", "")
    if not ref or not os.path.exists(ref):
        return None, "none", ref
    return load_scorecard(ref), "baseline", ref


def set_baseline(path: str) -> None:
    os.makedirs(eval_dir(), exist_ok=True)
    with open(baseline_pointer_path(), "w", encoding="utf-8") as fh:
        json.dump({"ref": os.path.abspath(path)}, fh, ensure_ascii=False)
