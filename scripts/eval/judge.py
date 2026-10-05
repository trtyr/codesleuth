"""LLM judge 质量分（P002 scoring-spec §三）。

三维 1-5 分：evidence_grounded（证据扎实度）/ depth（结论深度）/ honesty（死胡同诚实度）。
temperature=0 保证可复现；strict JSON 输出 + brace-scan 解析 + 容错
（解析失败/超时/拒答 → 返回 None，由调用方标 judge_degraded，硬门独立成立）。
"""

import json
import urllib.error
import urllib.request

JUDGE_SYSTEM = (
    "你是代码侦察报告的质量评审。对给定报告按三个维度打 1-5 分，只输出 JSON，"
    "不要输出任何其他文字：\n"
    '{"evidence_grounded": <1-5>, "depth": <1-5>, "honesty": <1-5>, "notes": "<一句话依据>"}\n'
    "维度定义：\n"
    "- evidence_grounded 证据扎实度：每条结论是否有具体 file:line 且相互支撑（5=全部有且相互支撑；1=泛泛而谈、证据与结论脱节）\n"
    "- depth 结论深度：是否点到符号/量化关系/指出不一致（5=有量化或矛盾分析；1=复述目录结构）\n"
    "- honesty 死胡同诚实度：该报死胡同时如实报、不编不凑（5=坦率且准确；1=无中生有或掩盖未查清处）\n"
    "只依据报告本身判断，不要臆测报告之外的代码内容。"
)

JUDGE_DIMENSIONS = ("evidence_grounded", "depth", "honesty")


def build_judge_messages(question: str, report: dict) -> list:
    """judge 输入：题目 + 最终报告结构（answer/findings/dead_ends），不带审计原文。"""
    compact = {
        "question": question,
        "answer": report.get("answer"),
        "findings": report.get("findings"),
        "dead_ends": report.get("dead_ends"),
        "confidence": report.get("confidence"),
    }
    return [
        {"role": "system", "content": JUDGE_SYSTEM},
        {"role": "user", "content": json.dumps(compact, ensure_ascii=False)},
    ]


def parse_judge_json(text: str):
    """brace-scan 提取第一个完整 JSON 对象并校验三维齐全、分值在 1-5。失败返回 None。"""
    if not text:
        return None
    start = text.find("{")
    while start != -1:
        depth = 0
        for i in range(start, len(text)):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    candidate = text[start : i + 1]
                    try:
                        obj = json.loads(candidate)
                    except ValueError:
                        break
                    if not isinstance(obj, dict):
                        break
                    if all(
                        isinstance(obj.get(d), (int, float)) and 1 <= obj[d] <= 5
                        for d in JUDGE_DIMENSIONS
                    ):
                        return {d: float(obj[d]) for d in JUDGE_DIMENSIONS} | {
                            "notes": str(obj.get("notes", ""))
                        }
                    break
        start = text.find("{", start + 1)
    return None


def build_payload(question: str, report: dict) -> dict:
    """构造 chat/completions 请求体。temperature=0 是可复现性硬约束（spec §三）。"""
    return {
        "model": "",  # 由调用方填入
        "messages": build_judge_messages(question, report),
        "temperature": 0,
    }


def judge_report(base_url: str, api_key: str, model: str, question: str, report: dict, timeout: int = 60):
    """调 judge 端点。成功返回 {"evidence_grounded","depth","honesty","notes"}；失败返回 None。"""
    url = base_url.rstrip("/") + "/chat/completions"
    payload = build_payload(question, report)
    payload["model"] = model
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json", "Authorization": f"Bearer {api_key}"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            body = json.loads(resp.read().decode("utf-8"))
    except (urllib.error.URLError, OSError, ValueError):
        return None
    try:
        text = body["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError):
        return None
    return parse_judge_json(text)
