"""scorecard / baseline 对比离线单测（P002 E3）：升/平/降三分支 + 阈值边界 + 退出码。"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scorecard  # noqa: E402


def make_totals(hard=1.0, grounded=4.0, depth=4.0, honesty=4.0):
    return {
        "questions": 15,
        "hard_gate_rate": hard,
        "judge_avg": {"evidence_grounded": grounded, "depth": depth, "honesty": honesty},
    }


def make_sc(hard=1.0, grounded=4.0, depth=4.0, honesty=4.0):
    return {
        "scorecard_version": 1,
        "totals": make_totals(hard, grounded, depth, honesty),
        "suites": [],
    }


class TestCompareBaseline(unittest.TestCase):
    def setUp(self):
        self.baseline = make_sc()

    def test_flat_is_not_regression(self):
        regressed, delta, reasons = scorecard.compare_to_baseline(make_sc(), self.baseline)
        self.assertFalse(regressed)
        self.assertEqual(delta["hard_gate_rate"], 0.0)
        self.assertEqual(delta["judge_avg"]["depth"], 0.0)

    def test_improvement_is_not_regression(self):
        regressed, _, _ = scorecard.compare_to_baseline(
            make_sc(depth=4.6, grounded=4.8), self.baseline
        )
        self.assertFalse(regressed)

    def test_hard_gate_drop_is_regression(self):
        regressed, delta, reasons = scorecard.compare_to_baseline(
            make_sc(hard=0.9333), self.baseline
        )
        self.assertTrue(regressed)
        self.assertTrue(any("hard_gate_rate" in r for r in reasons))
        self.assertLess(delta["hard_gate_rate"], 0)

    def test_judge_drop_over_threshold_is_regression(self):
        regressed, _, reasons = scorecard.compare_to_baseline(
            make_sc(depth=3.4), self.baseline  # 降 0.6 > 0.5
        )
        self.assertTrue(regressed)
        self.assertTrue(any("depth" in r for r in reasons))

    def test_judge_drop_at_threshold_is_not_regression(self):
        # 恰好降 0.5：不触发（规则是严格大于 0.5）
        regressed, _, _ = scorecard.compare_to_baseline(make_sc(depth=3.5), self.baseline)
        self.assertFalse(regressed)

    def test_none_dimensions_are_tolerated(self):
        cur = make_sc()
        cur["totals"]["judge_avg"]["depth"] = None
        regressed, delta, _ = scorecard.compare_to_baseline(cur, self.baseline)
        self.assertFalse(regressed)
        self.assertIsNone(delta["judge_avg"]["depth"])


class TestScorecardShape(unittest.TestCase):
    def test_version_and_totals(self):
        suite = {
            "fixture": "fixture-py",
            "questions": 15,
            "hard_gate_pass": 14,
            "judge": {"evidence_grounded": 4.0, "depth": None, "honesty": 5.0},
            "judge_degraded_count": 1,
            "tokens": 1000,
            "duration_ms": 60000,
        }
        sc = scorecard.build_scorecard([suite], "m", "jm", "0.1.0")
        self.assertEqual(sc["scorecard_version"], 1)
        self.assertEqual(sc["totals"]["questions"], 15)
        self.assertAlmostEqual(sc["totals"]["hard_gate_rate"], 0.9333, places=3)
        self.assertIsNone(sc["totals"]["judge_avg"]["depth"])
        self.assertEqual(sc["totals"]["judge_avg"]["honesty"], 5.0)

    def test_attach_baseline_fields(self):
        sc = scorecard.attach_baseline(make_sc(), make_sc(), "auto-first", "(本次)")
        self.assertEqual(sc["baseline"]["source"], "auto-first")
        self.assertFalse(sc["regression"]["detected"])
        self.assertIn("rule", sc["regression"])

    def test_exit_code_constants(self):
        self.assertEqual(scorecard.EXIT_OK, 0)
        self.assertEqual(scorecard.EXIT_REGRESSION, 5)
        self.assertNotEqual(scorecard.EXIT_REGRESSION, 0)  # 回归必须非零退出


if __name__ == "__main__":
    unittest.main()
