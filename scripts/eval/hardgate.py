"""确定性硬门判分（P002 scoring-spec §二）。

四项校验，任一失败该题 FAIL（一票否决）：
G1 gold 命中：报告文本（answer + findings.statement + evidence.file）包含全部
   expect_files 子串；有 expect_facts 时逐条大小写不敏感命中。
G2 证据可回溯：每条 finding 的每条 evidence.audit_seq 都能在审计行中找到
   kind ∈ {tool_call, tool_result} 的对应行。
G3 schema 合法：report_schema_version == 1、answer 非空、confidence 枚举合法。
G4 非降级：degraded == false（散文兜底不算收敛）。
"""

GATE_KINDS = ("tool_call", "tool_result")
VALID_CONFIDENCE = ("high", "medium", "low")


def report_text(report: dict) -> str:
    """G1 的判定文本：answer + findings 陈述 + 证据文件路径。"""
    parts = [report.get("answer") or ""]
    for f in report.get("findings") or []:
        parts.append(f.get("statement") or "")
        for e in f.get("evidence") or []:
            parts.append(e.get("file") or "")
    return "\n".join(parts)


def check_hard_gate(report: dict, audit_rows: dict, gold: dict) -> tuple:
    """返回 (passed: bool, failures: list[str])。

    audit_rows: {seq: row_dict}，来自该会话审计 JSONL 的全部行。
    gold: {"expect_files": [...], "expect_facts": [...]}，expect_facts 可选。
    """
    failures: list = []
    text = report_text(report)

    # G1 gold 命中
    for f in gold.get("expect_files") or []:
        if f not in text:
            failures.append(f"G1 未命中 expect_files: {f}")
    for fact in gold.get("expect_facts") or []:
        if fact.lower() not in text.lower():
            failures.append(f"G1 未命中 expect_facts: {fact}")

    # G2 证据可回溯
    for f in report.get("findings") or []:
        for e in f.get("evidence") or []:
            seq = e.get("audit_seq")
            if seq is None:
                failures.append("G2 evidence 缺 audit_seq")
                continue
            row = audit_rows.get(seq)
            if row is None or row.get("kind") not in GATE_KINDS:
                failures.append(f"G2 seq {seq} 无法回溯到工具调用行")

    # G3 schema 合法
    if report.get("report_schema_version") != 1:
        failures.append(f"G3 report_schema_version != 1: {report.get('report_schema_version')}")
    if not (report.get("answer") or "").strip():
        failures.append("G3 answer 为空")
    if report.get("confidence") not in VALID_CONFIDENCE:
        failures.append(f"G3 confidence 非法: {report.get('confidence')}")

    # G4 非降级
    if report.get("degraded"):
        failures.append("G4 degraded=true（散文兜底不视为收敛）")

    return (len(failures) == 0, failures)


def load_audit_rows(audit_path: str) -> dict:
    """读审计 JSONL → {seq: row}。seq 冲突时后行覆盖前行（审计本身保证单调）。"""
    rows = {}
    with open(audit_path, "r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                row = json_loads(line)
            except ValueError:
                continue
            seq = row.get("seq")
            if isinstance(seq, int):
                rows[seq] = row
    return rows


def json_loads(text: str) -> dict:
    import json

    return json.loads(text)
