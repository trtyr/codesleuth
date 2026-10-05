"""硬门判分离线单测（P002 E2）：canned 报告 + 审计行，无网络。"""

import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hardgate  # noqa: E402


def make_report(**over):
    report = {
        "report_schema_version": 1,
        "answer": "重试在 src/retry.rs，闭包约束 FnMut。",
        "findings": [
            {
                "statement": "重试核心在 src/retry.rs",
                "evidence": [{"file": "src/retry.rs", "lines": "7-25", "audit_seq": 4}],
                "confidence": "high",
            }
        ],
        "dead_ends": [],
        "confidence": "high",
        "degraded": False,
    }
    report.update(over)
    return report


AUDIT = {
    2: {"seq": 2, "kind": "tool_call", "name": "read"},
    4: {"seq": 4, "kind": "tool_result", "output": "..."},
    6: {"seq": 6, "kind": "llm"},
}

GOLD = {"expect_files": ["src/retry.rs"], "expect_facts": ["FnMut"]}


class TestHardGate(unittest.TestCase):
    def test_all_pass(self):
        ok, failures = hardgate.check_hard_gate(make_report(), AUDIT, GOLD)
        self.assertTrue(ok, failures)
        self.assertEqual(failures, [])

    def test_g1_file_miss(self):
        ok, failures = hardgate.check_hard_gate(
            make_report(answer="完全无关的答案", findings=[]), AUDIT, GOLD
        )
        self.assertFalse(ok)
        self.assertTrue(any(f.startswith("G1") and "src/retry.rs" in f for f in failures))

    def test_g1_fact_case_insensitive(self):
        gold = {"expect_files": ["src/retry.rs"], "expect_facts": ["FNMUT"]}
        ok, _ = hardgate.check_hard_gate(make_report(), AUDIT, gold)
        self.assertTrue(ok)

    def test_g1_fact_miss(self):
        gold = {"expect_files": ["src/retry.rs"], "expect_facts": ["ArcMutex"]}
        ok, failures = hardgate.check_hard_gate(make_report(), AUDIT, gold)
        self.assertFalse(ok)
        self.assertTrue(any("expect_facts" in f for f in failures))

    def test_g2_seq_missing(self):
        report = make_report(
            findings=[{"statement": "x", "evidence": [{"file": "a.rs", "audit_seq": 999}], "confidence": "high"}]
        )
        ok, failures = hardgate.check_hard_gate(report, AUDIT, {"expect_files": ["a.rs"]})
        self.assertFalse(ok)
        self.assertTrue(any("G2" in f and "999" in f for f in failures))

    def test_g2_seq_wrong_kind(self):
        report = make_report(
            findings=[{"statement": "x", "evidence": [{"file": "a.rs", "audit_seq": 6}], "confidence": "high"}]
        )
        ok, failures = hardgate.check_hard_gate(report, AUDIT, {"expect_files": ["a.rs"]})
        self.assertFalse(ok)  # seq 6 是 llm 行，不能当证据

    def test_g2_seq_none(self):
        report = make_report(
            findings=[{"statement": "x", "evidence": [{"file": "a.rs"}], "confidence": "high"}]
        )
        ok, failures = hardgate.check_hard_gate(report, AUDIT, {"expect_files": ["a.rs"]})
        self.assertFalse(ok)
        self.assertTrue(any("缺 audit_seq" in f for f in failures))

    def test_g3_schema_version(self):
        ok, failures = hardgate.check_hard_gate(make_report(report_schema_version=2), AUDIT, GOLD)
        self.assertFalse(ok)
        self.assertTrue(any("G3" in f and "schema" in f for f in failures))

    def test_g3_empty_answer(self):
        ok, failures = hardgate.check_hard_gate(make_report(answer=""), AUDIT, GOLD)
        self.assertFalse(ok)
        self.assertTrue(any("answer" in f for f in failures))

    def test_g3_bad_confidence(self):
        ok, failures = hardgate.check_hard_gate(make_report(confidence="sure"), AUDIT, GOLD)
        self.assertFalse(ok)
        self.assertTrue(any("confidence" in f for f in failures))

    def test_g4_degraded(self):
        ok, failures = hardgate.check_hard_gate(make_report(degraded=True), AUDIT, GOLD)
        self.assertFalse(ok)
        self.assertTrue(any("G4" in f for f in failures))

    def test_load_audit_rows_skips_malformed(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "a.jsonl")
            with open(p, "w", encoding="utf-8") as fh:
                fh.write('{"seq": 1, "kind": "tool_call"}\n')
                fh.write("not-json-line\n")
                fh.write('{"seq": 2, "kind": "llm"}\n')
            rows = hardgate.load_audit_rows(p)
        self.assertEqual(sorted(rows), [1, 2])
        self.assertEqual(rows[2]["kind"], "llm")


if __name__ == "__main__":
    unittest.main()
