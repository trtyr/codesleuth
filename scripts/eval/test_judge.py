"""judge 解析/容错/请求体离线单测（P002 E2）：全部无网络。"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import judge  # noqa: E402

REPORT = {
    "answer": "重试在 src/retry.rs:7。",
    "findings": [{"statement": "s", "evidence": [{"file": "src/retry.rs", "audit_seq": 2}]}],
    "dead_ends": [],
}


class TestParseJudgeJson(unittest.TestCase):
    def test_clean_json(self):
        text = '{"evidence_grounded": 4, "depth": 3, "honesty": 5, "notes": "ok"}'
        out = judge.parse_judge_json(text)
        self.assertEqual(out["evidence_grounded"], 4.0)
        self.assertEqual(out["depth"], 3.0)
        self.assertEqual(out["honesty"], 5.0)
        self.assertEqual(out["notes"], "ok")

    def test_json_in_prose(self):
        text = '评审如下：{"evidence_grounded": 4, "depth": 3, "honesty": 5, "notes": "x"} 以上。'
        out = judge.parse_judge_json(text)
        self.assertIsNotNone(out)
        self.assertEqual(out["depth"], 3.0)

    def test_out_of_range_rejected(self):
        text = '{"evidence_grounded": 7, "depth": 3, "honesty": 5}'
        self.assertIsNone(judge.parse_judge_json(text))

    def test_missing_dimension_rejected(self):
        text = '{"evidence_grounded": 4, "depth": 3}'
        self.assertIsNone(judge.parse_judge_json(text))

    def test_malformed_rejected(self):
        self.assertIsNone(judge.parse_judge_json("{not json"))

    def test_empty_rejected(self):
        self.assertIsNone(judge.parse_judge_json(""))
        self.assertIsNone(judge.parse_judge_json(None))


class TestBuildPayload(unittest.TestCase):
    def test_temperature_is_zero(self):
        payload = judge.build_payload("q", REPORT)
        self.assertEqual(payload["temperature"], 0)

    def test_messages_bounded_to_report(self):
        payload = judge.build_payload("谁调用了重试？", REPORT)
        system = payload["messages"][0]["content"]
        user = payload["messages"][1]["content"]
        self.assertIn("evidence_grounded", system)
        self.assertIn("depth", system)
        self.assertIn("honesty", system)
        self.assertIn("谁调用了重试？", user)
        self.assertIn("src/retry.rs", user)
        # 审计原文不进 judge：原始审计行带顶层 "kind" 字段，报告 schema 没有
        self.assertNotIn('"kind"', user)

    def test_dimensions_definition(self):
        self.assertEqual(judge.JUDGE_DIMENSIONS, ("evidence_grounded", "depth", "honesty"))


if __name__ == "__main__":
    unittest.main()
